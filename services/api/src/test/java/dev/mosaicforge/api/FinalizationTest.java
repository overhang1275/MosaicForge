package dev.mosaicforge.api;

import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.Test;

class FinalizationTest {
  @Test
  void rejectsIdentityCountHashAndQuotaMismatch() {
    var request = Contracts.JSON.createObjectNode();
    request
        .put("schemaVersion", "1")
        .put("jobId", java.util.UUID.randomUUID().toString())
        .put("artifactId", java.util.UUID.randomUUID().toString())
        .put("requestedAcceptedCount", 2);
    String wire = JobRequests.canonical(request);
    assertThrows(IllegalArgumentException.class, () -> Finalization.validate(wire, "{bad"));
    var response = Contracts.JSON.createObjectNode();
    response.put("schemaVersion", "1");
    response.set("jobId", request.path("jobId"));
    response.set("artifactId", request.path("artifactId"));
    response
        .put("requestHash", JobRequests.hash(wire))
        .put("documentConfigurationHash", "a".repeat(64))
        .put("pageCount", 2)
        .put("sha256", "b".repeat(64))
        .put("bytes", "200")
        .put("manifestSha256", "c".repeat(64))
        .put("manifestBytes", "100")
        .put("selectionSha256", "d".repeat(64));
    assertDoesNotThrow(() -> Finalization.validate(wire, JobRequests.canonical(response)));
    response.put("bytes", "0");
    assertThrows(
        IllegalArgumentException.class,
        () -> Finalization.validate(wire, JobRequests.canonical(response)));
    response.put("bytes", "1073741825");
    assertThrows(
        IllegalArgumentException.class,
        () -> Finalization.validate(wire, JobRequests.canonical(response)));
    response.put("bytes", "200").put("pageCount", 1);
    assertThrows(
        IllegalArgumentException.class,
        () -> Finalization.validate(wire, JobRequests.canonical(response)));
    response.put("pageCount", 2).put("requestHash", "e".repeat(64));
    assertThrows(
        IllegalArgumentException.class,
        () -> Finalization.validate(wire, JobRequests.canonical(response)));
    response
        .put("requestHash", JobRequests.hash(wire))
        .put("artifactId", java.util.UUID.randomUUID().toString());
    assertThrows(
        IllegalArgumentException.class,
        () -> Finalization.validate(wire, JobRequests.canonical(response)));
  }
}
