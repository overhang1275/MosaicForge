package dev.mosaicforge.api;

/** Checks the worker's sealed collection before a fenced database association. */
final class Finalization {
  private Finalization() {}

  static void validate(String wire, String response) {
    tools.jackson.databind.JsonNode request, result;
    try {
      request = Contracts.JSON.readTree(wire);
      result = Contracts.JSON.readTree(response);
    } catch (RuntimeException invalid) {
      throw new IllegalArgumentException("Invalid finalization JSON");
    }
    if (!new Contracts().valid(result, "finalize-response", "finalize")
        || !request.path("jobId").equals(result.path("jobId"))
        || !request.path("artifactId").equals(result.path("artifactId"))
        || !JobRequests.hash(wire).equals(Contracts.text(result, "requestHash"))
        || request.path("requestedAcceptedCount").asInt() != result.path("pageCount").asInt())
      throw new IllegalArgumentException("Invalid sealed artifact identity");
    for (var field : new String[] {"bytes", "manifestBytes"}) {
      var bytes = JobRequests.u64(Contracts.text(result, field));
      if (bytes.signum() <= 0
          || bytes.compareTo(java.math.BigInteger.valueOf(HttpEngine.MAX_PDF)) > 0)
        throw new IllegalArgumentException("Invalid sealed artifact size");
    }
    if (JobRequests.u64(Contracts.text(result, "bytes"))
            .add(JobRequests.u64(Contracts.text(result, "manifestBytes")))
            .compareTo(java.math.BigInteger.valueOf(HttpEngine.MAX_PDF))
        > 0) throw new IllegalArgumentException("Sealed collection exceeds quota");
  }
}
