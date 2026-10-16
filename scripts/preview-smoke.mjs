import assert from "node:assert/strict";
import { randomBytes, randomUUID, createHash } from "node:crypto";
import { spawn, execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createServer } from "node:net";
import { request as httpRequest } from "node:http";
import { once } from "node:events";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { setTimeout as delay } from "node:timers/promises";

const root = fileURLToPath(new URL("../", import.meta.url));
const engineOnly = process.argv.includes("--engine-only");
const suffix = process.platform === "win32" ? ".exe" : "";
const bin = resolve(root, "services/engine/target/debug");
const directory = resolve(root, "out", `preview-smoke-${randomUUID()}`);
await mkdir(directory, { recursive: true });
async function freePort() {
  const server = createServer();
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const port = server.address().port;
  await new Promise((r) => server.close(r));
  return port;
}
const enginePort = await freePort(),
  webPort = await freePort();
const engineUrl = `http://127.0.0.1:${enginePort}`,
  webUrl = `http://127.0.0.1:${webPort}`;
const token = randomBytes(32).toString("hex");
const children = [];
let session;
function start(executable, args, env, cwd = root) {
  const child = spawn(executable, args, {
    cwd,
    env: { ...process.env, ...env },
    windowsHide: true,
    stdio: ["ignore", "pipe", "pipe"],
  });
  children.push(child);
  let log = "";
  child.stdout.on("data", (d) => (log += d));
  child.stderr.on("data", (d) => (log += d));
  child.on("error", () => {});
  return { child, log: () => log };
}
async function waitReady(url, child) {
  for (let i = 0; i < 100; i++) {
    if (child.exitCode !== null)
      throw new Error("Local server exited before readiness");
    try {
      await fetch(url, { signal: AbortSignal.timeout(1000) });
      return;
    } catch {}
    await delay(200);
  }
  throw new Error("Local server readiness timeout");
}
const headers = {
  "content-type": "application/json",
  authorization: `Bearer ${token}`,
};
async function post(path, value, bff = false) {
  return fetch((bff ? webUrl + "/api" : engineUrl + "/v1") + path, {
    method: "POST",
    headers: bff
      ? { "content-type": "application/json", origin: webUrl }
      : headers,
    body: JSON.stringify(value),
    signal: AbortSignal.timeout(30_000),
  });
}
const id = randomUUID();
const templateSvg =
  '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="80"><rect width="100" height="80" fill="#123456"/></svg>';
