package dev.mosaicforge.api;

import java.util.UUID;

final class Tracing {
  private Tracing() {}

  static String context(String value) {
    if (value != null
        && value.matches("00-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}")
        && !value.substring(3, 35).equals("0".repeat(32))
        && !value.substring(36, 52).equals("0".repeat(16))) return value;
    return "00-"
        + UUID.randomUUID().toString().replace("-", "")
        + "-"
        + UUID.randomUUID().toString().replace("-", "").substring(0, 16)
        + "-01";
  }
}
