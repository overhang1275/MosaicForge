package dev.mosaicforge.api;

import com.networknt.schema.Schema;
import com.networknt.schema.SchemaRegistry;
import com.networknt.schema.SpecificationVersion;
import dev.mosaicforge.contracts.PreviewOperation;
import java.math.BigInteger;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.Base64;
import java.util.HashMap;
import java.util.HexFormat;
import java.util.Map;
import tools.jackson.core.StreamReadConstraints;
import tools.jackson.core.StreamReadFeature;
import tools.jackson.core.json.JsonFactory;
import tools.jackson.databind.DeserializationFeature;
import tools.jackson.databind.JsonNode;
import tools.jackson.databind.json.JsonMapper;

final class Contracts {
  static final JsonMapper JSON =
      JsonMapper.builder(
              JsonFactory.builder()
                  .streamReadConstraints(
                      StreamReadConstraints.builder()
                          .maxNestingDepth(64)
                          .maxStringLength(16 * 1024 * 1024)
                          .maxNumberLength(128)
                          .build())
                  .build())
          .enable(StreamReadFeature.STRICT_DUPLICATE_DETECTION)
          .enable(DeserializationFeature.FAIL_ON_TRAILING_TOKENS)
          .build();
  private static final BigInteger MAX_U64 = new BigInteger("18446744073709551615");
  private final Map<String, Schema> schemas = new HashMap<>();

  Contracts() {
    var registry = SchemaRegistry.withDefaultDialect(SpecificationVersion.DRAFT_7);
    for (var operation : PreviewOperation.values()) {
      load(registry, operation.requestSchema);
      load(registry, operation.responseSchema);
    }
    load(registry, "error");
    for (var name :
        new String[] {
          "durable-job-request", "durable-job-status", "block-request", "block-response"
        }) load(registry, name);
    for (var name :
        new String[] {
          "create-request",
          "revision-request",
          "me-response",
          "project-response",
          "list-response",
          "revision-response",
          "error"
        }) load(registry, "projects-" + name);
  }

  private void load(SchemaRegistry registry, String name) {
    if (schemas.containsKey(name)) return;
    try (var source = getClass().getResourceAsStream("/contracts/" + name + ".json")) {
      if (source == null) throw new IllegalStateException("Missing generated contract: " + name);
      schemas.put(name, registry.getSchema(source));
    } catch (java.io.IOException failure) {
      throw new IllegalStateException("Cannot load generated contract", failure);
    }
  }

  static String text(JsonNode node, String name) {
    return node.path(name).asString();
  }

  private static boolean u64(JsonNode node) {
    var value = node.asString();
    return value != null
        && value.matches("0|[1-9][0-9]{0,19}")
        && new BigInteger(value).compareTo(MAX_U64) <= 0;
  }

  private static boolean template(JsonNode value) {
    return text(value, "templateSvg").getBytes(StandardCharsets.UTF_8).length <= 5 * 1024 * 1024;
  }

  boolean valid(JsonNode value, String schema, String operation) {
    try {
      if (!schemas.containsKey(schema) || !schemas.get(schema).validate(value).isEmpty())
        return false;
      if (schema.equals("pdf-download-response")) {
        var document = value.path("document");
        var indices = value.path("variantIndices");
        int count = document.path("count").asInt();
        if (count < 1
            || count > 12
            || indices.size() != count
            || !indices.get(0).equals(document.path("startIndex"))
            || !u64(document.path("seed"))
            || !text(document, "documentHash").matches("[0-9a-f]{64}")
            || count != document.path("requestedCount").asInt()
            || count != document.path("acceptedCount").asInt()
            || count != document.path("pageCount").asInt()
            || document.path("rejectedCount").asInt() != 0
            || !text(document, "status").equals("completed")) return false;
        BigInteger previous = BigInteger.valueOf(-1);
        for (var index : indices) {
          if (!u64(index) || new BigInteger(index.asString()).compareTo(previous) <= 0)
            return false;
          previous = new BigInteger(index.asString());
        }
        return true;
      }
      if (schema.startsWith("selected-pdf-export-")) {
        var request = schema.endsWith("request");
        var spec = request ? value.path("job").path("generation") : value.path("document");
        var indices = value.path("variantIndices");
        if (indices.size() != spec.path("count").asInt()
            || !indices.get(0).equals(spec.path("startIndex"))) return false;
        BigInteger previous = BigInteger.valueOf(-1);
        for (var index : indices) {
          if (!u64(index) || new BigInteger(index.asString()).compareTo(previous) <= 0)
            return false;
          previous = new BigInteger(index.asString());
        }
        var legacy = JSON.createObjectNode();
        // Build the existing v1 envelope from the generated v2 envelope.
        legacy.removeAll();
        value
            .properties()
            .forEach(
                entry -> {
                  if (!entry.getKey().equals("schemaVersion")
                      && !entry.getKey().equals("variantIndices"))
                    legacy.set(entry.getKey(), entry.getValue());
                });
        return valid(legacy, request ? "pdf-export-request" : "pdf-export-response", "export-pdf");
      }
      return switch (schema) {
        case "inspect-request" -> template(value);
        case "generation-request" -> template(value) && spec(value.path("generation"), operation);
        case "pdf-export-request" -> {
          var generation = value.path("job").path("generation");
          var single = JSON.createObjectNode();
          generation.properties().forEach(entry -> single.set(entry.getKey(), entry.getValue()));
          single.put("count", 1);
          yield template(value)
              && generation.path("count").asInt() <= 12
              && spec(single, "export")
              && text(generation.path("output"), "format").equals("svg")
              && range(generation);
        }
        case "generation-response" -> {
          var variant = value.path("variant");
          if (!u64(variant.path("seed")) || !u64(variant.path("index"))) yield false;
          if (text(variant, "status").equals("rejected"))
            yield (value.path("artifact").isMissingNode() || value.path("artifact").isNull())
                && (variant.path("artifactHash").isMissingNode()
                    || variant.path("artifactHash").isNull())
                && variant.path("reason").isString();
          yield (variant.path("reason").isMissingNode() || variant.path("reason").isNull())
              && artifact(value, text(variant, "artifactHash"), false)
              && (!operation.equals("preview")
                  || text(value.path("artifact"), "mimeType").equals("image/png"));
        }
        case "pdf-export-response" -> {
          var document = value.path("document");
          int count = document.path("count").asInt();
          yield u64(document.path("seed"))
              && u64(document.path("startIndex"))
              && range(document)
              && count <= 12
              && count == document.path("requestedCount").asInt()
              && count == document.path("acceptedCount").asInt()
              && count == document.path("pageCount").asInt()
              && document.path("rejectedCount").asInt() == 0
              && text(document, "status").equals("completed")
              && artifact(value, text(document, "documentHash"), true);
        }
        case "inspect-response" ->
            value.path("viewBox").get(2).asDouble() > 0
                && value.path("viewBox").get(3).asDouble() > 0;
        default -> true;
      };
    } catch (RuntimeException failure) {
      return false;
    }
  }