const fixture = JSON.parse(
  await readFile(
    resolve(root, "contracts/preview/v1/fixtures/export-svg.json"),
    "utf8",
  ),
);
const evidence = {
  mode: engineOnly ? "engine" : "engine+bff",
  checks: [],
  artifacts: [],
};
try {
  if (!engineOnly) {
    const { startSession } = await import("./start-preview.mjs");
    session = startSession(enginePort, webPort, token);
  }
  const engine = engineOnly
    ? start(resolve(bin, `mosaicforge-preview${suffix}`), [], {
        MOSAICFORGE_ENGINE_TOKEN: token,
        MOSAICFORGE_ENGINE_PORT: String(enginePort),
      })
    : { child: session.children[0] };
  await waitReady(engineUrl + "/v1/inspect", engine.child);
  const inspection = await post("/inspect", { requestId: id, templateSvg });
  assert.equal(inspection.status, 200);
  const metadata = await inspection.json();
  assert.equal(metadata.requestId, id);
  evidence.checks.push("HTTP inspect real");
  // Leave a chunked upload open: it must occupy the one slot, then release on disconnect.
  const partial = httpRequest(engineUrl + "/v1/inspect", {
    method: "POST",
    headers,
  });
  partial.on("error", () => {});
  partial.write("{");
  await delay(80);
  assert.equal(
    (await post("/inspect", { requestId: id, templateSvg })).status,
    429,
  );
  partial.destroy();
  await delay(80);
  assert.equal(
    (await post("/inspect", { requestId: id, templateSvg })).status,
    200,
  );
  evidence.checks.push("busy 429 and disconnected upload releases slot");
  const tooLarge = await new Promise((resolveResult, reject) => {
    const req = httpRequest(
      engineUrl + "/v1/inspect",
      { method: "POST", headers },
      (res) => {
        res.resume();
        res.on("end", () => resolveResult(res.statusCode));
      },
    );
    req.on("error", reject);
    const chunk = Buffer.alloc(1024 * 1024, 32);
    for (let i = 0; i < 33; i++) req.write(chunk);
    req.end();
  });
  assert.equal(tooLarge, 413);
  evidence.checks.push("chunked body >32 MiB rejected");
  if (!engineOnly) {
    const web = { child: session.children[1] };
    await waitReady(webUrl, web.child);
    const html = await (await fetch(webUrl)).text();
    const asset = html.match(/src="([^\"]*\/_next\/static\/[^\"]+)"/);
    assert.ok(asset, "standalone HTML must reference a static script");
    assert.equal(
      (await fetch(new URL(asset[1], webUrl))).status,
      200,
      "standalone static asset must be packaged",
    );
    evidence.checks.push("standalone HTML referenced static script HTTP 200");
    const res = await post("/inspect", { requestId: id, templateSvg }, true);
    assert.equal(res.status, 200);
    assert.equal((await res.json()).templateHash, metadata.templateHash);
    evidence.checks.push("standalone Next BFF inspect real");
    const foreign = await fetch(webUrl + "/api/inspect", {
      method: "POST",
      headers: {
        "content-type": "application/json",
        origin: "https://evil.example",
      },
      body: "{}",
    });
    assert.equal(foreign.status, 403);
    const bffLarge = await new Promise((resolveResult, reject) => {
      const req = httpRequest(
        webUrl + "/api/inspect",
        {
          method: "POST",
          headers: { "content-type": "application/json", origin: webUrl },
        },
        (res) => {
          res.resume();
          res.on("end", () => resolveResult(res.statusCode));
        },
      );
      req.on("error", reject);
      const chunk = Buffer.alloc(1024 * 1024, 32);
      for (let i = 0; i < 33; i++) req.write(chunk);
      req.end();
    });
    assert.equal(bffLarge, 413);
    evidence.checks.push("real BFF foreign Origin 403 and chunked >32 MiB 413");
  }
  await writeFile(resolve(directory, "template.svg"), templateSvg);
  for (const format of ["svg", "png"]) {
    const generation = structuredClone(fixture.generation);
    generation.output.format = format;
    generation.output.width = 128;
    generation.output.height = 128;
    generation.templateHash = metadata.templateHash;
    await writeFile(
      resolve(directory, `${format}.json`),
      JSON.stringify(generation),
    );
    execFileSync(
      resolve(bin, `mosaicforge${suffix}`),
      [
        "generate",
        "--template",
        resolve(directory, "template.svg"),
        "--spec",
        resolve(directory, `${format}.json`),
        "--out",
        resolve(directory, format),
      ],
      { windowsHide: true, stdio: "pipe" },
    );
    const expected = await readFile(
      resolve(directory, format, `variant-18446744073709551615.${format}`),
    );
    const res = await post(format === "png" ? "/preview" : "/export", {
      requestId: id,
      templateSvg,
      generation,
    });
    assert.equal(res.status, 200);
    const output = await res.json();
    const bytes = Buffer.from(output.artifact.base64, "base64");
    assert.deepEqual(bytes, expected);
    assert.equal(output.variant.index, "18446744073709551615");
    const hash = createHash("sha256").update(expected).digest("hex");
    assert.equal(output.variant.artifactHash, hash);
    if (!engineOnly) {
      const downloaded = await post(
        "/export",
        { requestId: id, templateSvg, generation },
        true,
      );
      assert.equal(downloaded.status, 200);
      assert.deepEqual(Buffer.from(await downloaded.arrayBuffer()), expected);
      assert.match(downloaded.headers.get("content-disposition"), /attachment/);
    }
    evidence.artifacts.push({ format, bytes: expected.length, sha256: hash });
  }
  evidence.checks.push("max u64 seed/index, CLI=HTTP bytes and hashes SVG/PNG");
  if (!engineOnly) evidence.checks.push("CLI=BFF attachment bytes SVG/PNG");
  const rejectSvg = fixture.templateSvg;
  const inspectRejected = await (
    await post("/inspect", { requestId: id, templateSvg: rejectSvg })
  ).json();
  const generation = structuredClone(fixture.generation);
  generation.templateHash = inspectRejected.templateHash;
  const rejected = await (
    await post("/export", { requestId: id, templateSvg: rejectSvg, generation })
  ).json();
  assert.equal(rejected.variant.status, "rejected");
  assert.equal(rejected.artifact, null);
  evidence.checks.push("real engine rejection without bytes");
  await writeFile(
    resolve(directory, "result.json"),
    JSON.stringify(evidence, null, 2),
  );
  console.log(JSON.stringify({ directory, ...evidence }, null, 2));
} finally {
  await session?.stop();
  for (const child of session?.children ?? []) {
    assert.ok(
      child.exitCode !== null || child.signalCode !== null,
      "owned local server must exit on cleanup",
    );
  }
  for (const child of children.reverse()) {
    if (child.exitCode === null) {
      const exited = once(child, "exit");
      child.kill();
      await Promise.race([exited, delay(3000)]);
    }
  }
}
