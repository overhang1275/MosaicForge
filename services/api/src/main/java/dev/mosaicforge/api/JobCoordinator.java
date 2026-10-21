package dev.mosaicforge.api;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.UUID;

/** Explicitly invoked internal component; no scheduler or public routes before finalization. */
final class JobCoordinator {
  private final JobStore store;
  private final WorkNotifications notifications;
  private final EnginePort engine;

  JobCoordinator(JobStore store, WorkNotifications notifications, EnginePort engine) {
    this.store = store;
    this.notifications = notifications;
    this.engine = engine;
  }

  int dispatchOutbox() {
    int sent = 0;
    for (var notice : store.outbox(100)) {
      try {
        notifications.publish(notice.jobId());
        store.delivered(notice.id());
        sent++;
      } catch (RuntimeException unavailable) {
        break;
      }
    }
    return sent;
  }

  boolean tick(UUID id, String trace) {
    var claimed = store.claim(id, Duration.ofSeconds(60));
    if (claimed.isEmpty()) return false;
    var lease = claimed.get();
    try {
      var response =
          engine.execute(
              "/internal/v1/blocks",
              lease.blockRequest().getBytes(StandardCharsets.UTF_8),
              trace,
              false);
      if (response.status() == 429) {
        store.defer(lease, Duration.ofSeconds(2));
        return false;
      }
      if (response.status() != 200
          || !response.contentType().split(";", 2)[0].trim().equalsIgnoreCase("application/json"))
        throw new IOException("Block unavailable");
      return store.confirm(lease, new String(response.body(), StandardCharsets.UTF_8));
    } catch (IOException
        | java.util.concurrent.TimeoutException
        | IllegalArgumentException failure) {
      store.retry(lease);
      return false;
    } catch (InterruptedException interrupted) {
      Thread.currentThread().interrupt();
      store.retry(lease);
      return false;
    }
  }

  int consume(String trace) {
    return consume(trace, 100);
  }

  int consume(String trace, int limit) {
    int count = 0;
    for (var message : notifications.poll(limit)) {
      tick(message.jobId(), trace);
      notifications.acknowledge(message.id());
      count++;
    }
    return count;
  }

  int reconcile() {
    return store.reconcile(100);
  }
}
