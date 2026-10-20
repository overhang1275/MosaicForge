import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { readFile, mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

const mode = process.argv[2] ?? "create";
assert.ok(["create", "read", "prepare"].includes(mode));
const root = new URL("../", import.meta.url);
const out = new URL("out/durable/", root);
await mkdir(out, { recursive: true });
const state =
  mode === "read"
    ? JSON.parse(await readFile(new URL("block-state.json", out)))
    : null;
const fixture = JSON.parse(
  await readFile(
    new URL("contracts/preview/v1/fixtures/export-pdf.json", root),
  ),
);
const templateSvg =
  mode !== "read" ? await readFile(new URL("test.svg", root), "utf8") : null;
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
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { createHash, randomUUID } from 'node:crypto';
const input=JSON.parse(readFileSync(0,'utf8'));
let env;
const presence=[];
for(const pid of readdirSync('/proc').filter(p=>/^[0-9]+$/.test(p))){
 try{
  const args=readFileSync('/proc/'+pid+'/cmdline','utf8').split('\\0');
  const role=args.includes('/app/mosaicforge-api.jar')?'java':args[0]==='/app/mosaicforge-preview'?'rust':(args.includes('/app/apps/web/server.js')||args[0]?.startsWith('next-server ('))?'web':null;
  if(!role) continue;
  const vars=Object.fromEntries(readFileSync('/proc/'+pid+'/environ','utf8').split('\\0').filter(Boolean).map(s=>{const i=s.indexOf('=');return[s.slice(0,i),s.slice(i+1)];}));
  presence.push({role,valkeyPassword:Object.hasOwn(vars,'MOSAICFORGE_VALKEY_PASSWORD'),journalDirectory:Object.hasOwn(vars,'MOSAICFORGE_JOURNAL_DIR')});
  if(role==='java')env=vars;
 }catch(e){if(e.code!=='ENOENT'&&e.code!=='EACCES')throw e;}
}
assert.ok(env?.MOSAICFORGE_ENGINE_TOKEN);
const call=async(path,body)=>{
 const r=await fetch('http://127.0.0.1:4771'+path,{method:'POST',headers:{authorization:'Bearer '+env.MOSAICFORGE_ENGINE_TOKEN,'content-type':'application/json'},body,signal:AbortSignal.timeout(60000)});
 assert.equal(r.status,200,'private engine request failed');return await r.text();
};
let wire=input.state?.wire;
if(!wire){
 const inspected=JSON.parse(await call('/v1/inspect',JSON.stringify({requestId:randomUUID(),templateSvg:input.templateSvg})));
 const job=input.fixture.job;
 Object.assign(job.generation,{count:1,seed:'2026',rulesVersion:'3',templateHash:inspected.templateHash});
 Object.assign(job.generation.output,{width:512,height:512,format:'svg'});
 Object.assign(job.generation.rules,{preserveAspectRatio:true,scaleX:{min:1,max:4},scaleY:{min:1,max:4},rotationDegrees:{min:0,max:360}});
 wire=JSON.stringify({schemaVersion:'1',jobId:randomUUID(),blockId:randomUUID(),templateSvg:input.templateSvg,job,startIndex:'0',candidateCount:4,acceptedLimit:4});
}
if(input.mode==='prepare'){
 console.log(JSON.stringify({wire,response:null,report:{mode:'prepare',blockId:JSON.parse(wire).blockId}}));
}else{
const response=await call('/internal/v1/blocks',wire);
const result=JSON.parse(response);
assert.equal(result.requestHash,createHash('sha256').update(wire).digest('hex'));
assert.equal(result.evaluatedCount,result.acceptedCount+result.rejectedCount);
assert.equal(result.evaluatedCount,result.records.length);
assert.equal(result.acceptedCount,4);
assert.deepEqual(result.records.map(r=>r.index),['0','1','2','3']);
const sorted=result.records.map(r=>Object.fromEntries(Object.entries(r).sort(([a],[b])=>a.localeCompare(b))));
assert.equal(result.recordsSha256,createHash('sha256').update(JSON.stringify(sorted)).digest('hex'));
if(input.state?.response)assert.equal(response,input.state.response,'restarted journal must replay identical bytes');
for(const p of presence){assert.equal(p.valkeyPassword,p.role==='java');assert.equal(p.journalDirectory,p.role==='rust');}
assert.equal(statSync('/app/journal').mode&0o777,0o700);
console.log(JSON.stringify({wire,response,report:{mode:input.mode,jobId:result.jobId,blockId:result.blockId,acceptedCount:result.acceptedCount,evaluatedCount:result.evaluatedCount,requestHash:result.requestHash,recordsSha256:result.recordsSha256,credentialPresence:presence}}));
}
`;
const result = JSON.parse(
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
      input: JSON.stringify({ mode, state, fixture, templateSvg }),
      encoding: "utf8",
      maxBuffer: 8 * 1024 * 1024,
    },
  ),
);
if (mode !== "read" || !state?.response)
  await writeFile(
    new URL("block-state.json", out),
    JSON.stringify({ wire: result.wire, response: result.response }),
  );
await writeFile(
  new URL(`block-${mode}-report.json`, out),
  JSON.stringify(result.report, null, 2) + "\n",
);
console.log(JSON.stringify(result.report));
