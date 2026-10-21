package dev.mosaicforge.api;

import static org.junit.jupiter.api.Assertions.*;

import java.sql.DriverManager;
import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import java.time.ZoneId;
import java.time.ZoneOffset;
import java.util.UUID;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.Executors;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.EnabledIfEnvironmentVariable;
import tools.jackson.databind.node.ObjectNode;

@EnabledIfEnvironmentVariable(named = "MOSAICFORGE_TEST_DB_URL", matches = ".+")
class JobStoreTest {
  @Test
  void migrationAddsADurableProgressRevision() throws Exception {
    String url = System.getenv("MOSAICFORGE_TEST_DB_URL"),
        user = System.getenv("MOSAICFORGE_DB_USER"),
        password = System.getenv("MOSAICFORGE_DB_PASSWORD");
    new JdbcJobStore(url, user, password, Clock.systemUTC(), 2);
    try (var connection = DriverManager.getConnection(url, user, password);
        var columns =
            connection.getMetaData().getColumns(null, "public", "mosaicforge_jobs", "revision")) {
      assertTrue(columns.next(), "progress revision must persist in PostgreSQL");
    }
  }

  static final class MutableClock extends Clock {
    Instant now = Instant.parse("2026-10-20T00:00:00Z");

    @Override
    public Instant instant() {
      return now;
    }

    @Override
    public ZoneId getZone() {
      return ZoneOffset.UTC;
    }

    @Override
    public Clock withZone(ZoneId zone) {
      return this;
    }
  }

