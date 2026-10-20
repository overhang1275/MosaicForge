package dev.mosaicforge.api;

import static org.junit.jupiter.api.Assertions.*;

import com.sun.net.httpserver.HttpServer;
import java.net.InetSocketAddress;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.sql.DriverManager;
import java.util.HashMap;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.EnabledIfEnvironmentVariable;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.web.server.context.WebServerApplicationContext;
import tools.jackson.databind.node.ObjectNode;

@EnabledIfEnvironmentVariable(named = "MOSAICFORGE_TEST_DB_URL", matches = ".+")
class ProjectsHttpTest {
  @Test
  void authenticatedHttpValidatesSnapshotsOwnersContractsAndConflicts() throws Exception {
    var owner = UUID.randomUUID();
    String token = "a".repeat(64);
    var mode = new AtomicInteger();
    var server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
    server.createContext(
        "/v1/inspect",
        exchange -> {
          var request = Contracts.JSON.readTree(exchange.getRequestBody().readAllBytes());
          ObjectNode output;
          try (var source = getClass().getResourceAsStream("/fixtures/preview/metadata.json")) {
            output = (ObjectNode) Contracts.JSON.readTree(source.readAllBytes());
          }
          output.set("requestId", request.path("requestId"));
          output.put(
              "templateHash",
              mode.get() == 1
                  ? "0".repeat(64)
                  : "1cb2a436f15d755aeccbf3f7b1b720a3707a4325605bf95dd944d9727f8ebff7");
          var bytes = Contracts.JSON.writeValueAsBytes(output);
          exchange.getResponseHeaders().set("Content-Type", "application/json");
          exchange.sendResponseHeaders(200, bytes.length);
          exchange.getResponseBody().write(bytes);
          exchange.close();
        });
    server.start();
    String url = System.getenv("MOSAICFORGE_TEST_DB_URL"),
        user = System.getenv("MOSAICFORGE_DB_USER"),
        password = System.getenv("MOSAICFORGE_DB_PASSWORD");
    var config = new HashMap<String, Object>();
    config.put("server.port", "0");
    config.put("server.address", "127.0.0.1");
    config.put("mosaicforge.api-token", token);
    config.put("mosaicforge.engine-token", "b".repeat(64));
    config.put("mosaicforge.engine-url", "http://127.0.0.1:" + server.getAddress().getPort());
    config.put("mosaicforge.persistence.enabled", "true");
    config.put("mosaicforge.persistence.url", url);
    config.put("mosaicforge.persistence.user", user);
    config.put("mosaicforge.persistence.password", password);
    config.put("mosaicforge.persistence.owner-id", owner.toString());
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
      var contracts = new Contracts();
      java.util.function.BiFunction<String, String, HttpResponse<String>> call =
          (path, body) -> {
            try {
              var builder =
                  HttpRequest.newBuilder(URI.create("http://127.0.0.1:" + port + path))
                      .header("Authorization", "Bearer " + token)
                      .header("Content-Type", "application/json");
              return client.send(
                  body == null
                      ? builder.GET().build()
                      : builder.POST(HttpRequest.BodyPublishers.ofString(body)).build(),
                  HttpResponse.BodyHandlers.ofString());
            } catch (Exception failure) {
              throw new IllegalStateException(failure);
            }
          };
      var me = call.apply("/v1/me", null);
      assertEquals(200, me.statusCode());
      assertEquals(
          owner.toString(), Contracts.JSON.readTree(me.body()).path("user").path("id").asString());
      assertEquals(
          400,
          call.apply("/v1/projects", "{\"schemaVersion\":\"1\",\"name\":\"x\",\"owner\":\"spoof\"}")
              .statusCode());
      assertEquals(400, call.apply("/v1/projects", "{bad").statusCode());
      assertEquals(
          400,
          call.apply("/v1/projects", "{\"schemaVersion\":\"1\",\"name\":\"   \"}").statusCode());
      var created =
          call.apply("/v1/projects", "{\"schemaVersion\":\"1\",\"name\":\"Saved studio\"}");
      assertEquals(201, created.statusCode(), created.body());
      assertTrue(
          contracts.valid(
              Contracts.JSON.readTree(created.body()), "projects-project-response", "projects"));
      var id = Contracts.JSON.readTree(created.body()).path("project").path("id").asString();
      assertEquals(400, call.apply("/v1/projects/not-a-uuid", null).statusCode());
      assertEquals(400, call.apply("/v1/projects?limit=101", null).statusCode());
      assertEquals(400, call.apply("/v1/projects?owner=spoof", null).statusCode());
      ObjectNode snapshot;
      try (var source = getClass().getResourceAsStream("/fixtures/preview/export-pdf.json")) {
        snapshot = (ObjectNode) Contracts.JSON.readTree(source.readAllBytes());
      }
      snapshot.put("schemaVersion", "2");
      snapshot.set(
          "variantIndices", Contracts.JSON.readTree("[\"0\",\"2\",\"18446744073709551615\"]"));
      var body = Contracts.JSON.createObjectNode();
      body.put("schemaVersion", "1");
      body.put("expectedRevision", 0);
      body.set("snapshot", snapshot);
      var path = "/v1/projects/" + id + "/revisions";
      mode.set(1);
      assertEquals(400, call.apply(path, Contracts.JSON.writeValueAsString(body)).statusCode());
      mode.set(0);
      var saved = call.apply(path, Contracts.JSON.writeValueAsString(body));
      assertEquals(201, saved.statusCode(), saved.body());
      assertTrue(
          contracts.valid(
              Contracts.JSON.readTree(saved.body()), "projects-revision-response", "projects"));
      assertEquals(
          snapshot,
          Contracts.JSON
              .readTree(call.apply(path + "/1", null).body())
              .path("revision")
              .path("snapshot"));
      assertEquals(409, call.apply(path, Contracts.JSON.writeValueAsString(body)).statusCode());
      assertEquals(
          1,
          Contracts.JSON
              .readTree(call.apply("/v1/projects/" + id, null).body())
              .path("project")
              .path("currentRevision")
              .asInt());
      assertEquals(200, call.apply("/v1/projects?limit=1", null).statusCode());
      assertEquals(404, call.apply("/v1/projects/" + UUID.randomUUID(), null).statusCode());
    } finally {
      server.stop(0);
      try (var c = DriverManager.getConnection(url, user, password)) {
        for (String sql :
            new String[] {
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
}
