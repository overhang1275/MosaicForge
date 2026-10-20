import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { existsSync } from "node:fs";
import { join } from "node:path";

const mode = process.argv[2] ?? "create";
assert.ok(["create", "read"].includes(mode));
const directory = new URL("../out/projects/", import.meta.url);
await mkdir(directory, { recursive: true });
const original = JSON.parse(
  await readFile(
    new URL(
      "../contracts/preview/v1/fixtures/export-pdf.json",
      import.meta.url,
    ),
  ),
);
const snapshot = {
  ...original,
  schemaVersion: "2",
  variantIndices: ["0", "1", "2"],
};
const state =
  mode === "read"
    ? JSON.parse(await readFile(new URL("state.json", directory)))
    : null;
const windowsDocker = join(
  process.env.ProgramFiles ?? "C:/Program Files",
  "Docker/Docker/resources/bin/docker.exe",
);
const docker =
  process.platform === "win32" && existsSync(windowsDocker)
    ? windowsDocker
    : "docker";
const program = `
import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
const input=JSON.parse(readFileSync(0,'utf8'));
let api;
const presence=[];
for (const pid of readdirSync('/proc').filter(p=>/^[0-9]+$/.test(p))) {
  try {
    const args=readFileSync('/proc/'+pid+'/cmdline','utf8').split('\\0');
    const role=args.includes('/app/mosaicforge-api.jar')?'java':(args.includes('/app/apps/web/server.js')||args[0]?.startsWith('next-server ('))?'web':args[0]==='/app/mosaicforge-preview'?'rust':null;
    if(!role) continue;
    const env=Object.fromEntries(readFileSync('/proc/'+pid+'/environ','utf8').split('\\0').filter(Boolean).map(line=>{const n=line.indexOf('=');return[line.slice(0,n),line.slice(n+1)];}));
    const hasDatabasePassword=Object.hasOwn(env,'MOSAICFORGE_DB_PASSWORD');
    assert.equal(hasDatabasePassword,role==='java');
    presence.push({role,hasDatabasePassword});
    if(role==='java') api=env;
  } catch(error) { if(error.code!=='ENOENT'&&error.code!=='EACCES') throw error; }
}
assert.ok(api?.MOSAICFORGE_API_TOKEN);
assert.equal(presence.length,3);
const call=async(path,body)=>{
 const response=await fetch('http://127.0.0.1:4772'+path,{method:body?'POST':'GET',headers:{authorization:'Bearer '+api.MOSAICFORGE_API_TOKEN,...(body?{'content-type':'application/json'}:{})},body:body?JSON.stringify(body):undefined,signal:AbortSignal.timeout(30000)});
 return {status:response.status,body:await response.json()};
};
const me=await call('/v1/me');assert.equal(me.status,200);assert.equal(me.body.user.id,api.MOSAICFORGE_LOCAL_USER_ID);
let id=input.state?.projectId;
if(input.mode==='create'){
 const created=await call('/v1/projects',{schemaVersion:'1',name:'Persistence smoke '+crypto.randomUUID()});
 assert.equal(created.status,201);id=created.body.project.id;
 assert.equal((await call('/v1/projects',{schemaVersion:'1',name:'Invalid owner',ownerId:crypto.randomUUID()})).status,400);
 const revision=await call('/v1/projects/'+id+'/revisions',{schemaVersion:'1',expectedRevision:0,snapshot:input.snapshot});
 assert.equal(revision.status,201);assert.equal(revision.body.revision.number,1);
 assert.deepEqual(revision.body.revision.snapshot,input.snapshot);
 assert.equal((await call('/v1/projects/'+id+'/revisions',{schemaVersion:'1',expectedRevision:0,snapshot:input.snapshot})).status,409);
 const changed=structuredClone(input.snapshot);changed.job.generation.seed='2026';
 assert.equal((await call('/v1/projects/'+id+'/revisions',{schemaVersion:'1',expectedRevision:1,snapshot:changed})).status,201);
 const invalid=structuredClone(input.snapshot);invalid.job.generation.templateHash='0'.repeat(64);
 assert.equal((await call('/v1/projects/'+id+'/revisions',{schemaVersion:'1',expectedRevision:2,snapshot:invalid})).status,400);
}
const project=await call('/v1/projects/'+id);assert.equal(project.status,200);assert.equal(project.body.project.currentRevision,2);
const revision=await call('/v1/projects/'+id+'/revisions/1');assert.equal(revision.status,200);assert.deepEqual(revision.body.revision.snapshot,input.snapshot);
const page=await call('/v1/projects?limit=1');assert.equal(page.status,200);assert.ok(page.body.projects.length<=1);
assert.equal((await call('/v1/projects/not-a-uuid')).status,400);
console.log(JSON.stringify({mode:input.mode,projectId:id,currentRevision:2,originalSeed:revision.body.revision.snapshot.job.generation.seed,credentialPresence:presence}));
`;
const report = JSON.parse(
  execFileSync(
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
    {
      input: JSON.stringify({ mode, state, snapshot }),
      encoding: "utf8",
      maxBuffer: 1024 * 1024,
    },
  ),
);
if (mode === "create")
  await writeFile(
    new URL("state.json", directory),
    JSON.stringify(report, null, 2) + "\n",
  );
await writeFile(
  new URL(`${mode}-report.json`, directory),
  JSON.stringify(report, null, 2) + "\n",
);
console.log(JSON.stringify(report));
