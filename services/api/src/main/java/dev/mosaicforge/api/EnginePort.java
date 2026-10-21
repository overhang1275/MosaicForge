package dev.mosaicforge.api;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.concurrent.TimeoutException;

/** The actual HTTP boundary to the CPU engine; no persistence or queue abstraction yet. */
interface EnginePort {
  record Stream(
      int status,
      String contentType,
      String etag,
      String contentRange,
      long length,
      java.io.InputStream body)
      implements AutoCloseable {
    public void close() throws IOException {
      body.close();
    }
  }

  default Stream stream(String path, String method, String range, String ifRange, String trace)
      throws IOException, InterruptedException, TimeoutException {
    throw new IOException("Artifact stream unavailable");
  }

  record Reply(int status, String contentType, byte[] body) {}

  record Download(int status, String contentType, String metadata, Path body)
      implements AutoCloseable {
    @Override
    public void close() throws IOException {
      Files.deleteIfExists(body);
    }
  }

  default Download download(String path, byte[] request, String trace)
      throws IOException, InterruptedException, TimeoutException {
    throw new IOException("Binary download unavailable");
  }

  Reply execute(String path, byte[] request, String trace, boolean pdf)
      throws IOException, InterruptedException, TimeoutException;
}
