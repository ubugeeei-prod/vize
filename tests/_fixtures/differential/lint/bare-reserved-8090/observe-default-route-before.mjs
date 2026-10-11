import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sha256, wholeJson } from "../../../../tooling/lint-default-correctness-reference.mjs";

const fixture = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(fixture, "../../../../..");
const binary = process.argv[2];
assert.equal(sha256(fs.readFileSync(binary)), "9ad7cf956351265a152a9bbfa08b8c50c85dadacef66529474bbb6170267e254");
assert.match(spawnSync(binary, ["--version"], { encoding: "utf8" }).stdout, /^vize 0\.443\.0/);
const corpus = JSON.parse(fs.readFileSync(path.join(fixture, "cases.json")));
const companions = JSON.parse(fs.readFileSync(path.join(fixture, "default-route-companions.json")));
const projection = JSON.parse(fs.readFileSync(path.join(root, "tests/_fixtures/n8n-cli-adoption.json")));
const receipt = {
  schema: "vize.lint.default-route-public-before", version: 1,
  baseline: corpus.beforeBaseline,
  custody: ["cases.json", "default-route-companions.json", "observe-default-route-before.mjs"].map((name) => ({path: name, sha256: sha256(fs.readFileSync(path.join(fixture, name)))})),
  runs: [],
};
const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-default-route-before-"));
try {
  for (const [index, entry] of corpus.cases.entries()) {
    const companion = companions.cases[index];
    assert.equal(companion.id, entry.id);
    const bytes = fs.readFileSync(path.join(fixture, entry.filename + ".fixture"));
    assert.equal(sha256(bytes), entry.sourceSha256);
    const routes = [
      ...[null, "recommended", "essential", "ecosystem"].map((preset) => ({route: preset ?? "default", preset, config: null, diagnostics: companion.beforeDefaultDiagnostics})),
      {route: "incremental", preset: "incremental", config: null, diagnostics: []},
      {route: "explicit-off", preset: null, config: JSON.stringify({linter:{rules:{[corpus.rule]:"off"}}}) + "\n", diagnostics: companion.diagnostics},
      {route: "n8n-projection", preset: null, config: JSON.stringify({linter:projection.linter}) + "\n", diagnostics: []},
    ];
    for (const {route, preset, config, diagnostics} of routes) {
      fs.writeFileSync(path.join(directory, entry.filename), bytes);
      if (config) fs.writeFileSync(path.join(directory, "vize.config.json"), config);
      const argv = ["lint", ...(config ? ["-c", "vize.config.json"] : ["--no-config"]), ...(preset ? ["--preset", preset] : []), "--format", "json", "--locale", "en", "--help-level", "none", entry.filename];
      const result = spawnSync(binary, argv, {cwd:directory,encoding:"utf8", timeout:60000,maxBuffer:4*1024*1024});
      const run = {id:entry.id,route,argv,config,input:{text:bytes.toString("utf8"),sha256:sha256(bytes)},status:result.status,signal:result.signal,error:result.error?{name:result.error.name,message:result.error.message}:null,stdout:result.stdout??"",stderr:result.stderr??"",expected:wholeJson(entry,diagnostics),expectedStatus:diagnostics.length?1:0,actual:JSON.parse(result.stdout??"null")};
      receipt.runs.push(run);
      fs.writeFileSync(path.join(fixture, "default-route-public-before.json"),JSON.stringify(receipt,null,2)+"\n");
      assert.equal(run.error,null);assert.equal(run.signal,null);assert.equal(run.stderr,"");assert.equal(run.status,run.expectedStatus,entry.id+"/"+route);
      assert.deepEqual(run.actual,run.expected,entry.id+"/"+route);
    }
  }
  console.log("112/112 complete public default-route packets matched source-authored companions");
} finally { fs.rmSync(directory,{recursive:true,force:true}); }
