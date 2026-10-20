package dev.mosaicforge.api;

import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import java.io.IOException;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.Semaphore;
import java.util.concurrent.TimeoutException;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.core.env.Environment;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RestController;
import tools.jackson.databind.JsonNode;

@RestController
@ConditionalOnProperty(name = "mosaicforge.persistence.enabled", havingValue = "true")
class Projects {
  private static final org.slf4j.Logger LOG = org.slf4j.LoggerFactory.getLogger(Projects.class);
  private static final int MAX_BODY = 32 * 1024 * 1024;
  private final ProjectStore store;
  private final Contracts contracts;
  private final EnginePort engine;
  private final UUID owner;
  private final Semaphore uploads = new Semaphore(2);

  Projects(ProjectStore store, Contracts contracts, EnginePort engine, Environment env) {
    this.store = store;
    this.contracts = contracts;
    this.engine = engine;
    owner = uuid(env.getRequiredProperty("mosaicforge.persistence.owner-id"));
    store.ensureUser(owner);
  }

  private static UUID uuid(String value) {
    if (value == null
        || !value.matches(
            "[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"))
      throw new IllegalArgumentException("Invalid UUID");
    return UUID.fromString(value);
  }

  private static Map<String, Object> project(ProjectStore.Project p) {
    return Map.of(
        "id",
        p.id().toString(),
        "name",
        p.name(),
        "currentRevision",
        p.currentRevision(),
        "createdAt",
        p.createdAt().toString(),
        "updatedAt",
        p.updatedAt().toString());
  }

  private static Map<String, Object> revision(ProjectStore.Revision r) {
    return Map.of(
        "projectId",
        r.projectId().toString(),
        "number",
        r.number(),
        "createdAt",
        r.createdAt().toString(),
        "snapshot",
        Contracts.JSON.readTree(r.snapshot()));
  }

  @GetMapping({
    "/v1/me",
    "/v1/projects",
    "/v1/projects/{id}",
    "/v1/projects/{id}/revisions/{number}"
  })
  void get(HttpServletRequest req, HttpServletResponse res) throws IOException {
    handle(req, res, false);
  }

  @PostMapping({"/v1/projects", "/v1/projects/{id}/revisions"})
  void post(HttpServletRequest req, HttpServletResponse res) throws IOException {
    if (!uploads.tryAcquire()) {
      Gateway.failure(res, 429, Gateway.UNKNOWN_ID, "busy");
      return;
    }
    var async = req.startAsync();
    async.setTimeout(0);
    async.start(
        () -> {
          try {
            handle(req, res, true);
          } catch (IOException disconnected) {
            /* Client disconnected. */
          } finally {
            uploads.release();
            async.complete();
          }
        });
  }

