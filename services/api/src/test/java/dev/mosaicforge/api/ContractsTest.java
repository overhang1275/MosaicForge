package dev.mosaicforge.api;

import static org.junit.jupiter.api.Assertions.*;

import java.io.IOException;
import org.junit.jupiter.api.Test;
import tools.jackson.databind.JsonNode;
import tools.jackson.databind.json.JsonMapper;

class ContractsTest {
  private final JsonMapper json = JsonMapper.builder().build();

  private JsonNode fixture(String name) throws IOException {
    try (var stream = getClass().getResourceAsStream("/fixtures/preview/" + name)) {
      return json.readTree(stream);
    }
  }

  @Test
  void sharedContractFixturesAgreeWithWebAndRust() throws IOException {
    var contracts = new Contracts();
    for (var example : fixture("index.json")) {
      assertEquals(
          example.path("valid").asBoolean(),
          contracts.valid(
              fixture(example.path("file").asString()),
              example.path("schema").asString(),
              example.path("operation").asString()),
          example.path("file").asString());
    }
  }

  @Test
  void traceContextRejectsZeroIdentifiersAndGeneratesSafeContext() {
    var valid = "00-1234567890abcdef1234567890abcdef-1234567890abcdef-01";
    assertEquals(valid, Tracing.context(valid));
    assertNotEquals(
        "00-" + "0".repeat(32) + "-" + "0".repeat(16) + "-01",
        Tracing.context("00-" + "0".repeat(32) + "-" + "0".repeat(16) + "-01"));
    assertTrue(Tracing.context("bad").matches("00-[0-9a-f]{32}-[0-9a-f]{16}-01"));
  }

  @Test
  void selectedPdfContractPreservesExactOrderedU64Indices() throws IOException {
    var contracts = new Contracts();
    try (var stream = getClass().getResourceAsStream("/fixtures/v2/selection-cases.json")) {
      for (var example : json.readTree(stream)) {
        var request = (tools.jackson.databind.node.ObjectNode) fixture("export-pdf.json");
        request.put("schemaVersion", "2");
        request.set("variantIndices", example.path("indices"));
        assertEquals(
            example.path("valid").asBoolean(),
            contracts.valid(request, "selected-pdf-export-request", "export-pdf-selection"),
            example.toString());
      }
    }
  }

  @Test
  void parserRejectsDuplicateFieldsAndTrailingJson() {
    assertThrows(
        RuntimeException.class,
        () -> Contracts.JSON.readTree("{\"requestId\":\"a\",\"requestId\":\"b\"}"));
    assertThrows(RuntimeException.class, () -> Contracts.JSON.readTree("{} {}"));
  }
}
