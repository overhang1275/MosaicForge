import assert from "node:assert/strict";
import { readFile, writeFile } from "node:fs/promises";

const browser = await readFile("out/accepted/browser.log", "utf8");
assert.ok(!browser.includes("### Error"), "browser E2E must pass");
const result = browser.match(/^### Result\r?\n([^\r\n]+)/m);
assert.ok(result, "browser result required");
const data = JSON.parse(result[1]);
assert.equal(data.acceptedIndices.length, 12);
assert.ok(data.traces.length >= 12);
const events = (await readFile("out/phase31/runtime.log", "utf8"))
  .split(/\r?\n/)
  .flatMap((line) => {
    try {
      const event = JSON.parse(line);
      return event.message === "control.request" ? [event] : [];
    } catch {
      return [];
    }
  });
for (const trace of data.traces) {
  assert.ok(
    events.some(
      (event) =>
        event.requestId === trace.requestId &&
        event.traceparent === trace.traceparent &&
        event.operation === "/v1/preview" &&
        event.status === 200,
    ),
    "each real browser candidate must have a matching Java request/trace event",
  );
}
const ports = JSON.parse(await readFile("out/phase31/ports.json", "utf8"));
const published = Object.entries(ports).filter(
  ([, bindings]) => bindings?.length,
);
assert.equal(published.length, 1);
assert.equal(published[0][0], "3000/tcp");
assert.equal(published[0][1][0].HostIp, "127.0.0.1");
const summary = {
  candidateTracesMatched: data.traces.length,
  acceptedCount: 12,
  javaEvents: events.length,
  publishedPorts: published,
};
await writeFile(
  "out/phase31/control-verification.json",
  JSON.stringify(summary, null, 2),
);
console.log(JSON.stringify(summary));
