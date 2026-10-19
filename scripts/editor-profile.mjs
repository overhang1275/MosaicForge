import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { readFile, mkdir, writeFile } from "node:fs/promises";
import { randomUUID, createHash } from "node:crypto";
import { performance } from "node:perf_hooks";

const exec = promisify(execFile);
const base = process.env.MOSAICFORGE_WEB_ORIGIN || "http://127.0.0.1:3000";
assert.match(base, /^http:\/\/127\.0\.0\.1:\d+$/);
const docker = process.env.DOCKER_BIN || "docker";
await mkdir("out/phase26", { recursive: true });
const { stdout } = await exec(docker, [
  "compose",
  "-f",
  "deploy/compose/local.yml",
  "ps",
  "-q",
  "mosaicforge",
]);
const container = stdout.trim();
assert.match(container, /^[a-f0-9]{12,64}$/);
const templateSvg = await readFile("docs/fixtures/editor-seed.svg", "utf8");
const fixture = JSON.parse(
  await readFile("contracts/preview/v1/fixtures/export-pdf.json", "utf8"),
);
const memoryCode = `const fs=require('fs');const rss={};for(const p of fs.readdirSync('/proc').filter(x=>/^\\d+$/.test(x))){try{const cmd=fs.readFileSync('/proc/'+p+'/cmdline','utf8');const engine=cmd.split('\\0')[0]==='/app/mosaicforge-preview';if(engine||cmd.startsWith('next-server')){const s=fs.readFileSync('/proc/'+p+'/status','utf8');rss[engine?'engine':'web']=Number(s.match(/VmRSS:\\s+(\\d+)/)[1])*1024}}catch{}}console.log(JSON.stringify({cgroupBytes:Number(fs.readFileSync('/sys/fs/cgroup/memory.current','utf8')),rssBytes:rss}))`;
async function memory() {
  const { stdout } = await exec(
    docker,
    ["exec", container, "node", "-e", memoryCode],
    { windowsHide: true },
  );
  const sample = JSON.parse(stdout);
  assert.ok(
    sample.cgroupBytes > 0 &&
      sample.rssBytes.engine > 0 &&
      sample.rssBytes.web > 0,
  );
  return { elapsedMs: performance.now(), ...sample };
}
async function post(operation, payload) {
  const started = performance.now();
  const response = await fetch(`${base}/api/${operation}`, {
    method: "POST",
    headers: { "content-type": "application/json", origin: base },
    body: JSON.stringify({ requestId: randomUUID(), ...payload }),
    signal: AbortSignal.timeout(35000),
  });
  const bytes = Buffer.from(await response.arrayBuffer());
  assert.equal(response.status, 200, `${operation}: ${response.status}`);
  return { bytes, ms: performance.now() - started };
}
const inspection = await post("inspect", { templateSvg });
const metadata = JSON.parse(inspection.bytes);
const generation = fixture.job.generation;
generation.templateHash = metadata.templateHash;
generation.seed = "2026";
// Match the editor's default rules, not identity duplicates.
generation.rules = {
  translateX: { min: -12, max: 12 },
  translateY: { min: -12, max: 12 },
  rotationDegrees: { min: 0, max: 360 },
  scaleX: { min: 1, max: 1.8 },
  scaleY: { min: 1, max: 1.8 },
  mirrorX: true,
  mirrorY: true,
  opacity: { min: 1, max: 1 },
  shuffleLayers: false,
  distortion: 0,
};
await post("preview", {
  templateSvg,
  generation: {
    ...generation,
    count: 1,
    output: { ...generation.output, format: "png" },
  },
});
const runs = [];
for (const count of [1, 6, 12]) {
  for (let repetition = 0; repetition < 3; repetition++) {
    const samples = [await memory()];
    let stopped = false;
    const sampler = (async () => {
      while (!stopped) {
        await new Promise((resolve) => setTimeout(resolve, 250));
        if (!stopped) samples.push(await memory());
      }
    })();
    let run;
    try {
      const started = performance.now();
      const imported = await post("inspect", { templateSvg });
      const previewMs = [];
      for (let index = 0; index < count; index++) {
        const result = await post("preview", {
          templateSvg,
          generation: {
            ...generation,
            count: 1,
            startIndex: String(index),
            output: { ...generation.output, format: "png" },
          },
        });
        assert.equal(JSON.parse(result.bytes).variant.status, "accepted");
        previewMs.push(result.ms);
      }
      const exported = await post("export-pdf", {
        templateSvg,
        job: { ...fixture.job, generation: { ...generation, count } },
      });
      assert.ok(exported.bytes.subarray(0, 5).equals(Buffer.from("%PDF-")));
      const totalMs = performance.now() - started;
      await writeFile(`out/phase26/profile-${count}.pdf`, exported.bytes);
      run = {
        count,
        repetition,
        inspectMs: imported.ms,
        previewMs,
        pdfMs: exported.ms,
        totalMs,
        pdfBytes: exported.bytes.length,
        pdfSha256: createHash("sha256").update(exported.bytes).digest("hex"),
      };
    } finally {
      stopped = true;
      await sampler;
      samples.push(await memory());
    }
    runs.push({ ...run, memorySamples: samples });
  }
}
for (const count of [1, 6, 12])
  assert.equal(
    new Set(
      runs.filter((run) => run.count === count).map((run) => run.pdfSha256),
    ).size,
    1,
    "repeated PDF bytes must be deterministic",
  );
const { stdout: image } = await exec(docker, [
  "inspect",
  "--format",
  "{{.Image}}",
  container,
]);
const report = {
  backend: "cpu",
  image: image.trim(),
  sourceBytes: Buffer.byteLength(templateSvg),
  warmup: "one inspect + one preview",
  method:
    "Sequential BFF HTTP operations, full response bytes; 3 repetitions per count; Docker exec memory sampling target 250ms plus probe latency. RSS samples are not a proven peak. cgroup includes cache and other container processes. No renderer/encoder substage instrumentation.",
  runs,
};
await writeFile("out/phase26/profile.json", JSON.stringify(report, null, 2));
console.log(
  JSON.stringify(
    runs.map(
      ({
        count,
        repetition,
        inspectMs,
        previewMs,
        pdfMs,
        totalMs,
        pdfBytes,
        memorySamples,
      }) => ({
        count,
        repetition,
        inspectMs,
        previewsMs: previewMs.reduce((a, b) => a + b, 0),
        pdfMs,
        totalMs,
        pdfBytes,
        samples: memorySamples.length,
      }),
    ),
    null,
    2,
  ),
);
