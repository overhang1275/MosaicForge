package dev.mosaicforge.api;

import jakarta.servlet.ReadListener;
import jakarta.servlet.http.HttpServletRequest;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

final class ProjectBody {
  private ProjectBody() {}

  static byte[] read(HttpServletRequest request, int max)
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
              int count = input.read(buffer, 0, Math.min(buffer.length, max + 1 - output.size()));
              if (count < 0) break;
              output.write(buffer, 0, count);
              if (output.size() > max) {
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
      return result.get(30, TimeUnit.SECONDS);
    } catch (ExecutionException failure) {
      throw new IOException("Invalid request body");
    } finally {
      result.cancel(false);
    }
  }
}
