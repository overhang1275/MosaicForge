import assert from "node:assert/strict";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { randomUUID, createHash } from "node:crypto";
import { resolve } from "node:path";

const log = await readFile("out/phase26/browser.log", "utf8");
assert.ok(
  !log.includes("### Error"),
  "browser run must pass before comparing artifacts",
);
const match = log.match(/^### Result\r?\n([^\r\n]+)/m);
assert.ok(match, "browser log must contain a JSON result");
const result = JSON.parse(match[1]);
assert.equal(result.errors.length, 0);
assert.equal(result.checks.length, 6);
const directory = `out/phase26/parity-${randomUUID()}`;
await mkdir(directory);
const template = "docs/fixtures/editor-seed.svg";
const templateSvg = await readFile(template, "utf8");
const cli =
  process.env.MOSAICFORGE_CLI ||
  `services/engine/target/debug/mosaicforge${process.platform === "win32" ? ".exe" : ""}`;
const exec = promisify(execFile);
const jobFile = `${directory}/pdf-job.json`;
await writeFile(jobFile, JSON.stringify(result.pdfJob));
await exec(
  resolve(cli),
  [
    "generate-pdf",
    "--template",
    template,
    "--spec",
    jobFile,
    "--out",
    `${directory}/pdf`,
  ],
  { windowsHide: true },
);
const cliPdf = await readFile(`${directory}/pdf/mosaic.pdf`);
const browserPdf = await readFile("out/phase26/editor.pdf");
assert.deepEqual(
  browserPdf,
  cliPdf,
  "browser PDF must match CLI bytes for the saved job",
);
const artifacts = [
  {
    format: "pdf",
    bytes: cliPdf.length,
    sha256: createHash("sha256").update(cliPdf).digest("hex"),
  },
];
const base = process.env.MOSAICFORGE_WEB_ORIGIN || "http://127.0.0.1:3000";
assert.match(base, /^http:\/\/127\.0\.0\.1:\d+$/);
for (const format of ["png", "svg"]) {
  const generation = structuredClone(result.pdfJob.generation);
  generation.count = 1;
  generation.output.format = format;
  const spec = `${directory}/${format}.json`;
  await writeFile(spec, JSON.stringify(generation));
  await exec(
    resolve(cli),
    [
      "generate",
      "--template",
      template,
      "--spec",
      spec,
      "--out",
      `${directory}/${format}`,
    ],
    { windowsHide: true },
  );
  const expected = await readFile(
    `${directory}/${format}/variant-${generation.startIndex}.${format}`,
  );
  const response = await fetch(`${base}/api/export`, {
    method: "POST",
    headers: { "content-type": "application/json", origin: base },
    body: JSON.stringify({ requestId: randomUUID(), templateSvg, generation }),
    signal: AbortSignal.timeout(30000),
  });
  assert.equal(response.status, 200);
  assert.match(response.headers.get("content-disposition"), /attachment/);
  assert.deepEqual(
    Buffer.from(await response.arrayBuffer()),
    expected,
    `${format} CLI/BFF bytes`,
  );
  artifacts.push({
    format,
    bytes: expected.length,
    sha256: createHash("sha256").update(expected).digest("hex"),
  });
}
await writeFile(
  "out/phase26/parity.json",
  JSON.stringify({ checks: result.checks, artifacts }, null, 2),
);
console.log(JSON.stringify(artifacts, null, 2));
