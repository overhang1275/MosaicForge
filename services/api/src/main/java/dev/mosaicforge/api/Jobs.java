package dev.mosaicforge.api;

import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import java.io.IOException;
import java.security.MessageDigest;
import java.util.HexFormat;
import java.util.UUID;
import java.util.concurrent.Executors;
import java.util.concurrent.Semaphore;
import java.util.concurrent.TimeUnit;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.core.env.Environment;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestMethod;
import org.springframework.web.bind.annotation.RestController;
import tools.jackson.databind.JsonNode;

@RestController
@ConditionalOnProperty(name = "mosaicforge.jobs.enabled", havingValue = "true")
class Jobs {
  private static final org.slf4j.Logger LOG = org.slf4j.LoggerFactory.getLogger(Jobs.class);
  private final JobStore store;
  private final Contracts contracts;
  private final EnginePort engine;
  private final JobRunner runner;
  private final UUID owner;
  private final Semaphore uploads = new Semaphore(2), downloads = new Semaphore(2);

  Jobs(JobStore store, Contracts contracts, EnginePort engine, JobRunner runner, Environment env) {
    this.store = store;
    this.contracts = contracts;
    this.engine = engine;
    this.runner = runner;
    owner = UUID.fromString(env.getRequiredProperty("mosaicforge.persistence.owner-id"));
  }

  @RequestMapping(
      value = {"/v2/jobs", "/v2/jobs/{id}/cancel"},
      method = RequestMethod.POST)
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

  @RequestMapping(
      value = {"/v2/jobs/{id}", "/v2/jobs/{id}/artifacts/{kind}"},
      method = {RequestMethod.GET, RequestMethod.HEAD})
  void get(HttpServletRequest req, HttpServletResponse res) throws IOException {
    handle(req, res, false);
  }

  private static UUID uuid(String value) {
    if (!value.matches(
        "[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"))
      throw new IllegalArgumentException();
    return UUID.fromString(value);
  }

  private void handle(HttpServletRequest req, HttpServletResponse res, boolean write)
      throws IOException {
    String requestId = UUID.randomUUID().toString();
    long started = System.nanoTime();
    try {
      if (req.getQueryString() != null) throw new IllegalArgumentException();
      var parts = req.getRequestURI().split("/");
      JobStore.Job job;
      if (write && parts.length == 3) {
        String key = req.getHeader("Idempotency-Key");
        if (key == null
            || !key.matches("[!-~]{16,128}")
            || java.util.Collections.list(req.getHeaders("Idempotency-Key")).size() != 1)
          throw new IllegalArgumentException();
        if (req.getContentType() == null
            || !req.getContentType().split(";", 2)[0].trim().equalsIgnoreCase("application/json")) {
          Gateway.failure(res, 415, requestId, "unsupported_media_type");
          return;
        }
        int limit = 2 * 1024 * 1024;
        if (req.getContentLengthLong() > limit) {
          Gateway.failure(res, 413, requestId, "payload_too_large");
          return;
        }
        byte[] bytes = ProjectBody.read(req, limit);
        if (bytes.length > limit) {
          Gateway.failure(res, 413, requestId, "payload_too_large");
          return;
        }
        JsonNode body;
        try {
          body = Contracts.JSON.readTree(bytes);
        } catch (RuntimeException invalid) {
          throw new IllegalArgumentException();
        }
        if (body == null || !contracts.valid(body, "durable-job-request", "durable"))
          throw new IllegalArgumentException();
        job = store.submit(owner, key, JobRequests.canonical(body));
        res.setStatus(202);
        res.setHeader("Location", "/v2/jobs/" + job.id());
      } else {
        UUID id = uuid(parts[3]);
        job = store.find(owner, id).orElseThrow(ProjectStore.Missing::new);
        if (parts.length == 6) {
          if (write) throw new IllegalArgumentException();
          artifact(req, res, job, parts[5], requestId);
          return;
        }
        if (write) {
          if (parts.length != 5 || !parts[4].equals("cancel")) throw new IllegalArgumentException();
          if (req.getContentLengthLong() > 0 || req.getHeader("Transfer-Encoding") != null)
            throw new IllegalArgumentException();
          store.cancel(owner, id);
          job = store.find(owner, id).orElseThrow(ProjectStore.Missing::new);
          if (job.state().equals("cancelled")) runner.cancel(id);
        }
        res.setStatus(200);
      }
      var output = status(job);
      if (!contracts.valid(output, "durable-job-status", "durable"))
        throw new IllegalStateException("Invalid job status");
      res.setContentType("application/json");
      if (!req.getMethod().equals("HEAD"))
        res.getOutputStream().write(Contracts.JSON.writeValueAsBytes(output));
    } catch (ProjectStore.Missing missing) {
      Gateway.failure(res, 404, requestId, "not_found");
    } catch (ProjectStore.Conflict conflict) {
      Gateway.failure(res, 409, requestId, "conflict");
    } catch (IllegalArgumentException invalid) {
      Gateway.failure(res, 400, requestId, "invalid_request");
    } catch (java.util.concurrent.TimeoutException timeout) {
      Gateway.failure(res, 504, requestId, "timeout");
    } catch (InterruptedException interrupted) {
      Thread.currentThread().interrupt();
      Gateway.failure(res, 502, requestId, "engine_unavailable");
    } catch (IllegalStateException unavailable) {
      Gateway.failure(res, 503, requestId, "unavailable");
    } catch (IOException unavailable) {
      if (!res.isCommitted()) {
        res.resetBuffer();
        for (var header :
            new String[] {
              "Content-Length", "Content-Range", "ETag", "Accept-Ranges", "Content-Disposition"
            }) res.setHeader(header, null);
        Gateway.failure(res, 502, requestId, "engine_unavailable");
      } else throw unavailable;
    } finally {
      LOG.atInfo()
          .addKeyValue("requestId", requestId)
          .addKeyValue("traceparent", res.getHeader("traceparent"))
          .addKeyValue("operation", req.getRequestURI())
          .addKeyValue("status", res.getStatus())
          .addKeyValue("durationMs", (System.nanoTime() - started) / 1000000)
          .log("control.job");
    }
  }