  @Test
  void durableConcurrentSubmissionFencingOrderedConfirmationAndCancellation() throws Exception {
    String url = System.getenv("MOSAICFORGE_TEST_DB_URL"),
        user = System.getenv("MOSAICFORGE_DB_USER"),
        password = System.getenv("MOSAICFORGE_DB_PASSWORD");
    UUID owner = UUID.randomUUID();
    var clock = new MutableClock();
    try {
      var projects = new JdbcProjectStore(url, user, password);
      projects.ensureUser(owner);
      var project = projects.create(owner, "durable test");
      var snapshot =
          Contracts.JSON.readTree(
              getClass().getResourceAsStream("/fixtures/preview/export-pdf.json"));
      projects.append(owner, project.id(), 0, Contracts.JSON.writeValueAsString(snapshot));
      var request = Contracts.JSON.createObjectNode();
      request.put("schemaVersion", "2");
      request.put("projectId", project.id().toString());
      request.put("projectRevision", 1);
      var generation = (ObjectNode) snapshot.path("job").path("generation").deepCopy();
      generation.put("count", 1);
      generation.put("startIndex", "18446744073709551614");
      request.set("generation", generation);
      request.set("pdf", snapshot.path("job").path("pdf"));
      request.put("requestedAcceptedCount", 2);
      request.put("maxCandidates", 4);
      request.putObject("execution").put("blockSize", 2).put("concurrency", 1);
      var store = new JdbcJobStore(url, user, password, clock, 2);
      String wire = JobRequests.canonical(request);
      var ready = new CountDownLatch(2);
      var go = new CountDownLatch(1);
      JobStore.Job job;
      try (var threads = Executors.newFixedThreadPool(2)) {
        java.util.concurrent.Callable<JobStore.Job> submit =
            () -> {
              ready.countDown();
              go.await();
              return store.submit(owner, "same", wire);
            };
        var a = threads.submit(submit);
        var b = threads.submit(submit);
        ready.await();
        go.countDown();
        job = a.get();
        assertEquals(job.id(), b.get().id());
      }
      final var id = job.id();
      assertEquals(0, job.revision());
      assertEquals(1, store.outbox(100).stream().filter(n -> n.jobId().equals(id)).count());
      assertTrue(store.find(UUID.randomUUID(), id).isEmpty());
      request.put("requestedAcceptedCount", 1);
      assertThrows(
          ProjectStore.Conflict.class,
          () -> store.submit(owner, "same", JobRequests.canonical(request)));
      request.put("requestedAcceptedCount", 2);
      request.path("execution");
      ((ObjectNode) request.path("execution")).put("concurrency", 2);
      assertThrows(
          IllegalArgumentException.class,
          () -> store.submit(owner, "unsupported", JobRequests.canonical(request)));
      ((ObjectNode) request.path("execution")).put("concurrency", 1);
      request.put("projectRevision", 2);
      assertThrows(
          ProjectStore.Missing.class,
          () -> store.submit(owner, "foreign", JobRequests.canonical(request)));
      request.put("projectRevision", 1);
      var lease = store.claim(id, Duration.ofSeconds(5)).orElseThrow();
      assertTrue(lease.job().revision() > job.revision());
      assertTrue(store.claim(id, Duration.ofSeconds(5)).isEmpty());
      clock.now = clock.now.plusSeconds(6);
      assertFalse(store.heartbeat(id, lease.fence(), Duration.ofSeconds(5)));
      var replacement =
          new JdbcJobStore(url, user, password, clock, 2)
              .claim(id, Duration.ofSeconds(5))
              .orElseThrow();
      assertEquals(lease.blockRequest(), replacement.blockRequest());
      assertTrue(replacement.fence() > lease.fence());
      String response = response(replacement, true);
      assertFalse(store.confirm(lease, response));
      assertTrue(store.confirm(replacement, response));
      var confirmedRevision = store.find(owner, id).orElseThrow().revision();
      assertTrue(confirmedRevision > replacement.job().revision());
      assertFalse(store.confirm(replacement, response));
      assertEquals(confirmedRevision, store.find(owner, id).orElseThrow().revision());
      assertEquals(
          confirmedRevision,
          new JdbcJobStore(url, user, password, clock, 2).find(owner, id).orElseThrow().revision());
      job = store.find(owner, id).orElseThrow();
      assertEquals(2, job.accepted());
      assertEquals(2, job.evaluated());
      assertNull(job.nextIndex());
      assertEquals("running", job.state());
      assertTrue(store.claim(id, Duration.ofSeconds(5)).isEmpty());
      var finalLease = store.claimFinal(id, Duration.ofSeconds(5)).orElseThrow();
      assertTrue(store.claimFinal(id, Duration.ofSeconds(5)).isEmpty());
      clock.now = clock.now.plusSeconds(6);
      var finalReplacement =
          new JdbcJobStore(url, user, password, clock, 2)
              .claimFinal(id, Duration.ofSeconds(5))
              .orElseThrow();
      assertEquals(finalLease.artifactId(), finalReplacement.artifactId());
      assertEquals(finalLease.request(), finalReplacement.request());
      String sealed = sealed(finalReplacement);
      assertFalse(store.completeFinal(finalLease, sealed));
      assertTrue(store.completeFinal(finalReplacement, sealed));
      assertEquals("succeeded", store.find(owner, id).orElseThrow().state());
      assertEquals(2, store.artifact(owner, id).orElseThrow().pages());
      long doneRevision = store.find(owner, id).orElseThrow().revision();
      store.cancel(owner, id);
      assertFalse(store.completeFinal(finalReplacement, sealed));
      assertEquals(doneRevision, store.find(owner, id).orElseThrow().revision());
      assertTrue(store.artifact(UUID.randomUUID(), id).isEmpty());
      var deferred = store.submit(owner, "deferred", wire);
      var deferredLease = store.claim(deferred.id(), Duration.ofSeconds(5)).orElseThrow();
      store.defer(deferredLease, Duration.ofSeconds(2));
      var deferredJob = store.find(owner, deferred.id()).orElseThrow();
      assertFalse(store.heartbeat(deferred.id(), deferredLease.fence(), Duration.ofSeconds(5)));
      assertFalse(store.confirm(deferredLease, response(deferredLease, true)));
      store.retry(deferredLease);
      assertEquals(deferredJob, store.find(owner, deferred.id()).orElseThrow());
      for (var notice : store.outbox(100))
        if (notice.jobId().equals(deferred.id())) store.delivered(notice.id());
      store.reconcile(100);
      assertTrue(store.outbox(100).stream().noneMatch(n -> n.jobId().equals(deferred.id())));
      clock.now = clock.now.plusSeconds(3);
      store.reconcile(100);
      assertTrue(store.outbox(100).stream().anyMatch(n -> n.jobId().equals(deferred.id())));
      store.cancel(owner, deferred.id());
      var busy = store.submit(owner, "busy", wire);
      var busyEngine =
          (EnginePort)
              (path, bytes, trace, pdf) ->
                  new EnginePort.Reply(429, "application/json", new byte[0]);
      WorkNotifications silent =
          new WorkNotifications() {
            public void publish(UUID id) {}

            public java.util.List<Message> poll(int limit) {
              return java.util.List.of();
            }

            public void acknowledge(String id) {}

            public void close() {}
          };
      var busyCoordinator = new JobCoordinator(store, silent, busyEngine);
      assertFalse(busyCoordinator.tick(busy.id(), "trace"));
      assertTrue(store.claim(busy.id(), Duration.ofSeconds(5)).isEmpty());
      assertEquals(0, store.find(owner, busy.id()).orElseThrow().retries());
      clock.now = clock.now.plusSeconds(3);
      assertTrue(store.claim(busy.id(), Duration.ofSeconds(5)).isPresent());
      var cancelled = store.submit(owner, "cancel", wire);
      var cancelLease = store.claim(cancelled.id(), Duration.ofSeconds(5)).orElseThrow();
      store.cancel(owner, cancelled.id());
      assertFalse(store.confirm(cancelLease, response(cancelLease, true)));
      assertEquals("cancelled", store.find(owner, cancelled.id()).orElseThrow().state());
      var cancelledFinal = store.submit(owner, "cancel-final", wire);
      var acceptedLease = store.claim(cancelledFinal.id(), Duration.ofSeconds(5)).orElseThrow();
      assertTrue(store.confirm(acceptedLease, response(acceptedLease, true)));
      var cancelledFinalLease =
          store.claimFinal(cancelledFinal.id(), Duration.ofSeconds(5)).orElseThrow();
      store.cancel(owner, cancelledFinal.id());
      assertFalse(store.completeFinal(cancelledFinalLease, sealed(cancelledFinalLease)));
      assertTrue(store.artifact(owner, cancelledFinal.id()).isEmpty());
      assertTrue(store.cancelled(100).contains(cancelledFinal.id()));
      store.cancellationDelivered(cancelledFinal.id());
      assertFalse(store.cancelled(100).contains(cancelledFinal.id()));
      var retry = store.submit(owner, "retry", wire);
      var first = store.claim(retry.id(), Duration.ofSeconds(5)).orElseThrow();
      store.retry(first);
      var second = store.claim(retry.id(), Duration.ofSeconds(5)).orElseThrow();
      assertEquals(first.blockRequest(), second.blockRequest());
      store.retry(second);
      assertEquals("failed", store.find(owner, retry.id()).orElseThrow().state());
      var exhausted = store.submit(owner, "exhaust", wire);
      var exhaustedLease = store.claim(exhausted.id(), Duration.ofSeconds(5)).orElseThrow();
      assertTrue(store.confirm(exhaustedLease, response(exhaustedLease, false)));
      assertEquals("failed", store.find(owner, exhausted.id()).orElseThrow().state());
      var forged = store.submit(owner, "forged", wire);
      var forgedLease = store.claim(forged.id(), Duration.ofSeconds(5)).orElseThrow();
      var bad = (ObjectNode) Contracts.JSON.readTree(response(forgedLease, true));
      bad.put("acceptedCount", 1);
      assertThrows(
          IllegalArgumentException.class,
          () -> store.confirm(forgedLease, JobRequests.canonical(bad)));
      assertEquals(0, store.find(owner, forged.id()).orElseThrow().evaluated());
      clock.now = clock.now.plusSeconds(6);
      assertTrue(store.reconcile(100) >= 1);
      if (System.getenv("MOSAICFORGE_TEST_VALKEY_URI") != null) {
        String uri = System.getenv("MOSAICFORGE_TEST_VALKEY_URI"),
            secret = System.getenv("MOSAICFORGE_VALKEY_PASSWORD"),
            stream = "mosaicforge:test:" + UUID.randomUUID();
        var unavailable = new LettuceWorkNotifications(uri, stream, secret);
        unavailable.close();
        var engine =
            (EnginePort)
                (path, bytes, trace, pdf) -> {
                  throw new java.io.IOException("controlled transport failure");
                };
        assertEquals(0, new JobCoordinator(store, unavailable, engine).dispatchOutbox());
        assertFalse(store.outbox(100).isEmpty());
        try (var queue = new LettuceWorkNotifications(uri, stream, secret)) {
          var pending = store.outbox(100).getFirst();
          queue.publish(pending.jobId());
          var coordinator = new JobCoordinator(store, queue, engine);
          assertTrue(coordinator.dispatchOutbox() > 0);
          assertTrue(queue.poll(100).size() >= 2);
          var transport = store.submit(owner, "transport", wire);
          assertFalse(
              coordinator.tick(
                  transport.id(), "00-12345678901234567890123456789012-1234567890123456-01"));
          assertEquals(0, store.find(owner, transport.id()).orElseThrow().evaluated());
          for (var notice : queue.poll(100)) queue.acknowledge(notice.id());
        }
      }
      new JdbcProjectStore(url, user, password);
    } finally {
      try (var c = DriverManager.getConnection(url, user, password)) {
        c.setAutoCommit(false);
        for (String sql :
            new String[] {
              "DELETE FROM mosaicforge_job_outbox WHERE job_id IN (SELECT id FROM mosaicforge_jobs WHERE owner_id=?)",
              "DELETE FROM mosaicforge_job_blocks WHERE job_id IN (SELECT id FROM mosaicforge_jobs WHERE owner_id=?)",
              "DELETE FROM mosaicforge_jobs WHERE owner_id=?",
              "DELETE FROM mosaicforge_project_revisions WHERE project_id IN (SELECT id FROM mosaicforge_projects WHERE owner_id=?)",
              "DELETE FROM mosaicforge_projects WHERE owner_id=?",
              "DELETE FROM mosaicforge_users WHERE id=?"
            }) {
          try (var s = c.prepareStatement(sql)) {
            s.setObject(1, owner);
            s.executeUpdate();
          }
        }
        c.commit();
      }
    }
  }