  private static boolean range(JsonNode spec) {
    return u64(spec.path("startIndex"))
        && new BigInteger(text(spec, "startIndex"))
                .add(BigInteger.valueOf(spec.path("count").asLong() - 1))
                .compareTo(MAX_U64)
            <= 0;
  }

  private static boolean spec(JsonNode spec, String operation) {
    if (!u64(spec.path("seed"))
        || !u64(spec.path("startIndex"))
        || spec.path("count").asInt() != 1
        || spec.path("output").path("width").asInt() > 512
        || spec.path("output").path("height").asInt() > 512
        || JSON.writeValueAsBytes(spec).length > 1024 * 1024
        || (operation.equals("preview") && !text(spec.path("output"), "format").equals("png")))
      return false;
    var rules = spec.path("rules");
    var lock = rules.path("preserveAspectRatio");
    if ((text(spec, "rulesVersion").equals("1") && !lock.isMissingNode())
        || ((text(spec, "rulesVersion").equals("2") || text(spec, "rulesVersion").equals("3"))
            && !lock.isBoolean())
        || (lock.isBoolean()
            && lock.asBoolean()
            && (rules.path("scaleX").path("min").asDouble()
                    != rules.path("scaleY").path("min").asDouble()
                || rules.path("scaleX").path("max").asDouble()
                    != rules.path("scaleY").path("max").asDouble()))) return false;
    for (var name :
        new String[] {
          "translateX", "translateY", "rotationDegrees", "scaleX", "scaleY", "opacity"
        }) {
      double min = rules.path(name).path("min").asDouble(),
          max = rules.path(name).path("max").asDouble();
      if (!Double.isFinite(min) || !Double.isFinite(max) || min > max) return false;
    }
    if (rules.path("scaleX").path("min").asDouble() <= 0
        || rules.path("scaleY").path("min").asDouble() <= 0
        || rules.path("opacity").path("min").asDouble() < 0
        || rules.path("opacity").path("max").asDouble() > 1) return false;
    var crop = rules.path("crop");
    return crop.isMissingNode()
        || crop.isNull()
        || (Double.isFinite(crop.path("x").asDouble() + crop.path("width").asDouble())
            && Double.isFinite(crop.path("y").asDouble() + crop.path("height").asDouble()));
  }

  private static boolean artifact(JsonNode value, String hash, boolean pdf) {
    String encoded = text(value.path("artifact"), "base64");
    byte[] bytes = Base64.getDecoder().decode(encoded);
    return Base64.getEncoder().encodeToString(bytes).equals(encoded)
        && hash.equals(sha256(bytes))
        && (!pdf
            || (bytes.length >= 8
                && new String(bytes, 0, 8, StandardCharsets.US_ASCII).equals("%PDF-1.7")));
  }

  static String sha256(byte[] bytes) {
    try {
      return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
    } catch (NoSuchAlgorithmException impossible) {
      throw new IllegalStateException(impossible);
    }
  }
}
