package dev.mosaicforge.api;

import static org.junit.jupiter.api.Assertions.*;

import com.sun.net.httpserver.HttpServer;
import java.net.InetSocketAddress;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.util.Map;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.web.server.context.WebServerApplicationContext;

class GatewayTest {
  private static final String TOKEN = "a".repeat(64), ENGINE_TOKEN = "b".repeat(64);
  private static final String ID = "12345678-1234-1234-1234-123456789abc";
  private static final String TRACE = "00-1234567890abcdef1234567890abcdef-1234567890abcdef-01";
  private final HttpClient client = HttpClient.newHttpClient();

  private static byte[] fixture(String name) throws Exception {
    try (var input = GatewayTest.class.getResourceAsStream("/fixtures/preview/" + name)) {
      return input.readAllBytes();
    }
  }

  private HttpRequest request(int port, String token, String body, String path) {
    var builder =
        HttpRequest.newBuilder(URI.create("http://127.0.0.1:" + port + path))
            .header("Content-Type", "application/json")
            .header("traceparent", TRACE);
    if (token != null) builder.header("Authorization", "Bearer " + token);
    return builder.POST(HttpRequest.BodyPublishers.ofString(body)).build();
  }

  @Test
  void gatewayAuthenticatesValidatesBoundsCorrelatesAndPreservesTrace() throws Exception {
    var mode = new AtomicInteger();
    var observed = new AtomicReference<String>();
    var entered = new CountDownLatch(1);
    var release = new CountDownLatch(1);
    var server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
    server.createContext(
        "/v1/inspect",
        exchange -> {
          observed.set(exchange.getRequestHeaders().getFirst("traceparent"));
          assertEquals(
              "Bearer " + ENGINE_TOKEN, exchange.getRequestHeaders().getFirst("Authorization"));
          exchange.getRequestBody().readAllBytes();
          try {
            if (mode.get() == 4) {
              entered.countDown();
              release.await(15, TimeUnit.SECONDS);
            }
            if (mode.get() == 5) Thread.sleep(6000);
            byte[] body = fixture("metadata.json");
            if (mode.get() == 1) body = "{}".getBytes(StandardCharsets.UTF_8);
            if (mode.get() == 2)
              body =
                  new String(body, StandardCharsets.UTF_8)
                      .replace(ID, "0".repeat(36))
                      .getBytes(StandardCharsets.UTF_8);
            if (mode.get() == 3) body = new byte[16 * 1024 * 1024 + 1];
            exchange.getResponseHeaders().set("Content-Type", "application/json");
            exchange.sendResponseHeaders(200, body.length);
            exchange.getResponseBody().write(body);
          } catch (Exception ignored) {
            /* Deliberate timeout/closed connection in robustness tests. */
          } finally {
            exchange.close();
          }
        });
    server.start();
    var app = new SpringApplication(Application.class);
    app.setDefaultProperties(
        Map.of(
            "server.port",
            "0",
            "server.address",
            "127.0.0.1",
            "mosaicforge.api-token",
            TOKEN,
            "mosaicforge.engine-token",
            ENGINE_TOKEN,
            "mosaicforge.engine-url",
            "http://127.0.0.1:" + server.getAddress().getPort(),
            "mosaicforge.timeout-ms",
            "5000",
            "spring.main.banner-mode",
            "off"));
    try (var context =
        app.run(
            "--mosaicforge.body-timeout-ms=1000",
            "--server.port=0",
            "--mosaicforge.api-token=" + TOKEN,
            "--mosaicforge.engine-token=" + ENGINE_TOKEN,
            "--mosaicforge.engine-url=http://127.0.0.1:" + server.getAddress().getPort())) {
      int port = ((WebServerApplicationContext) context).getWebServer().getPort();
      var inspect = new String(fixture("inspect.json"), StandardCharsets.UTF_8);
      assertEquals(
          401,
          client
              .send(
                  request(port, null, "broken", "/v1/inspect"),
                  HttpResponse.BodyHandlers.ofString())
              .statusCode());
      assertEquals(
          401,
          client
              .send(
                  request(port, ENGINE_TOKEN, inspect, "/v1/inspect"),
                  HttpResponse.BodyHandlers.ofString())
              .statusCode());
      var origin =
          HttpRequest.newBuilder(
                  request(port, TOKEN, inspect, "/v1/inspect"), (name, value) -> true)
              .header("Origin", "http://127.0.0.1:3000")
              .build();
      assertEquals(403, client.send(origin, HttpResponse.BodyHandlers.ofString()).statusCode());
      assertEquals(
          400,
          client
              .send(request(port, TOKEN, "{}", "/v1/inspect"), HttpResponse.BodyHandlers.ofString())
              .statusCode());
      assertEquals(
          404,
          client
              .send(request(port, TOKEN, inspect, "/v1/jobs"), HttpResponse.BodyHandlers.ofString())
              .statusCode());
      var good =
          client.send(
              request(port, TOKEN, inspect, "/v1/inspect"), HttpResponse.BodyHandlers.ofString());
      assertEquals(200, good.statusCode());
      assertEquals(TRACE, observed.get());
      assertEquals(TRACE, good.headers().firstValue("traceparent").orElseThrow());
      assertEquals(ID, Contracts.text(Contracts.JSON.readTree(good.body()), "requestId"));
      for (int bad : new int[] {1, 2, 3}) {
        mode.set(bad);
        assertEquals(
            502,
            client
                .send(
                    request(port, TOKEN, inspect, "/v1/inspect"),
                    HttpResponse.BodyHandlers.ofString())
                .statusCode(),
            "mode=" + bad);
      }
      mode.set(4);
      var held =
          client.sendAsync(
              request(port, TOKEN, inspect, "/v1/inspect"), HttpResponse.BodyHandlers.ofString());
      assertTrue(entered.await(10, TimeUnit.SECONDS));
      assertEquals(
          429,
          client
              .send(
                  request(port, TOKEN, inspect, "/v1/inspect"),
                  HttpResponse.BodyHandlers.ofString())
              .statusCode());
      release.countDown();
      assertEquals(200, held.get().statusCode());
      mode.set(5);
      assertEquals(
          504,
          client
              .send(
                  request(port, TOKEN, inspect, "/v1/inspect"),
                  HttpResponse.BodyHandlers.ofString())
              .statusCode());
      var health =
          client.send(
              HttpRequest.newBuilder(URI.create("http://127.0.0.1:" + port + "/health")).build(),
              HttpResponse.BodyHandlers.ofString());
      assertEquals(200, health.statusCode());
      assertFalse(health.body().contains(TOKEN));
      mode.set(0);
      try (var slow = new java.net.Socket("127.0.0.1", port)) {
        slow.setSoTimeout(5000);
        slow.getOutputStream()
            .write(
                ("POST /v1/inspect HTTP/1.1\r\nHost: 127.0.0.1:"
                        + port
                        + "\r\nAuthorization: Bearer "
                        + TOKEN
                        + "\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\n\r\n1\r\n{\r\n")
                    .getBytes(StandardCharsets.US_ASCII));
        slow.getOutputStream().flush();
        var line =
            new java.io.BufferedReader(
                    new java.io.InputStreamReader(slow.getInputStream(), StandardCharsets.US_ASCII))
                .readLine();
        assertTrue(line.contains("504"), "slow upload must hit absolute deadline: " + line);
      }
      assertEquals(
          200,
          client
              .send(
                  request(port, TOKEN, inspect, "/v1/inspect"),
                  HttpResponse.BodyHandlers.ofString())
              .statusCode());
    } finally {
      release.countDown();
      server.stop(0);
    }
  }
}
