import assert from "node:assert/strict";
import { readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";

const log = await readFile("out/accepted/browser.log", "utf8");
assert.ok(!log.includes("### Error"), "browser checks must pass");
const result = log.match(/^### Result\r?\n([^\r\n]+)/m);
assert.ok(result, "browser result required");
const data = JSON.parse(result[1]);
assert.equal(data.errors.length, 0);
assert.equal(data.checks.length, 5);
assert.equal(data.acceptedIndices.length, 12);
assert.ok(data.rejectedIndices.length > 0);
assert.deepEqual(data.acceptedIndices, data.selection.variantIndices);
const pdf = await readFile("out/accepted/twelve.pdf");
const replay = await readFile("out/accepted/twelve-replay.pdf");
assert.ok(pdf.length > 0 && pdf.subarray(0, 8).equals(Buffer.from("%PDF-1.7")));
assert.deepEqual(
  pdf,
  replay,
  "saved selection must reproduce identical downloaded PDF bytes",
);
const summary = {
  checks: data.checks,
  acceptedIndices: data.acceptedIndices,
  rejectedIndices: data.rejectedIndices,
  attempts: data.attempts,
  pdfBytes: pdf.length,
  pdfSha256: createHash("sha256").update(pdf).digest("hex"),
  replayIdentical: true,
};
await writeFile(
  "out/accepted/verification.json",
  JSON.stringify(summary, null, 2),
);
console.log(JSON.stringify(summary, null, 2));
