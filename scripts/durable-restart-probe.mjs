// Integration-only fault injection for the QA job saved by durable-job-smoke.
// Stop only the process producing this job, then let the operator restart Docker.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFile, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { join } from "node:path";
const mode = process.argv[2] ?? "partial";
assert.ok(["partial", "sealed"].includes(mode));
const state = JSON.parse(
  await readFile(new URL("../out/finalize/job-state.json", import.meta.url)),
);
const executable = join(
  process.env.ProgramFiles ?? "C:/Program Files",
  "Docker/Docker/resources/bin/docker.exe",
);
const docker =
  process.platform === "win32" && existsSync(executable)
    ? executable
    : "docker";
const program = `
import assert from 'node:assert/strict';
import {readFileSync,readdirSync,existsSync,statSync} from 'node:fs';
const {jobId,mode}=JSON.parse(readFileSync(0,'utf8'));
const root='/app/journal/artifacts';
let collection, stopped;
const deadline=Date.now()+180000;
const pid=(role)=>readdirSync('/proc').filter(p=>/^\\d+$/.test(p)).find(p=>{
 try{const a=readFileSync('/proc/'+p+'/cmdline','utf8').split('\\0');return role==='rust'?a[0]==='/app/mosaicforge-preview':a.includes('/app/mosaicforge-api.jar');}catch{return false;}
});
while(Date.now()<deadline){
 for(const entry of existsSync(root)?readdirSync(root):[]){
  if(!entry.endsWith('.tmp'))continue;
  try{
   const first=readFileSync(root+'/'+entry+'/manifest.ndjson','utf8').split('\\n')[0];
   if(JSON.parse(first).jobId!==jobId)continue;
   if(statSync(root+'/'+entry+'/document.pdf').size<4)continue;
   collection=entry.slice(0,-4);
   const target=pid(mode==='partial'?'rust':'java');assert.ok(target);
   process.kill(Number(target),'SIGSTOP');stopped=mode==='partial'?'rust':'java';
   break;
  }catch(e){if(e.code!=='ENOENT'&&!(e instanceof SyntaxError))throw e;}
 }
 if(stopped)break;
 await new Promise(r=>setTimeout(r,20));
}
assert.ok(stopped,'did not observe this QA job during finalization');
if(mode==='sealed'){
 while(Date.now()<deadline&&!existsSync(root+'/'+collection+'/metadata.json'))await new Promise(r=>setTimeout(r,20));
 assert.ok(existsSync(root+'/'+collection+'/metadata.json'),'worker did not seal orphan');
}
console.log(JSON.stringify({jobId,mode,collection,stopped,sealed:existsSync(root+'/'+collection+'/metadata.json'),partial:existsSync(root+'/'+collection+'.tmp/document.pdf')}));
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
      input: JSON.stringify({ jobId: state.jobId, mode }),
      encoding: "utf8",
      maxBuffer: 1024 * 1024,
    },
  ),
);
await writeFile(
  new URL(`../out/finalize/restart-${mode}.json`, import.meta.url),
  JSON.stringify(report, null, 2) + "\n",
);
console.log(JSON.stringify(report));
