package dev.mosaicforge.api;

import dev.mosaicforge.contracts.PreviewOperation;
import jakarta.servlet.ReadListener;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.Semaphore;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.core.env.Environment;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RestController;
import tools.jackson.databind.JsonNode;

@RestController
class Gateway {
  static final String UNKNOWN_ID = "00000000-0000-0000-0000-000000000000";
  private static final Logger LOG = LoggerFactory.getLogger(Gateway.class);
  private static final int MAX_BODY = 32 * 1024 * 1024;
  private static final Map<String, Integer> ERROR_STATUS =
      Map.of(
          "invalid_request",
          400,
          "invalid_template",
          400,
          "unauthorized",
          401,
          "forbidden",
          403,
          "payload_too_large",
          413,
          "unsupported_media_type",
          415,
          "busy",
          429,
          "internal_error",
          500);
  private final Contracts contracts;
  private final EnginePort engine;
  private final long bodyTimeoutMs;
  private final Semaphore gate = new Semaphore(1);

  Gateway(Contracts contracts, EnginePort engine, Environment environment) {
    this.contracts = contracts;
    this.engine = engine;
    this.bodyTimeoutMs = environment.getProperty("mosaicforge.body-timeout-ms", Long.class, 30000L);
    if (bodyTimeoutMs < 1 || bodyTimeoutMs > 30000)
      throw new IllegalArgumentException("Invalid upload deadline");
  }

  @GetMapping("/health")
  Map<String, String> health() {
    return Map.of("status", "up");
  }

  @PostMapping("/**")
  void execute(HttpServletRequest request, HttpServletResponse response) throws IOException {
    var operation = PreviewOperation.forPath(request.getRequestURI());
    if (operation == null) {
      failure(response, 404, UNKNOWN_ID, "invalid_request");
      return;
    }
    if (request.getQueryString() != null) {
      failure(response, 400, UNKNOWN_ID, "invalid_request");
      return;
    }
    if (request.getContentType() == null
        || !request.getContentType().split(";", 2)[0].trim().equalsIgnoreCase("application/json")) {
      failure(response, 415, UNKNOWN_ID, "unsupported_media_type");
      return;
    }
    if (request.getContentLengthLong() > MAX_BODY) {
      failure(response, 413, UNKNOWN_ID, "payload_too_large");
      return;
    }
    if (!gate.tryAcquire()) {
      failure(response, 429, UNKNOWN_ID, "busy");
      return;
    }
    var async = request.startAsync();
    async.setTimeout(0); // The body and engine stages each have explicit bounded deadlines.
    async.start(
        () -> {
          try {
            executeAdmitted(request, response, operation);
          } catch (IOException disconnected) {
            LOG.debug("Control response connection closed");
          } finally {
            async.complete();
          }
        });
  }

  private void executeAdmitted(
      HttpServletRequest request, HttpServletResponse response, PreviewOperation operation)
      throws IOException {
    String id = UNKNOWN_ID, trace = response.getHeader("traceparent");
    long started = System.nanoTime();
    try {
      byte[] bytes;
      try {
        bytes = readBody(request);
      } catch (IOException invalid) {
        failure(response, 400, id, "invalid_request");
        return;
      }
      if (bytes.length > MAX_BODY) {
        failure(response, 413, id, "payload_too_large");
        return;
      }
      JsonNode input;
      try {
        input = Contracts.JSON.readTree(bytes);
      } catch (RuntimeException invalid) {
        failure(response, 400, id, "invalid_request");
        return;
      }
      if (input == null
          || !contracts.valid(input, operation.requestSchema, operationName(operation))) {
        failure(response, 400, id, "invalid_request");
        return;
      }
      id = Contracts.text(input, "requestId");
      var reply = engine.execute(operation.path, bytes, trace, operation.pdf);
      JsonNode output;
      try {
        output = Contracts.JSON.readTree(reply.body());
      } catch (RuntimeException invalid) {
        failure(response, 502, id, "engine_unavailable");
        return;
      }
      if (!reply.contentType().split(";", 2)[0].trim().equalsIgnoreCase("application/json")
          || output == null) {
        failure(response, 502, id, "engine_unavailable");
        return;
      }
      if (reply.status() != 200) {
        var code = Contracts.text(output, "code");
        var outputId = Contracts.text(output, "requestId");
        if (!contracts.valid(output, "error", operationName(operation))
            || ERROR_STATUS.getOrDefault(code, 0) != reply.status()
            || (!id.equals(outputId) && !UNKNOWN_ID.equals(outputId))) {
          failure(response, 502, id, "engine_unavailable");
          return;
        }
        failure(response, reply.status(), id, code);
        return;
      }
      if (!contracts.valid(output, operation.responseSchema, operationName(operation))
          || !correlates(input, output, operation)) {
        failure(response, 502, id, "engine_unavailable");
        return;
      }
      response.setStatus(200);
      response.setContentType("application/json");
      response.getOutputStream().write(reply.body());
    } catch (TimeoutException timeout) {
      failure(response, 504, id, "timeout");
    } catch (InterruptedException interrupted) {
      Thread.currentThread().interrupt();
      failure(response, 502, id, "engine_unavailable");
    } catch (IOException transport) {
      if (!response.isCommitted()) failure(response, 502, id, "engine_unavailable");
    } finally {
      gate.release();
      LOG.atInfo()
          .addKeyValue("requestId", id)
          .addKeyValue("traceparent", trace)
          .addKeyValue("operation", operation.path)
          .addKeyValue("status", response.getStatus())
          .addKeyValue("durationMs", (System.nanoTime() - started) / 1000000)
          .log("control.request");
    }
  }

