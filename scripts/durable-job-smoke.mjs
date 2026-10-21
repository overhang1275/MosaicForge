import assert from "node:assert/strict";
import { createHash, randomUUID } from "node:crypto";
import { createWriteStream } from "node:fs";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { Readable, Transform } from "node:stream";
import { pipeline } from "node:stream/promises";

const mode = process.argv[2] ?? "submit";
assert.ok(["submit", "read", "cancel"].includes(mode));
const origin = process.env.MOSAICFORGE_WEB_ORIGIN ?? "http://127.0.0.1:3000";
const out = new URL("../out/finalize/", import.meta.url);
await mkdir(out, { recursive: true });
async function call(path, body, headers = {}) {
  return fetch(origin + "/api" + path, {
    method: body === undefined ? "GET" : "POST",
    headers: {
      "sec-fetch-site": "same-origin",
      ...(body === undefined
        ? {}
        : { origin, "content-type": "application/json" }),
      ...headers,
    },
    body: body === undefined ? undefined : JSON.stringify(body),
    signal: AbortSignal.timeout(120000),
  });
}
async function json(path, body, status = 200, headers = {}) {
  const response = await call(path, body, headers);
  const result = await response.json();
  assert.equal(response.status, status, JSON.stringify(result));
  return result;
}
let saved;
if (mode === "read") {
  saved = JSON.parse(await readFile(new URL("job-state.json", out)));
} else {
  const fixture = JSON.parse(
    await readFile(
      new URL(
        "../contracts/preview/v1/fixtures/export-pdf.json",
        import.meta.url,
      ),
    ),
  );
  const templateSvg = await readFile(
    new URL("../test.svg", import.meta.url),
    "utf8",
  );
  assert.equal(
    createHash("sha256").update(templateSvg).digest("hex"),
    "ceae058483698070aee16a8d1337715167142b198c47a28e626a0bb8683fbbf8",
  );
  const inspected = await json("/inspect", {
    requestId: randomUUID(),
    templateSvg,
  });
  const job = structuredClone(fixture.job);
  Object.assign(job.generation, {
    count: 1,
    seed: "2026",
    startIndex: "0",
    rulesVersion: "3",
    templateHash: inspected.templateHash,
  });
  Object.assign(job.generation.output, {
    width: 512,
    height: 512,
    format: "svg",
  });
  Object.assign(job.generation.rules, {
    preserveAspectRatio: true,
    scaleX: { min: 1, max: 4 },
    scaleY: { min: 1, max: 4 },
    rotationDegrees: { min: 0, max: 360 },
  });
  job.pdf = {
    exporterVersion: "1",
    trimWidthMm: 100,
    trimHeightMm: 100,
    bleedMm: 0,
  };
  const project = await json(
    "/projects",
    { schemaVersion: "1", name: "Durable finalization QA " + randomUUID() },
    201,
  );
  const projectId = project.project.id;
  const snapshot = {
    requestId: randomUUID(),
    schemaVersion: "2",
    templateSvg,
    job: { ...job, generation: { ...job.generation, count: 12 } },
    variantIndices: Array.from({ length: 12 }, (_, i) => String(i)),
  };
  await json(
    `/projects/${projectId}/revisions`,
    { schemaVersion: "1", expectedRevision: 0, snapshot },
    201,
  );
  const request = {
    schemaVersion: "2",
    projectId,
    projectRevision: 1,
    generation: job.generation,
    requestedAcceptedCount: 12,
    maxCandidates: 120,
    pdf: job.pdf,
    execution: { blockSize: 4, concurrency: 1 },
  };
  const idempotencyKey = randomUUID();
  const result = await json("/jobs", request, 202, {
    "idempotency-key": idempotencyKey,
  });
  const repeated = await json("/jobs", request, 202, {
    "idempotency-key": idempotencyKey,
  });
  assert.equal(repeated.jobId, result.jobId);
  saved = {
    jobId: result.jobId,
    projectId,
    request,
    idempotencyKey,
    submittedState: result.state,
  };
  await writeFile(
    new URL(mode === "cancel" ? "cancel-state.json" : "job-state.json", out),
    JSON.stringify(saved, null, 2) + "\n",
  );
  if (mode === "submit") {
    console.log(JSON.stringify(saved));
    process.exit(0);
  }
  const response = await fetch(origin + `/api/jobs/${saved.jobId}/cancel`, {
    method: "POST",
    headers: { origin, "sec-fetch-site": "same-origin" },
    signal: AbortSignal.timeout(30000),
  });
  assert.ok([200, 202].includes(response.status));
  const cancelled = await response.json();
  assert.equal(cancelled.state, "cancelled");
  assert.equal((await call(`/jobs/${saved.jobId}/artifacts/pdf`)).status, 404);
  await writeFile(
    new URL("cancel-report.json", out),
    JSON.stringify(cancelled, null, 2) + "\n",
  );
  console.log(JSON.stringify(cancelled));
  process.exit(0);
}
let status,
  previousRevision = -1;