  JsonNode status(JobStore.Job job) {
    var request = Contracts.JSON.readTree(job.request());
    var body = Contracts.JSON.createObjectNode();
    body.put("schemaVersion", "2").put("jobId", job.id().toString());
    body.set("projectId", request.path("projectId"));
    body.put("state", job.state().equals("finalizing") ? "running" : job.state())
        .put("revision", job.revision());
    body.set("requestedAcceptedCount", request.path("requestedAcceptedCount"));
    body.set("maxCandidates", request.path("maxCandidates"));
    body.put("evaluatedCount", job.evaluated())
        .put("acceptedCount", job.accepted())
        .put("rejectedCount", job.rejected());
    if (job.nextIndex() == null) body.putNull("nextCandidateIndex");
    else body.put("nextCandidateIndex", job.nextIndex());
    body.putNull("artifact");
    if (job.state().equals("succeeded")) {
      var artifact =
          store
              .artifact(owner, job.id())
              .orElseThrow(() -> new IllegalStateException("Missing sealed artifact"));
      body.putObject("artifact")
          .put("pdfId", artifact.id().toString())
          .put("manifestId", artifact.id().toString())
          .put("pageCount", artifact.pages())
          .put("sha256", artifact.sha256())
          .put("bytes", Long.toString(artifact.bytes()));
    }
    return body;
  }