  private byte[] readBody(HttpServletRequest request)
      throws IOException, InterruptedException, TimeoutException {
    var result = new CompletableFuture<byte[]>();
    var input = request.getInputStream();
    var output = new ByteArrayOutputStream();
    input.setReadListener(
        new ReadListener() {
          private final byte[] buffer = new byte[8192];

          @Override
          public void onDataAvailable() throws IOException {
            while (!result.isDone() && input.isReady() && !input.isFinished()) {
              int count =
                  input.read(buffer, 0, Math.min(buffer.length, MAX_BODY + 1 - output.size()));
              if (count < 0) break;
              output.write(buffer, 0, count);
              if (output.size() > MAX_BODY) {
                result.complete(output.toByteArray());
                return;
              }
            }
          }

          @Override
          public void onAllDataRead() {
            result.complete(output.toByteArray());
          }

          @Override
          public void onError(Throwable failure) {
            result.completeExceptionally(failure);
          }
        });
    try {
      return result.get(bodyTimeoutMs, TimeUnit.MILLISECONDS);
    } catch (ExecutionException failure) {
      throw new IOException("Invalid request body", failure);
    } finally {
      result.cancel(false);
    }
  }

  private static String operationName(PreviewOperation operation) {
    return operation.name().toLowerCase(java.util.Locale.ROOT).replace('_', '-');
  }

  private static boolean correlates(JsonNode input, JsonNode output, PreviewOperation operation) {
    if (!input.path("requestId").equals(output.path("requestId"))) return false;
    if (operation == PreviewOperation.INSPECT) return true;
    var spec = operation.pdf ? input.path("job").path("generation") : input.path("generation");
    var result = output.path(operation.pdf ? "document" : "variant");
    if (!spec.path("seed").equals(result.path("seed"))
        || !spec.path("templateHash").equals(result.path("templateHash"))
        || !spec.path("startIndex").equals(result.path(operation.pdf ? "startIndex" : "index")))
      return false;
    if (operation.pdf)
      return spec.path("count").equals(result.path("count"))
          && (operation != PreviewOperation.EXPORT_PDF_SELECTION
              || input.path("variantIndices").equals(output.path("variantIndices")));
    if (!spec.path("rulesVersion").equals(result.path("rulesVersion"))) return false;
    return output.path("artifact").isMissingNode()
        || output.path("artifact").isNull()
        || Contracts.text(output.path("artifact"), "mimeType")
            .equals(
                Contracts.text(spec.path("output"), "format").equals("png")
                    ? "image/png"
                    : "image/svg+xml");
  }

  static void failure(HttpServletResponse response, int status, String id, String code)
      throws IOException {
    response.setStatus(status);
    response.setContentType("application/json");
    response
        .getOutputStream()
        .write(
            Contracts.JSON.writeValueAsBytes(
                Map.of("requestId", id, "code", code, "message", code)));
  }
}