  static String sealed(JobStore.FinalLease lease) {
    var response = Contracts.JSON.createObjectNode();
    response
        .put("schemaVersion", "1")
        .put("jobId", lease.job().id().toString())
        .put("artifactId", lease.artifactId().toString())
        .put("requestHash", JobRequests.hash(lease.request()))
        .put("documentConfigurationHash", "a".repeat(64))
        .put("pageCount", lease.job().accepted())
        .put("sha256", "b".repeat(64))
        .put("bytes", "200")
        .put("manifestSha256", "c".repeat(64))
        .put("manifestBytes", "100")
        .put("selectionSha256", "d".repeat(64));
    return JobRequests.canonical(response);
  }

  static String response(JobStore.Lease lease, boolean accepted) {
    var request = Contracts.JSON.readTree(lease.blockRequest());
    var result = Contracts.JSON.createObjectNode();
    result.put("schemaVersion", "1");
    result.set("jobId", request.path("jobId"));
    result.set("blockId", request.path("blockId"));
    result.put("requestHash", JobRequests.hash(lease.blockRequest()));
    result.set("startIndex", request.path("startIndex"));
    int count = request.path("candidateCount").asInt();
    result.put("evaluatedCount", count);
    result.put("acceptedCount", accepted ? count : 0);
    result.put("rejectedCount", accepted ? 0 : count);
    result.putNull("nextCandidateIndex");
    result.put("recordsId", UUID.randomUUID().toString());
    var records = result.putArray("records");
    var index = JobRequests.u64(Contracts.text(request, "startIndex"));
    for (int n = 0; n < count; n++) {
      var record = records.addObject();
      record.put("index", index.add(java.math.BigInteger.valueOf(n)).toString());
      record.put("accepted", accepted);
      if (accepted) record.putNull("reason");
      else record.put("reason", "empty_crop");
    }
    result.put("recordsSha256", JobRequests.hash(JobRequests.canonical(records)));
    return JobRequests.canonical(result);
  }
}
