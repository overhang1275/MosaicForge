package dev.mosaicforge.api;

import static org.junit.jupiter.api.Assertions.*;

import com.sun.net.httpserver.HttpServer;
import java.net.InetSocketAddress;
import java.net.URI;
import java.net.http.*;
import java.nio.charset.StandardCharsets;
import java.sql.DriverManager;
import java.util.*;
import java.util.concurrent.atomic.AtomicBoolean;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.EnabledIfEnvironmentVariable;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.web.server.context.WebServerApplicationContext;
import tools.jackson.databind.node.ObjectNode;

@EnabledIfEnvironmentVariable(named = "MOSAICFORGE_TEST_VALKEY_URI", matches = ".+")
class JobsHttpTest {
  private static final String TOKEN = "a".repeat(64), KEY = "durable-http-idempotency";

  @Test
  void authenticatedDurableLifecycleAndStreamingRanges() throws Exception {
    UUID owner = UUID.randomUUID();
    String url = System.getenv("MOSAICFORGE_TEST_DB_URL"),
        user = System.getenv("MOSAICFORGE_DB_USER"),
        password = System.getenv("MOSAICFORGE_DB_PASSWORD");
    byte[] pdf = "%PDF-1.7\nverified-test-document\n".getBytes(StandardCharsets.UTF_8),
        manifest = "{\"index\":\"0\"}\n".getBytes(StandardCharsets.UTF_8);
    var busy = new AtomicBoolean(false);
    var corrupt = new AtomicBoolean(false);
    var malformedCancel = new AtomicBoolean(false);
    var cancellationCalls = new java.util.concurrent.atomic.AtomicInteger();
    var server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
    server.createContext(
        "/internal/v1/blocks",
        exchange -> {
          var request = Contracts.JSON.readTree(exchange.getRequestBody().readAllBytes());
          if (busy.get()) {
            exchange.sendResponseHeaders(429, -1);
            exchange.close();
            return;
          }
          var result = Contracts.JSON.createObjectNode();
          result.put("schemaVersion", "1");
          result.set("jobId", request.path("jobId"));
          result.set("blockId", request.path("blockId"));
          result.put("requestHash", JobRequests.hash(JobRequests.canonical(request)));
          result.set("startIndex", request.path("startIndex"));
          int count =
              Math.min(
                  request.path("acceptedLimit").asInt(), request.path("candidateCount").asInt());
          result.put("evaluatedCount", count).put("acceptedCount", count).put("rejectedCount", 0);
          var start = JobRequests.u64(Contracts.text(request, "startIndex"));
          result.put(
              "nextCandidateIndex", start.add(java.math.BigInteger.valueOf(count)).toString());
          result.put("recordsId", UUID.randomUUID().toString());
          var records = result.putArray("records");
          for (int n = 0; n < count; n++)
            records
                .addObject()
                .put("index", start.add(java.math.BigInteger.valueOf(n)).toString())
                .put("accepted", true)
                .putNull("reason");
          result.put("recordsSha256", JobRequests.hash(JobRequests.canonical(records)));
          byte[] bytes = Contracts.JSON.writeValueAsBytes(result);
          exchange.getResponseHeaders().set("Content-Type", "application/json");
          exchange.sendResponseHeaders(200, bytes.length);
          exchange.getResponseBody().write(bytes);
          exchange.close();
        });
    server.createContext(
        "/internal/v1/finalize",
        exchange -> {
          byte[] wire = exchange.getRequestBody().readAllBytes();
          var request = Contracts.JSON.readTree(wire);
          var result = Contracts.JSON.createObjectNode().put("schemaVersion", "1");
          result.set("jobId", request.path("jobId"));
          result.set("artifactId", request.path("artifactId"));
          result
              .put("requestHash", Contracts.sha256(wire))
              .put("documentConfigurationHash", "a".repeat(64))
              .put("pageCount", request.path("requestedAcceptedCount").asInt())
              .put("sha256", Contracts.sha256(pdf))
              .put("bytes", Integer.toString(pdf.length))
              .put("manifestSha256", Contracts.sha256(manifest))
              .put("manifestBytes", Integer.toString(manifest.length))
              .put("selectionSha256", "b".repeat(64));
          byte[] bytes = Contracts.JSON.writeValueAsBytes(result);
          exchange.getResponseHeaders().set("Content-Type", "application/json");
          exchange.sendResponseHeaders(200, bytes.length);
          exchange.getResponseBody().write(bytes);
          exchange.close();
        });
    server.createContext(
        "/internal/v1/cancel",
        exchange -> {
          cancellationCalls.incrementAndGet();
          if (malformedCancel.get()) {
            byte[] bytes = "{bad".getBytes(StandardCharsets.UTF_8);
            exchange.getResponseHeaders().set("Content-Type", "application/json");
            exchange.sendResponseHeaders(200, bytes.length);
            exchange.getResponseBody().write(bytes);
            exchange.close();
            return;
          }
          var request = Contracts.JSON.readTree(exchange.getRequestBody().readAllBytes());
          var result =
              Contracts.JSON.createObjectNode().put("schemaVersion", "1").put("cancelled", true);
          result.set("jobId", request.path("jobId"));
          byte[] bytes = Contracts.JSON.writeValueAsBytes(result);
          exchange.getResponseHeaders().set("Content-Type", "application/json");
          exchange.sendResponseHeaders(200, bytes.length);
          exchange.getResponseBody().write(bytes);
          exchange.close();
        });
    server.createContext(
        "/internal/v1/artifacts/",
        exchange -> {
          boolean isPdf = exchange.getRequestURI().getPath().endsWith("/pdf");
          byte[] all = isPdf ? pdf : manifest;
          String etag = "\"" + Contracts.sha256(all) + "\"";
          if (corrupt.get()) {
            all = all.clone();
            all[0] ^= 1;
          }
          var headers = exchange.getResponseHeaders();
          headers.set("Content-Type", isPdf ? "application/pdf" : "application/x-ndjson");
          headers.set("ETag", etag);
          String range = exchange.getRequestHeaders().getFirst("Range"),
              ifRange = exchange.getRequestHeaders().getFirst("If-Range");
          int status = 200, start = 0, end = all.length - 1;
          if (range != null && (ifRange == null || ifRange.equals(etag))) {
            try {
              long[] interval = Jobs.interval(range, all.length);
              start = (int) interval[0];
              end = (int) interval[1];
              status = 206;
              headers.set("Content-Range", "bytes " + start + "-" + end + "/" + all.length);
            } catch (java.io.IOException invalid) {
              headers.set("Content-Range", "bytes */" + all.length);
              exchange.sendResponseHeaders(416, -1);
              exchange.close();
              return;
            }
          }
          headers.set("Content-Length", Integer.toString(end - start + 1));
          exchange.sendResponseHeaders(
              status, exchange.getRequestMethod().equals("HEAD") ? -1 : end - start + 1);
          if (!exchange.getRequestMethod().equals("HEAD"))
            exchange.getResponseBody().write(all, start, end - start + 1);
          exchange.close();
        });
    server.start();
    try {
      var projects = new JdbcProjectStore(url, user, password);
      projects.ensureUser(owner);
      var project = projects.create(owner, "http-jobs");
      var snapshot =
          Contracts.JSON.readTree(
              getClass().getResourceAsStream("/fixtures/preview/export-pdf.json"));
      projects.append(owner, project.id(), 0, JobRequests.canonical(snapshot));
      var request =
          Contracts.JSON
              .createObjectNode()
              .put("schemaVersion", "2")
              .put("projectId", project.id().toString())
              .put("projectRevision", 1)
              .put("requestedAcceptedCount", 2)
              .put("maxCandidates", 4);
      var generation = (ObjectNode) snapshot.path("job").path("generation").deepCopy();
      generation.put("count", 1);
      request.set("generation", generation);
      request.set("pdf", snapshot.path("job").path("pdf"));
      request.putObject("execution").put("blockSize", 2).put("concurrency", 1);
      var config = new HashMap<String, Object>();
      config.put("server.port", "0");
      config.put("server.address", "127.0.0.1");
      config.put("mosaicforge.api-token", TOKEN);
      config.put("mosaicforge.engine-token", "b".repeat(64));
      config.put("mosaicforge.engine-url", "http://127.0.0.1:" + server.getAddress().getPort());
      config.put("mosaicforge.persistence.enabled", "true");
      config.put("mosaicforge.persistence.url", url);
      config.put("mosaicforge.persistence.user", user);
      config.put("mosaicforge.persistence.password", password);
      config.put("mosaicforge.persistence.owner-id", owner.toString());
      config.put("mosaicforge.jobs.enabled", "true");
      config.put("mosaicforge.jobs.valkey-uri", System.getenv("MOSAICFORGE_TEST_VALKEY_URI"));
      config.put("mosaicforge.jobs.valkey-password", System.getenv("MOSAICFORGE_VALKEY_PASSWORD"));
      config.put("logging.level.root", "ERROR");
      var app = new SpringApplication(Application.class);
      app.setDefaultProperties(config);
      try (var context =
          app.run(
              config.entrySet().stream()
                  .map(e -> "--" + e.getKey() + "=" + e.getValue())
                  .toArray(String[]::new))) {
        int port = ((WebServerApplicationContext) context).getWebServer().getPort();
        var client = HttpClient.newHttpClient();
        String base = "http://127.0.0.1:" + port;
        var submit =
            call(
                client,
                base,
                "/v2/jobs",
                "POST",
                JobRequests.canonical(request),
                Map.of("Idempotency-Key", KEY));
        assertEquals(202, submit.statusCode(), submit.body());
        var submitted = Contracts.JSON.readTree(submit.body());
        String path = "/v2/jobs/" + Contracts.text(submitted, "jobId");
        assertTrue(new Contracts().valid(submitted, "durable-job-status", "durable"));
        assertEquals(
            Contracts.text(submitted, "jobId"),
            Contracts.text(
                Contracts.JSON.readTree(
                    call(
                            client,
                            base,
                            "/v2/jobs",
                            "POST",
                            JobRequests.canonical(request),
                            Map.of("Idempotency-Key", KEY))
                        .body()),
                "jobId"));
        request.put("requestedAcceptedCount", 1);
        assertEquals(
            409,
            call(
                    client,
                    base,
                    "/v2/jobs",
                    "POST",
                    JobRequests.canonical(request),
                    Map.of("Idempotency-Key", KEY))
                .statusCode());
        request.put("requestedAcceptedCount", 2);
        assertEquals(
            400,
            call(
                    client,
                    base,
                    "/v2/jobs",
                    "POST",
                    JobRequests.canonical(request),
                    Map.of("Idempotency-Key", "short"))
                .statusCode());
        assertEquals(
            404,
            call(client, base, "/v2/jobs/" + UUID.randomUUID(), "GET", null, Map.of())
                .statusCode());
        assertEquals(
            401,
            client
                .send(
                    HttpRequest.newBuilder(URI.create(base + path)).GET().build(),
                    HttpResponse.BodyHandlers.ofString())
                .statusCode());
        for (int n = 0; n < 100; n++) {
          var status =
              Contracts.JSON.readTree(call(client, base, path, "GET", null, Map.of()).body());
          if (Contracts.text(status, "state").equals("succeeded")) break;
          Thread.sleep(100);
        }
        assertEquals(
            "succeeded",
            Contracts.text(
                Contracts.JSON.readTree(call(client, base, path, "GET", null, Map.of()).body()),
                "state"));
        var download = call(client, base, path + "/artifacts/pdf", "GET", null, Map.of());
        assertEquals(200, download.statusCode(), download.body());
        assertEquals(new String(pdf, StandardCharsets.UTF_8), download.body());
        var range =
            call(client, base, path + "/artifacts/pdf", "GET", null, Map.of("Range", "bytes=2-7"));
        assertEquals(206, range.statusCode());
        assertEquals(new String(pdf, 2, 6, StandardCharsets.UTF_8), range.body());
        assertEquals(
            "bytes 2-7/" + pdf.length, range.headers().firstValue("Content-Range").orElseThrow());
        assertEquals(
            200,
            call(
                    client,
                    base,
                    path + "/artifacts/pdf",
                    "GET",
                    null,
                    Map.of("Range", "bytes=2-7", "If-Range", "\"stale\""))
                .statusCode());
        assertEquals(
            416,
            call(
                    client,
                    base,
                    path + "/artifacts/pdf",
                    "GET",
                    null,
                    Map.of("Range", "bytes=999999-"))
                .statusCode());
        assertEquals(
            416,
            call(
                    client,
                    base,
                    path + "/artifacts/pdf",
                    "GET",
                    null,
                    Map.of("Range", "bytes=0-1,3-4"))
                .statusCode());
        var head = call(client, base, path + "/artifacts/pdf", "HEAD", null, Map.of());
        assertEquals(200, head.statusCode());
        assertTrue(head.body().isEmpty());
        assertEquals(
            Integer.toString(pdf.length),
            head.headers().firstValue("Content-Length").orElseThrow());
        assertEquals(
            200,
            call(client, base, path + "/artifacts/manifest", "GET", null, Map.of()).statusCode());
        corrupt.set(true);
        var corrupted = call(client, base, path + "/artifacts/pdf", "GET", null, Map.of());
        assertEquals(502, corrupted.statusCode());
        assertTrue(
            new Contracts()
                .valid(Contracts.JSON.readTree(corrupted.body()), "projects-error", "projects"));
        assertTrue(corrupted.headers().firstValue("ETag").isEmpty());
        corrupt.set(false);
        int cancellationBefore = cancellationCalls.get();
        assertEquals(
            "succeeded",
            Contracts.text(
                Contracts.JSON.readTree(
                    call(client, base, path + "/cancel", "POST", null, Map.of()).body()),
                "state"));
        assertEquals(cancellationBefore, cancellationCalls.get());
        busy.set(true);
        var pending =
            call(
                client,
                base,
                "/v2/jobs",
                "POST",
                JobRequests.canonical(request),
                Map.of("Idempotency-Key", KEY + "-cancel"));
        String cancelledPath =
            "/v2/jobs/" + Contracts.text(Contracts.JSON.readTree(pending.body()), "jobId");
        malformedCancel.set(true);
        assertEquals(
            "cancelled",
            Contracts.text(
                Contracts.JSON.readTree(
                    call(client, base, cancelledPath + "/cancel", "POST", null, Map.of()).body()),
                "state"));
        assertEquals(
            "cancelled",
            Contracts.text(
                Contracts.JSON.readTree(
                    call(client, base, cancelledPath + "/cancel", "POST", null, Map.of()).body()),
                "state"));
        malformedCancel.set(false);
        var cancelledId = UUID.fromString(cancelledPath.substring("/v2/jobs/".length()));
        context.getBean(JobRunner.class).cancel(cancelledId);
        assertFalse(context.getBean(JobStore.class).cancelled(100).contains(cancelledId));
        assertEquals(
            404,
            call(client, base, cancelledPath + "/artifacts/pdf", "GET", null, Map.of())
                .statusCode());
      }
    } finally {
      server.stop(0);
      try (var c = DriverManager.getConnection(url, user, password)) {
        for (String sql :
            new String[] {
              "DELETE FROM mosaicforge_job_outbox WHERE job_id IN (SELECT id FROM mosaicforge_jobs WHERE owner_id=?)",
              "DELETE FROM mosaicforge_job_blocks WHERE job_id IN (SELECT id FROM mosaicforge_jobs WHERE owner_id=?)",
              "DELETE FROM mosaicforge_jobs WHERE owner_id=?",
              "DELETE FROM mosaicforge_project_revisions WHERE project_id IN (SELECT id FROM mosaicforge_projects WHERE owner_id=?)",
              "DELETE FROM mosaicforge_projects WHERE owner_id=?",
              "DELETE FROM mosaicforge_users WHERE id=?"
            })
          try (var s = c.prepareStatement(sql)) {
            s.setObject(1, owner);
            s.executeUpdate();
          }
      }
    }
  }

  private static HttpResponse<String> call(
      HttpClient client,
      String base,
      String path,
      String method,
      String body,
      Map<String, String> headers)
      throws Exception {
    var builder =
        HttpRequest.newBuilder(URI.create(base + path))
            .header("Authorization", "Bearer " + TOKEN)
            .header("Content-Type", "application/json");
    headers.forEach(builder::header);
    builder.method(
        method,
        body == null
            ? HttpRequest.BodyPublishers.noBody()
            : HttpRequest.BodyPublishers.ofString(body));
    return client.send(builder.build(), HttpResponse.BodyHandlers.ofString());
  }
}
