package dev.mosaicforge.api;

import java.math.BigInteger;

final class BlockResults {
  private BlockResults() {}

  static void validate(String wire, String response) {
    var request = Contracts.JSON.readTree(wire);
    var result = Contracts.JSON.readTree(response);
    var contracts = new Contracts();
    if (!contracts.valid(request, "block-request", "block")
        || !contracts.valid(result, "block-response", "block")
        || !request.path("jobId").equals(result.path("jobId"))
        || !request.path("blockId").equals(result.path("blockId"))
        || !request.path("startIndex").equals(result.path("startIndex"))
        || !JobRequests.hash(wire).equals(Contracts.text(result, "requestHash")))
      throw new IllegalArgumentException("Invalid block identity");
    int evaluated = result.path("evaluatedCount").asInt(),
        accepted = result.path("acceptedCount").asInt(),
        rejected = result.path("rejectedCount").asInt();
    if (evaluated != accepted + rejected
        || evaluated > request.path("candidateCount").asInt()
        || accepted > request.path("acceptedLimit").asInt()
        || evaluated != result.path("records").size()
        || (evaluated < request.path("candidateCount").asInt()
            && accepted != request.path("acceptedLimit").asInt())
        || !JobRequests.hash(JobRequests.canonical(result.path("records")))
            .equals(Contracts.text(result, "recordsSha256")))
      throw new IllegalArgumentException("Invalid block counters or digest");
    BigInteger next = JobRequests.u64(Contracts.text(request, "startIndex"));
    int counted = 0;
    for (var record : result.path("records")) {
      if (!JobRequests.u64(Contracts.text(record, "index")).equals(next))
        throw new IllegalArgumentException("Unordered block records");
      boolean ok = record.path("accepted").asBoolean();
      if (ok) {
        counted++;
        if (!record.path("reason").isNull()) throw new IllegalArgumentException("Accepted reason");
      } else if (!record.path("reason").isString() || record.path("reason").asString().isBlank())
        throw new IllegalArgumentException("Missing rejection reason");
      if (counted == request.path("acceptedLimit").asInt()
          && !record.equals(result.path("records").get(evaluated - 1)))
        throw new IllegalArgumentException("Speculative records");
      next = next.add(BigInteger.ONE);
    }
    if (counted != accepted
        || (next.compareTo(JobRequests.MAX_U64) > 0
            ? !result.path("nextCandidateIndex").isNull()
            : !next.equals(JobRequests.u64(Contracts.text(result, "nextCandidateIndex")))))
      throw new IllegalArgumentException("Invalid next index");
  }
}
