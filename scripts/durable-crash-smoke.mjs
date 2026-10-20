import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { readFile, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { join } from "node:path";

// Run durable-block-smoke prepare first. Interrupt only this project's application.
const root = new URL("../", import.meta.url);
const state = JSON.parse(
  await readFile(new URL("out/durable/block-state.json", root)),
);
assert.equal(state.response, null, "prepare a fresh uncommitted block first");
const blockId = JSON.parse(state.wire).blockId;
assert.match(blockId, /^[0-9a-f-]{36}$/);
const windowsDocker = join(
  process.env.ProgramFiles ?? "C:/Program Files",
  "Docker/Docker/resources/bin/docker.exe",
);
const docker =
  process.platform === "win32" && existsSync(windowsDocker)
    ? windowsDocker
    : "docker";
const program = `
import {readFileSync,readdirSync} from 'node:fs';
const wire=readFileSync(0,'utf8');let token;
for(const pid of readdirSync('/proc').filter(p=>/^[0-9]+$/.test(p))){try{
 const args=readFileSync('/proc/'+pid+'/cmdline','utf8').split('\\0');
 if(args[0]!=='/app/mosaicforge-preview')continue;
 token=readFileSync('/proc/'+pid+'/environ','utf8').split('\\0').find(s=>s.startsWith('MOSAICFORGE_ENGINE_TOKEN='))?.slice('MOSAICFORGE_ENGINE_TOKEN='.length);
}catch(e){if(e.code!=='ENOENT'&&e.code!=='EACCES')throw e;}}
if(!token)throw new Error('private engine unavailable');
console.error('block-request-started');
try{const r=await fetch('http://127.0.0.1:4771/internal/v1/blocks',{method:'POST',headers:{authorization:'Bearer '+token,'content-type':'application/json'},body:wire,signal:AbortSignal.timeout(60000)});await r.arrayBuffer();process.exitCode=r.ok?0:2;}
catch{process.exitCode=3;}
`;
const child = spawn(
  docker,
  [
    "exec",
    "-i",
    "mosaicforge-local-mosaicforge-1",
    "node",
    "--input-type=module",
    "-e",
    program,
  ],
  { stdio: ["pipe", "ignore", "pipe"] },
);
const completion = new Promise((resolve, reject) => {
  child.on("error", reject);
  child.on("close", resolve);
});
const ready = new Promise((resolve, reject) => {
  let stderr = "";
  child.stderr.on("data", (bytes) => {
    stderr += bytes.toString();
    if (stderr.includes("block-request-started")) resolve();
  });
  child.on("close", () => reject(new Error("request stopped before starting")));
  child.on("error", reject);
});
child.stdin.end(state.wire);
await ready;
await new Promise((resolve) => setTimeout(resolve, 150));
const check = execFileSync(
  docker,
  [
    "exec",
    "mosaicforge-local-mosaicforge-1",
    "node",
    "-e",
    "console.log(require('node:fs').existsSync('/app/journal/'+process.argv[1]+'.json'))",
    blockId,
  ],
  { encoding: "utf8" },
).trim();
assert.equal(
  check,
  "false",
  "block completed before crash; repeat prepare to test interruption",
);
execFileSync(docker, ["restart", "mosaicforge-local-mosaicforge-1"], {
  stdio: "pipe",
});
const code = await completion;
assert.notEqual(
  code,
  0,
  "request must be interrupted before successful confirmation",
);
const report = {
  blockId,
  committedBeforeRestart: false,
  interruptedExitCode: code,
};
await writeFile(
  new URL("out/durable/crash-report.json", root),
  JSON.stringify(report, null, 2) + "\n",
);
console.log(JSON.stringify(report));
