package dev.mosaicforge.api;

import java.math.BigInteger;
import java.nio.charset.StandardCharsets;
import java.util.TreeMap;
import tools.jackson.databind.JsonNode;

final class JobRequests {
  static final BigInteger MAX_U64 = new BigInteger("18446744073709551615");

  private JobRequests() {}

  static BigInteger u64(String value) {
    if (value == null || !value.matches("0|[1-9][0-9]{0,19}"))
      throw new IllegalArgumentException("Invalid unsigned counter");
    var result = new BigInteger(value);
    if (result.compareTo(MAX_U64) > 0)
      throw new IllegalArgumentException("Unsigned counter overflow");
    return result;
  }

  static JsonNode sorted(JsonNode value) {
    if (value.isObject()) {
      var values = new TreeMap<String, JsonNode>();
      value.properties().forEach(e -> values.put(e.getKey(), e.getValue()));
      var result = Contracts.JSON.createObjectNode();
      values.forEach((key, child) -> result.set(key, sorted(child)));
      return result;
    }
    if (value.isArray()) {
      var result = Contracts.JSON.createArrayNode();
      value.forEach(child -> result.add(sorted(child)));
      return result;
    }
    return value;
  }

  static String canonical(JsonNode value) {
    return Contracts.JSON.writeValueAsString(sorted(value));
  }

  static String hash(String value) {
    return Contracts.sha256(value.getBytes(StandardCharsets.UTF_8));
  }

  static void validate(JsonNode request, JsonNode snapshot) {
    var contracts = new Contracts();
    var generation = request.path("generation");
    if (!contracts.valid(request, "durable-job-request", "durable")
        || request.path("requestedAcceptedCount").asInt() > 12
        || request.path("execution").path("concurrency").asInt() != 1
        || request.path("maxCandidates").asInt() < request.path("requestedAcceptedCount").asInt()
        || generation.path("count").asInt() != 1
        || !Contracts.text(generation.path("output"), "format").equals("svg")
        || !generation
            .path("templateHash")
            .equals(snapshot.path("job").path("generation").path("templateHash")))
      throw new IllegalArgumentException("Invalid or unsupported durable request");
    u64(Contracts.text(generation, "seed"));
    u64(Contracts.text(generation, "startIndex"));
    var single = Contracts.JSON.createObjectNode();
    single.put("requestId", "00000000-0000-0000-0000-000000000000");
    single.set("templateSvg", snapshot.path("templateSvg"));
    single.set("generation", generation);
    if (!contracts.valid(single, "generation-request", "export"))
      throw new IllegalArgumentException("Invalid generation");
  }
}
