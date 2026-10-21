package dev.mosaicforge.api;

import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.UUID;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;

/** One bounded worker; database polling recovers lost advisory messages. */
final class JobRunner implements AutoCloseable {
  private static final org.slf4j.Logger LOG = org.slf4j.LoggerFactory.getLogger(JobRunner.class);
  private final JobStore store;
  private final JobCoordinator coordinator;
  private final EnginePort engine;
  private final ScheduledExecutorService loop = Executors.newSingleThreadScheduledExecutor();
  private final ScheduledExecutorService heartbeats = Executors.newSingleThreadScheduledExecutor();

  JobRunner(JobStore store, WorkNotifications notifications, EnginePort engine) {
    this.store = store;
    this.engine = engine;
    coordinator = new JobCoordinator(store, notifications, engine);
  }

  void start() {
    loop.scheduleWithFixedDelay(this::cycle, 0, 1, TimeUnit.SECONDS);
  }

  void cycle() {
    try {
      for (var id : store.cancelled(10)) cancel(id);
      store.reconcile(10);
      // The database path runs even when Valkey is unavailable.
      for (var id : store.pending(10)) {
        if (Thread.currentThread().isInterrupted()) break;
        String trace = Tracing.context(null);
        if (!finalizeJob(id, trace)) coordinator.tick(id, trace);
      }
      coordinator.dispatchOutbox();
      try {
        coordinator.consume(Tracing.context(null), 10);
      } catch (RuntimeException unavailable) {
        LOG.warn("control.jobs.notifications_unavailable");
      }
    } catch (RuntimeException unavailable) {
      LOG.warn("control.jobs.cycle_unavailable");
    }
  }

  void cancel(UUID id) {
    var body =
        Contracts.JSON.createObjectNode().put("schemaVersion", "1").put("jobId", id.toString());
    try {
      var reply =
          engine.execute(
              "/internal/v1/cancel",
              Contracts.JSON.writeValueAsBytes(body),
              Tracing.context(null),
              false);
      var response = Contracts.JSON.readTree(reply.body());
      if (reply.status() == 200
          && reply.contentType().split(";", 2)[0].trim().equalsIgnoreCase("application/json")
          && new Contracts().valid(response, "cancel-response", "cancel")
          && response.path("jobId").equals(body.path("jobId"))) store.cancellationDelivered(id);
    } catch (java.io.IOException
        | java.util.concurrent.TimeoutException
        | RuntimeException unavailable) {
      LOG.warn("control.jobs.cancel_deferred");
    } catch (InterruptedException interrupted) {
      Thread.currentThread().interrupt();
    }
  }

  boolean finalizeJob(UUID id, String trace) {
    var claimed = store.claimFinal(id, Duration.ofSeconds(60));
    if (claimed.isEmpty()) return false;
    var lease = claimed.get();
    var heartbeat =
        heartbeats.scheduleWithFixedDelay(
            () -> {
              try {
                store.heartbeat(id, lease.fence(), Duration.ofSeconds(60));
              } catch (RuntimeException unavailable) {
                LOG.warn("control.jobs.heartbeat_unavailable");
              }
            },
            15,
            15,
            TimeUnit.SECONDS);
    try {
      var reply =
          engine.execute(
              "/internal/v1/finalize",
              lease.request().getBytes(StandardCharsets.UTF_8),
              trace,
              true);
      if (reply.status() != 200
          || !reply.contentType().split(";", 2)[0].trim().equalsIgnoreCase("application/json"))
        throw new java.io.IOException("Finalization unavailable");
      store.completeFinal(lease, new String(reply.body(), StandardCharsets.UTF_8));
    } catch (java.io.IOException
        | java.util.concurrent.TimeoutException
        | IllegalArgumentException failure) {
      store.retryFinal(
          lease, Duration.ofSeconds(Math.min(30, 1L << Math.min(lease.job().retries(), 5))));
    } catch (InterruptedException interrupted) {
      Thread.currentThread().interrupt();
      store.retryFinal(lease, Duration.ofSeconds(2));
    } finally {
      heartbeat.cancel(false);
    }
    return true;
  }

  @Override
  public void close() {
    loop.shutdownNow();
    heartbeats.shutdownNow();
    try {
      loop.awaitTermination(3, TimeUnit.SECONDS);
      heartbeats.awaitTermination(3, TimeUnit.SECONDS);
    } catch (InterruptedException interrupted) {
      Thread.currentThread().interrupt();
    }
  }
}