const deadline = Date.now() + 240000;
while (Date.now() < deadline) {
  status = await json(`/jobs/${saved.jobId}`);
  assert.ok(status.revision >= previousRevision);
  assert.equal(
    status.evaluatedCount,
    status.acceptedCount + status.rejectedCount,
  );
  previousRevision = status.revision;
  if (["succeeded", "failed", "cancelled"].includes(status.state)) break;
  await new Promise((resolve) => setTimeout(resolve, 1000));
}
assert.equal(status.state, "succeeded", JSON.stringify(status));
assert.equal(status.acceptedCount, 12);
assert.equal(status.artifact.pageCount, 12);
const hash = createHash("sha256");
let bytes = 0;
const pdf = await call(`/jobs/${saved.jobId}/artifacts/pdf`);
assert.equal(pdf.status, 200);
assert.equal(pdf.headers.get("content-type"), "application/pdf");
await pipeline(
  Readable.fromWeb(pdf.body),
  new Transform({
    transform(chunk, _, callback) {
      bytes += chunk.length;
      hash.update(chunk);
      callback(null, chunk);
    },
  }),
  createWriteStream(new URL("job.pdf", out)),
);
assert.equal(bytes, Number(status.artifact.bytes));
assert.equal(hash.digest("hex"), status.artifact.sha256);
assert.equal(pdf.headers.get("etag"), '"' + status.artifact.sha256 + '"');
const manifestResponse = await call(`/jobs/${saved.jobId}/artifacts/manifest`);
assert.equal(manifestResponse.status, 200);
const manifestText = await manifestResponse.text();
await writeFile(new URL("manifest.ndjson", out), manifestText);
const manifest = manifestText
  .trim()
  .split("\n")
  .map((line) => JSON.parse(line));
assert.equal(manifest.length, 14);
assert.equal(manifest[0].jobId, saved.jobId);
assert.equal(manifest[0].artifactId, status.artifact.pdfId);
assert.equal(manifest[0].requestedAcceptedCount, 12);
assert.equal(manifest[13].pageCount, 12);
assert.equal(manifest[13].sha256, status.artifact.sha256);
const indices = manifest.slice(1, -1).map((record, i) => {
  assert.equal(record.kind, "variant");
  assert.equal(record.pageNumber, i + 1);
  return record.index;
});
for (let i = 1; i < indices.length; i++)
  assert.ok(BigInt(indices[i]) > BigInt(indices[i - 1]));
assert.equal(
  manifest[13].selectionSha256,
  createHash("sha256")
    .update(
      "mosaicforge-selection-v1\n" +
        indices.map((index) => index + "\n").join(""),
    )
    .digest("hex"),
);
const head = await fetch(origin + `/api/jobs/${saved.jobId}/artifacts/pdf`, {
  method: "HEAD",
  headers: { "sec-fetch-site": "same-origin" },
});
assert.equal(head.status, 200);
assert.equal(head.headers.get("content-length"), String(bytes));
const range = await call(`/jobs/${saved.jobId}/artifacts/pdf`, undefined, {
  range: "bytes=0-15",
});
assert.equal(range.status, 206);
assert.equal(range.headers.get("content-range"), `bytes 0-15/${bytes}`);
assert.equal((await range.arrayBuffer()).byteLength, 16);
const invalidRange = await call(
  `/jobs/${saved.jobId}/artifacts/pdf`,
  undefined,
  { range: `bytes=${bytes}-` },
);
assert.equal(invalidRange.status, 416);
assert.equal(invalidRange.headers.get("content-range"), `bytes */${bytes}`);
await invalidRange.arrayBuffer();
const report = {
  ...status,
  indices,
  documentConfigurationHash: manifest[0].documentConfigurationHash,
  selectionSha256: manifest[13].selectionSha256,
  manifestSha256: createHash("sha256").update(manifestText).digest("hex"),
  range: "0-15",
  inspectedAt: new Date().toISOString(),
};
await writeFile(
  new URL("job-report.json", out),
  JSON.stringify(report, null, 2) + "\n",
);
console.log(JSON.stringify(report));
