import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sha256, wholeJson } from "../../../../tooling/lint-default-correctness-reference.mjs";
const fixture = path.dirname(fileURLToPath(import.meta.url));
const corpus = JSON.parse(fs.readFileSync(path.join(fixture, "cases.json")));
const binary = process.argv[2];
assert.equal(sha256(fs.readFileSync(binary)), corpus.beforeBaseline.binarySha256);
assert.match(spawnSync(binary,["--version"],{encoding:"utf8"}).stdout,/^vize 0\.443\.0/);
const config = JSON.stringify({linter:{preset:"incremental",rules:{[corpus.rule]:"error"}}})+"\n";
const receipt = {schema:"vize.lint.reserved-suppression-public-before",version:1,baseline:corpus.beforeBaseline,config,custody:["cases.json","observe-public-before.mjs"].map((name)=>({path:name,sha256:sha256(fs.readFileSync(path.join(fixture,name)))})),runs:[]};
const directory = fs.mkdtempSync(path.join(os.tmpdir(),"vize-reserved-suppression-before-"));
try {
 fs.writeFileSync(path.join(directory,"vize.config.json"),config);
 for(const entry of corpus.cases) {
  const bytes=fs.readFileSync(path.join(fixture,entry.filename+".fixture"));
  assert.equal(bytes.toString("utf8"),entry.source);assert.equal(sha256(bytes),entry.sourceSha256);
  fs.writeFileSync(path.join(directory,entry.filename),bytes);
  const argv=["lint","-c","vize.config.json","--format","json","--locale","en","--help-level","none",entry.filename];
  const result=spawnSync(binary,argv,{cwd:directory,encoding:"utf8",timeout:60000,maxBuffer:4*1024*1024});
  const run={id:entry.id,argv,input:{text:entry.source,sha256:entry.sourceSha256},status:result.status,error:result.error?{name:result.error.name,message:result.error.message}:null,signal:result.signal,stdout:result.stdout??"",stderr:result.stderr??"",expected:wholeJson(entry,entry.beforeDiagnostics),expectedStatus:entry.beforeDiagnostics.length?1:0,actual:JSON.parse(result.stdout??"null")};
  receipt.runs.push(run);fs.writeFileSync(path.join(fixture,"original-public-before.json"),JSON.stringify(receipt,null,2)+"\n");
  assert.equal(run.error,null);assert.equal(run.signal,null);assert.equal(run.stderr,"");assert.equal(run.status,run.expectedStatus,entry.id);assert.deepEqual(run.actual,run.expected,entry.id);
 }
 console.log("24/24 complete public-before suppression packets matched authored vectors");
} finally { fs.rmSync(directory,{recursive:true,force:true}); }