  private void handle(HttpServletRequest req, HttpServletResponse res, boolean write)
      throws IOException {
    String requestId = UUID.randomUUID().toString();
    long started = System.nanoTime();
    try {
      String path = req.getRequestURI();
      Object value;
      String schema;
      String key;
      if (write) {
        if (req.getQueryString() != null) throw new IllegalArgumentException();
        if (req.getContentType() == null
            || !req.getContentType().split(";", 2)[0].trim().equalsIgnoreCase("application/json")) {
          Gateway.failure(res, 415, requestId, "unsupported_media_type");
          return;
        }
        if (req.getContentLengthLong() > MAX_BODY) {
          Gateway.failure(res, 413, requestId, "payload_too_large");
          return;
        }
        byte[] bytes = ProjectBody.read(req, MAX_BODY);
        if (bytes.length > MAX_BODY) {
          Gateway.failure(res, 413, requestId, "payload_too_large");
          return;
        }
        JsonNode body;
        try {
          body = Contracts.JSON.readTree(bytes);
        } catch (RuntimeException invalid) {
          throw new IllegalArgumentException("Invalid JSON");
        }
        boolean create = path.equals("/v1/projects");
        if (body == null
            || !contracts.valid(
                body, "projects-" + (create ? "create-request" : "revision-request"), "projects"))
          throw new IllegalArgumentException();
        if (create) {
          value = project(store.create(owner, Contracts.text(body, "name")));
          key = "project";
          schema = "project-response";
        } else {
          UUID id = uuid(path.split("/")[3]);
          if (store.find(owner, id).isEmpty()) throw new ProjectStore.Missing();
          JsonNode snapshot = body.path("snapshot");
          if (!contracts.valid(snapshot, "selected-pdf-export-request", "export-pdf-selection"))
            throw new IllegalArgumentException();
          var inspect = Contracts.JSON.createObjectNode();
          inspect.set("requestId", snapshot.path("requestId"));
          inspect.set("templateSvg", snapshot.path("templateSvg"));
          var reply =
              engine.execute(
                  "/v1/inspect",
                  Contracts.JSON.writeValueAsBytes(inspect),
                  res.getHeader("traceparent"),
                  false);
          if (reply.status() != 200) {
            Gateway.failure(
                res,
                reply.status() == 400 ? 400 : 502,
                requestId,
                reply.status() == 400 ? "invalid_template" : "engine_unavailable");
            return;
          }
          JsonNode checked;
          try {
            checked = Contracts.JSON.readTree(reply.body());
          } catch (RuntimeException invalid) {
            Gateway.failure(res, 502, requestId, "engine_unavailable");
            return;
          }
          if (!reply.contentType().split(";", 2)[0].trim().equalsIgnoreCase("application/json")
              || !contracts.valid(checked, "inspect-response", "inspect")
              || !checked.path("requestId").equals(snapshot.path("requestId"))) {
            Gateway.failure(res, 502, requestId, "engine_unavailable");
            return;
          }
          if (!checked
              .path("templateHash")
              .equals(snapshot.path("job").path("generation").path("templateHash")))
            throw new IllegalArgumentException();
          value =
              revision(
                  store.append(
                      owner,
                      id,
                      body.path("expectedRevision").asInt(),
                      Contracts.JSON.writeValueAsString(snapshot)));
          key = "revision";
          schema = "revision-response";
        }
      } else if (path.equals("/v1/me")) {
        if (req.getQueryString() != null) throw new IllegalArgumentException();
        value = Map.of("id", owner.toString());
        key = "user";
        schema = "me-response";
      } else if (path.equals("/v1/projects")) {
        for (var entry : req.getParameterMap().entrySet())
          if (!entry.getKey().equals("limit") && !entry.getKey().equals("cursor")
              || entry.getValue().length != 1) throw new IllegalArgumentException();
        int limit =
            req.getParameter("limit") == null ? 20 : Integer.parseInt(req.getParameter("limit"));
        if (limit < 1 || limit > 100) throw new IllegalArgumentException();
        UUID cursor = req.getParameter("cursor") == null ? null : uuid(req.getParameter("cursor"));
        var rows = store.list(owner, cursor, limit + 1);
        boolean more = rows.size() > limit;
        var page = rows.subList(0, Math.min(limit, rows.size()));
        var output = Contracts.JSON.createObjectNode();
        output.put("schemaVersion", "1");
        output.put("requestId", requestId);
        output.set(
            "projects", Contracts.JSON.valueToTree(page.stream().map(Projects::project).toList()));
        if (more) output.put("nextCursor", page.getLast().id().toString());
        else output.putNull("nextCursor");
        respond(res, 200, output, "list-response");
        return;
      } else {
        if (req.getQueryString() != null) throw new IllegalArgumentException();
        var parts = path.split("/");
        UUID id = uuid(parts[3]);
        if (parts.length == 4) {
          value = project(store.find(owner, id).orElseThrow(ProjectStore.Missing::new));
          key = "project";
          schema = "project-response";
        } else {
          int number = Integer.parseInt(parts[5]);
          if (number < 1) throw new IllegalArgumentException();
          value =
              revision(store.revision(owner, id, number).orElseThrow(ProjectStore.Missing::new));
          key = "revision";
          schema = "revision-response";
        }
      }
      respond(
          res,
          write ? 201 : 200,
          Contracts.JSON.valueToTree(
              Map.of("schemaVersion", "1", "requestId", requestId, key, value)),
          schema);
    } catch (ProjectStore.Missing missing) {
      Gateway.failure(res, 404, requestId, "not_found");
    } catch (ProjectStore.Conflict conflict) {
      Gateway.failure(res, 409, requestId, "conflict");
    } catch (IllegalArgumentException invalid) {
      Gateway.failure(res, 400, requestId, "invalid_request");
    } catch (TimeoutException timeout) {
      Gateway.failure(res, 504, requestId, "timeout");
    } catch (InterruptedException interrupted) {
      Thread.currentThread().interrupt();
      Gateway.failure(res, 502, requestId, "engine_unavailable");
    } catch (IllegalStateException unavailable) {
      Gateway.failure(res, 503, requestId, "unavailable");
    } catch (IOException unavailable) {
      if (!res.isCommitted()) Gateway.failure(res, 502, requestId, "engine_unavailable");
    } finally {
      LOG.atInfo()
          .addKeyValue("requestId", requestId)
          .addKeyValue("traceparent", res.getHeader("traceparent"))
          .addKeyValue("operation", req.getRequestURI())
          .addKeyValue("status", res.getStatus())
          .addKeyValue("durationMs", (System.nanoTime() - started) / 1000000)
          .log("control.project");
    }
  }

  private void respond(HttpServletResponse response, int status, JsonNode body, String schema)
      throws IOException {
    if (!contracts.valid(body, "projects-" + schema, "projects"))
      throw new IllegalStateException("Invalid project response");
    response.setStatus(status);
    response.setContentType("application/json");
    response.getOutputStream().write(Contracts.JSON.writeValueAsBytes(body));
  }
}
