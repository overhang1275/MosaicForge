package dev.mosaicforge.api;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.ByteBuffer;
import java.time.Duration;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.Flow;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

final class HttpEngine implements EnginePort {
  static final int MAX_RESPONSE = 16 * 1024 * 1024;
  private final URI origin;
  private final String token;
  private final long timeoutMs;
  private final HttpClient client =
      HttpClient.newBuilder()
          .connectTimeout(Duration.ofSeconds(3))
          .followRedirects(HttpClient.Redirect.NEVER)
          .build();

  HttpEngine(String url, String token, long timeoutMs) {
    origin = URI.create(url);
    if (!"http".equals(origin.getScheme())
        || !"127.0.0.1".equals(origin.getHost())
        || origin.getUserInfo() != null
        || origin.getRawQuery() != null
        || origin.getRawFragment() != null
        || !origin.getRawPath().isEmpty()
        || origin.getPort() < 1
        || origin.getPort() > 65535)
      throw new IllegalArgumentException("Expected canonical engine HTTP loopback origin");
    if (!token.matches("[!-~]{32,256}") || timeoutMs < 1 || timeoutMs > 30000)
      throw new IllegalArgumentException("Invalid engine credentials or deadline");
    this.token = token;
    this.timeoutMs = timeoutMs;
  }

  @Override
  public Reply execute(String path, byte[] request, String trace, boolean pdf)
      throws IOException, InterruptedException, TimeoutException {
    long deadline = pdf ? 120000 : timeoutMs;
    var message =
        HttpRequest.newBuilder(origin.resolve(path))
            .timeout(Duration.ofMillis(deadline))
            .header("Authorization", "Bearer " + token)
            .header("Content-Type", "application/json")
            .header("traceparent", trace)
            .POST(HttpRequest.BodyPublishers.ofByteArray(request))
            .build();
    var result =
        client.sendAsync(
            message,
            info ->
                new BoundedBody(
                    info.headers().firstValueAsLong("Content-Length").orElse(-1) > MAX_RESPONSE));
    try {
      var response = result.get(deadline, TimeUnit.MILLISECONDS);
      return new Reply(
          response.statusCode(),
          response.headers().firstValue("Content-Type").orElse(""),
          response.body());
    } catch (ExecutionException failure) {
      if (failure.getCause() instanceof java.net.http.HttpTimeoutException)
        throw new TimeoutException();
      throw new IOException("Engine transport failure", failure);
    } catch (TimeoutException | InterruptedException failure) {
      result.cancel(true);
      throw failure;
    }
  }

  /** Cancels at the byte limit while receiving, rather than buffering an unbounded response. */
  private static final class BoundedBody implements HttpResponse.BodySubscriber<byte[]> {
    private final CompletableFuture<byte[]> result = new CompletableFuture<>();
    private final ByteArrayOutputStream output = new ByteArrayOutputStream();
    private Flow.Subscription subscription;
    private final boolean oversized;

    BoundedBody(boolean oversized) {
      this.oversized = oversized;
    }

    @Override
    public CompletionStage<byte[]> getBody() {
      return result;
    }

    @Override
    public void onSubscribe(Flow.Subscription value) {
      subscription = value;
      if (oversized) {
        value.cancel();
        result.completeExceptionally(new IOException("Engine response too large"));
        return;
      }
      value.request(1);
    }

    @Override
    public void onNext(List<ByteBuffer> chunks) {
      for (var chunk : chunks) {
        if ((long) output.size() + chunk.remaining() > MAX_RESPONSE) {
          subscription.cancel();
          result.completeExceptionally(new IOException("Engine response too large"));
          return;
        }
        byte[] bytes = new byte[chunk.remaining()];
        chunk.get(bytes);
        output.writeBytes(bytes);
      }
      subscription.request(1);
    }

    @Override
    public void onError(Throwable failure) {
      result.completeExceptionally(failure);
    }

    @Override
    public void onComplete() {
      result.complete(output.toByteArray());
    }
  }
}
