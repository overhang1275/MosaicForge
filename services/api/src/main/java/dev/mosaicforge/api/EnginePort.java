package dev.mosaicforge.api;

import java.io.IOException;
import java.util.concurrent.TimeoutException;

/** The actual HTTP boundary to the CPU engine; no persistence or queue abstraction yet. */
interface EnginePort {
  record Reply(int status, String contentType, byte[] body) {}

  Reply execute(String path, byte[] request, String trace, boolean pdf)
      throws IOException, InterruptedException, TimeoutException;
}
