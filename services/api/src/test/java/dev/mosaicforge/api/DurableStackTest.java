package dev.mosaicforge.api;

import static org.junit.jupiter.api.Assertions.*;

import java.nio.file.Files;
import java.nio.file.Path;
import java.sql.DriverManager;
import java.time.Clock;
import java.util.UUID;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.EnabledIfEnvironmentVariable;
import tools.jackson.databind.node.ObjectNode;

@EnabledIfEnvironmentVariable(named = "MOSAICFORGE_TEST_ENGINE_URL", matches = ".+")
class DurableStackTest {
  @Test
  void realRustBlocksPersistExactlyTwelveAcrossControlRestart() throws Exception {
    String url = System.getenv("MOSAICFORGE_TEST_DB_URL"),
        user = System.getenv("MOSAICFORGE_DB_USER"),
        password = System.getenv("MOSAICFORGE_DB_PASSWORD");
    UUID owner = UUID.randomUUID();
    String trace = "00-12345678901234567890123456789012-1234567890123456-01";
    var engine =
        new HttpEngine(
            System.getenv("MOSAICFORGE_TEST_ENGINE_URL"),
            System.getenv("MOSAICFORGE_ENGINE_TOKEN"),
            30000);
    var lastTransport = new java.util.concurrent.atomic.AtomicReference<String>("not called");
    EnginePort observedEngine =
        (path, bytes, parent, pdf) -> {
          try {
            var reply = engine.execute(path, bytes, parent, pdf);
            lastTransport.set("worker HTTP " + reply.status());
            return reply;
          } catch (java.io.IOException
              | java.util.concurrent.TimeoutException
              | InterruptedException failure) {
            lastTransport.set(failure.getClass().getSimpleName());
            throw failure;
          }
        };
    String svg = Files.readString(Path.of(System.getenv("MOSAICFORGE_TEST_SVG")));
    try (var queue =
        new LettuceWorkNotifications(
            System.getenv("MOSAICFORGE_TEST_VALKEY_URI"),
            "mosaicforge:test:" + UUID.randomUUID(),
            System.getenv("MOSAICFORGE_VALKEY_PASSWORD"))) {
      var inspect = Contracts.JSON.createObjectNode();
      inspect.put("requestId", UUID.randomUUID().toString());
      inspect.put("templateSvg", svg);
      var inspected =
          engine.execute("/v1/inspect", Contracts.JSON.writeValueAsBytes(inspect), trace, false);
      assertEquals(200, inspected.status());
      var template = Contracts.JSON.readTree(inspected.body());
      var snapshot =
          (ObjectNode)
              Contracts.JSON.readTree(
                  getClass().getResourceAsStream("/fixtures/preview/export-pdf.json"));
      snapshot.put("templateSvg", svg);
      var generation = (ObjectNode) snapshot.path("job").path("generation");
      generation.put("templateHash", Contracts.text(template, "templateHash"));
      generation.put("count", 1);
      generation.put("seed", "2026");
      generation.put("rulesVersion", "3");
      var rules = (ObjectNode) generation.path("rules");
      rules.put("preserveAspectRatio", true);
      ((ObjectNode) rules.path("scaleX")).put("min", 1).put("max", 4);
      ((ObjectNode) rules.path("scaleY")).put("min", 1).put("max", 4);
      var projects = new JdbcProjectStore(url, user, password);
      projects.ensureUser(owner);
      var project = projects.create(owner, "stack-test");
      projects.append(owner, project.id(), 0, Contracts.JSON.writeValueAsString(snapshot));
      var request = Contracts.JSON.createObjectNode();
      request.put("schemaVersion", "2");
      request.put("projectId", project.id().toString());
      request.put("projectRevision", 1);
      request.set("generation", generation);
      request.set("pdf", snapshot.path("job").path("pdf"));
      request.put("requestedAcceptedCount", 12);
      request.put("maxCandidates", 100);
      request.putObject("execution").put("blockSize", 4).put("concurrency", 1);
      var store = new JdbcJobStore(url, user, password, Clock.systemUTC(), 3);
      var job = store.submit(owner, "stack", JobRequests.canonical(request));
      var coordinator = new JobCoordinator(store, queue, observedEngine);
      assertTrue(coordinator.dispatchOutbox() > 0);
      queue.publish(job.id());
      queue.publish(job.id());
      assertTrue(coordinator.tick(job.id(), trace), lastTransport::get);
      int first = store.find(owner, job.id()).orElseThrow().accepted();
      assertTrue(first > 0);
      store = new JdbcJobStore(url, user, password, Clock.systemUTC(), 3);
      coordinator = new JobCoordinator(store, queue, observedEngine);
      for (int n = 0; n < 30 && store.find(owner, job.id()).orElseThrow().accepted() < 12; n++)
        coordinator.tick(job.id(), trace);
      var finalJob = store.find(owner, job.id()).orElseThrow();
      assertEquals(12, finalJob.accepted());
      assertEquals(finalJob.evaluated(), finalJob.accepted() + finalJob.rejected());
      assertEquals("running", finalJob.state());
      assertFalse(coordinator.tick(job.id(), trace));
      coordinator.consume(trace);
      assertEquals(finalJob, store.find(owner, job.id()).orElseThrow());
      try (var c = DriverManager.getConnection(url, user, password);
          var s =
              c.prepareStatement("SELECT response FROM mosaicforge_job_blocks WHERE job_id=?")) {
        s.setObject(1, job.id());
        try (var rows = s.executeQuery()) {
          int accepted = 0;
          while (rows.next())
            accepted += Contracts.JSON.readTree(rows.getString(1)).path("acceptedCount").asInt();
          assertEquals(12, accepted);
        }
      }
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
}
