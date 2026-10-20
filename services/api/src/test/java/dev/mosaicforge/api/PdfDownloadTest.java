package dev.mosaicforge.api;

import static org.junit.jupiter.api.Assertions.*;

import com.sun.net.httpserver.HttpServer;
import java.net.InetSocketAddress;
import java.net.URI;
import java.net.http.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.Map;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.web.server.context.WebServerApplicationContext;
import tools.jackson.databind.node.ObjectNode;

class PdfDownloadTest {
  private static final String TOKEN = "a".repeat(64);

  private static ObjectNode fixture(String name) throws Exception {
    try (var stream = PdfDownloadTest.class.getResourceAsStream("/fixtures/preview/" + name)) {
      return (ObjectNode) Contracts.JSON.readTree(stream);
    }
  }

  @Test
  void interruptedDownloadCancelsReceptionAndDeletesPartialFile() throws Exception {
    var entered = new CountDownLatch(1);
    var release = new CountDownLatch(1);
    var failure = new AtomicReference<Throwable>();
    var server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
    server.createContext(
        "/v3/export-pdf-selection",
        exchange -> {
          exchange.getRequestBody().readAllBytes();
          exchange.sendResponseHeaders(200, 1000000);
          exchange.getResponseBody().write("%PDF-1.7".getBytes(StandardCharsets.US_ASCII));
          exchange.getResponseBody().flush();
          entered.countDown();
          try {
            release.await(10, TimeUnit.SECONDS);
          } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
          } finally {
            exchange.close();
          }
        });
    server.start();
    var engine = new HttpEngine("http://127.0.0.1:" + server.getAddress().getPort(), TOKEN, 1000);
    var task =
        new Thread(
            () -> {
              try (var download =
                  engine.download(
                      "/v3/export-pdf-selection",
                      new byte[0],
                      "00-1234567890abcdef1234567890abcdef-1234567890abcdef-01")) {
                failure.set(
                    new AssertionError(
                        "Interrupted download unexpectedly completed: " + download.status()));
              } catch (Throwable actual) {
                failure.set(actual);
              }
            });
    try {
      task.start();
      assertTrue(entered.await(10, TimeUnit.SECONDS));
      task.interrupt();
      task.join(10000);
      assertFalse(task.isAlive());
      assertInstanceOf(InterruptedException.class, failure.get());
      try (var files = Files.list(java.nio.file.Path.of(System.getProperty("java.io.tmpdir")))) {
        assertFalse(
            files.anyMatch(file -> file.getFileName().toString().startsWith("mosaicforge-pdf-")));
      }
    } finally {
      task.interrupt();
      release.countDown();
      server.stop(0);
    }
  }

  @Test
  void legacyPortRemainsFunctionalAndBinarySupportIsExplicit() throws Exception {
    EnginePort legacy =
        (path, request, trace, pdf) -> new EnginePort.Reply(200, "application/json", request);
    assertArrayEquals(
        new byte[] {1}, legacy.execute("/v1/inspect", new byte[] {1}, "trace", false).body());
    assertThrows(
        java.io.IOException.class,
        () -> legacy.download("/v3/export-pdf-selection", new byte[0], "trace"));
  }

  @Test
  void verifiesLargeBinaryBeforePublishingAndCleansEveryTemporaryFile() throws Exception {
    byte[] pdf = new byte[17 * 1024 * 1024];
    System.arraycopy("%PDF-1.7".getBytes(StandardCharsets.US_ASCII), 0, pdf, 0, 8);
    var input = fixture("export-pdf.json");
    input.put("schemaVersion", "2");
    input.putArray("variantIndices").add("0").add("2").add("5");
    var metadata = fixture("accepted-pdf.json");
    metadata.remove("artifact");
    metadata.put("schemaVersion", "3");
    metadata.set("variantIndices", input.path("variantIndices"));
    ((ObjectNode) metadata.path("document")).put("documentHash", Contracts.sha256(pdf));
    var mode = new AtomicInteger();
    var server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
    server.createContext(
        "/v3/export-pdf-selection",
        exchange -> {
          exchange.getRequestBody().readAllBytes();
          assertEquals("Bearer " + TOKEN, exchange.getRequestHeaders().getFirst("Authorization"));
          var result = metadata.deepCopy();
          if (mode.get() == 1)
            ((ObjectNode) result.path("document")).put("documentHash", "0".repeat(64));
          if (mode.get() == 2) ((ObjectNode) result.path("document")).put("pageCount", 2);
          if (mode.get() == 3) result.put("requestId", "00000000-0000-0000-0000-000000000000");
          if (mode.get() == 4) result.putArray("variantIndices").add("0").add("1").add("5");
          if (mode.get() == 6) result.putArray("variantIndices").add("0").add("5").add("2");
          if (mode.get() == 7) ((ObjectNode) result.path("document")).put("seed", "1");
          if (mode.get() == 8) ((ObjectNode) result.path("document")).putNull("documentHash");
          if (mode.get() == 9) {
            exchange.getResponseHeaders().set("Content-Type", "application/json");
            exchange.sendResponseHeaders(500, 4);
            exchange.getResponseBody().write("null".getBytes(StandardCharsets.US_ASCII));
            exchange.close();
            return;
          }
          exchange.getResponseHeaders().set("Content-Type", "application/pdf");
          exchange
              .getResponseHeaders()
              .set("x-mosaicforge-result", Contracts.JSON.writeValueAsString(result));
          exchange.sendResponseHeaders(
              200, mode.get() == 10 ? 1073741825L : pdf.length + (mode.get() == 5 ? 1 : 0));
          try {
            exchange.getResponseBody().write(pdf);
          } finally {
            exchange.close();
          }
        });
    server.start();
    var app = new SpringApplication(Application.class);
    app.setDefaultProperties(Map.of("spring.main.banner-mode", "off"));
    var temp = java.nio.file.Path.of(System.getProperty("java.io.tmpdir"));
    try (var context =
        app.run(
            "--server.port=0",
            "--mosaicforge.api-token=" + TOKEN,
            "--mosaicforge.engine-token=" + TOKEN,
            "--mosaicforge.engine-url=http://127.0.0.1:" + server.getAddress().getPort())) {
      int port = ((WebServerApplicationContext) context).getWebServer().getPort();
      var client = HttpClient.newHttpClient();
      for (int current = 0; current <= 10; current++) {
        mode.set(current);
        var request =
            HttpRequest.newBuilder(
                    URI.create("http://127.0.0.1:" + port + "/v3/export-pdf-selection"))
                .header("Authorization", "Bearer " + TOKEN)
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString(Contracts.JSON.writeValueAsString(input)))
                .build();
        var response = client.send(request, HttpResponse.BodyHandlers.ofByteArray());
        assertEquals(current == 0 ? 200 : 502, response.statusCode(), "mode=" + current);
        if (current == 0) {
          assertArrayEquals(pdf, response.body());
          assertEquals(
              "application/pdf", response.headers().firstValue("content-type").orElseThrow());
          assertEquals(
              (long) pdf.length,
              response.headers().firstValueAsLong("content-length").orElseThrow());
        }
        try (var files = Files.list(temp)) {
          assertFalse(
              files.anyMatch(file -> file.getFileName().toString().startsWith("mosaicforge-pdf-")));
        }
      }
    } finally {
      server.stop(0);
    }
  }
}