  private void artifact(
      HttpServletRequest req,
      HttpServletResponse res,
      JobStore.Job job,
      String kind,
      String requestId)
      throws IOException, InterruptedException, java.util.concurrent.TimeoutException {
    if (!kind.equals("pdf") && !kind.equals("manifest")) throw new ProjectStore.Missing();
    var artifact = store.artifact(owner, job.id()).orElseThrow(ProjectStore.Missing::new);
    long total = kind.equals("pdf") ? artifact.bytes() : artifact.manifestBytes();
    String hash = kind.equals("pdf") ? artifact.sha256() : artifact.manifestSha256(),
        etag = "\"" + hash + "\"",
        type = kind.equals("pdf") ? "application/pdf" : "application/x-ndjson";
    String range = req.getHeader("Range"), ifRange = req.getHeader("If-Range");
    if (range != null
        && (range.length() > 128 || !range.matches("bytes=(?:[0-9]+-[0-9]*|-[0-9]+)"))) {
      res.setHeader("Content-Range", "bytes */" + total);
      Gateway.failure(res, 416, requestId, "invalid_request");
      return;
    }
    if (ifRange != null && ifRange.length() > 256) throw new IllegalArgumentException();
    if (java.util.Collections.list(req.getHeaders("Range")).size() > 1
        || java.util.Collections.list(req.getHeaders("If-Range")).size() > 1)
      throw new IllegalArgumentException();
    if (!downloads.tryAcquire()) {
      Gateway.failure(res, 429, requestId, "busy");
      return;
    }
    try (var stream =
        engine.stream(
            "/internal/v1/artifacts/" + artifact.id() + "/" + kind,
            req.getMethod(),
            range,
            ifRange,
            res.getHeader("traceparent"))) {
      if (stream.status() == 416) {
        if (!stream.contentRange().equals("bytes */" + total))
          throw new IOException("Invalid unsatisfiable range metadata");
        res.setHeader("Content-Range", "bytes */" + total);
        Gateway.failure(res, 416, requestId, "invalid_request");
        return;
      }
      if ((stream.status() != 200 && stream.status() != 206)
          || !stream.etag().equals(etag)
          || !stream.contentType().split(";", 2)[0].trim().equals(type))
        throw new IOException("Invalid artifact response");
      long expected = total;
      if (stream.status() == 200 && range != null && (ifRange == null || ifRange.equals(etag)))
        throw new IOException("Ignored artifact range");
      if (stream.status() == 206) {
        if (range == null || ifRange != null && !ifRange.equals(etag))
          throw new IOException("Unexpected artifact range");
        long[] interval = interval(range, total);
        expected = interval[1] - interval[0] + 1;
        String contentRange = "bytes " + interval[0] + "-" + interval[1] + "/" + total;
        if (!stream.contentRange().equals(contentRange))
          throw new IOException("Invalid artifact range");
        res.setHeader("Content-Range", contentRange);
      } else if (!stream.contentRange().isEmpty())
        throw new IOException("Unexpected artifact range");
      if (stream.length() != expected || expected <= 0)
        throw new IOException("Invalid artifact length");
      res.setStatus(stream.status());
      res.setContentType(type);
      res.setContentLengthLong(expected);
      res.setHeader("ETag", etag);
      res.setHeader("Accept-Ranges", "bytes");
      res.setHeader(
          "Content-Disposition",
          "attachment; filename=\"mosaicforge-"
              + job.id()
              + "."
              + (kind.equals("pdf") ? "pdf" : "ndjson")
              + "\"");
      if (req.getMethod().equals("HEAD")) return;
      try (var timeout = Executors.newSingleThreadScheduledExecutor()) {
        var alarm =
            timeout.schedule(
                () -> {
                  try {
                    stream.body().close();
                  } catch (IOException ignored) {
                    /* Deadline closes the body. */
                  }
                },
                120,
                TimeUnit.SECONDS);
        try {
          var digest = sha256();
          byte[] buffer = new byte[8192];
          long received = 0;
          int held = -1;
          for (int count; (count = stream.body().read(buffer)) != -1; ) {
            if (count == 0) continue;
            if (received + count > expected) throw new IOException("Artifact exceeds metadata");
            digest.update(buffer, 0, count);
            received += count;
            if (held >= 0) res.getOutputStream().write(held);
            res.getOutputStream().write(buffer, 0, count - 1);
            held = Byte.toUnsignedInt(buffer[count - 1]);
          }
          if (received != expected
              || stream.status() == 200 && !HexFormat.of().formatHex(digest.digest()).equals(hash))
            throw new IOException("Artifact integrity mismatch");
          if (held >= 0) res.getOutputStream().write(held);
        } finally {
          alarm.cancel(false);
          timeout.shutdownNow();
        }
      }
    } finally {
      downloads.release();
    }
  }

  static long[] interval(String range, long total) throws IOException {
    try {
      String[] values = range.substring(6).split("-", -1);
      long start, end;
      if (values[0].isEmpty()) {
        long suffix = Long.parseLong(values[1]);
        if (suffix < 1) throw new IOException("Invalid suffix");
        start = Math.max(0, total - suffix);
        end = total - 1;
      } else {
        start = Long.parseLong(values[0]);
        end = values[1].isEmpty() ? total - 1 : Math.min(total - 1, Long.parseLong(values[1]));
      }
      if (start < 0 || start >= total || end < start) throw new IOException("Unsatisfiable range");
      return new long[] {start, end};
    } catch (NumberFormatException invalid) {
      throw new IOException("Invalid range", invalid);
    }
  }

  private static MessageDigest sha256() {
    try {
      return MessageDigest.getInstance("SHA-256");
    } catch (java.security.NoSuchAlgorithmException impossible) {
      throw new IllegalStateException(impossible);
    }
  }
}
