import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { SHIPPING_RECIPE } from "./shipping-adapter.ts";
type Obj = Record<string, unknown>;
const object = (value: unknown): Obj => { assert.ok(value && typeof value === "object" && !Array.isArray(value)); return value as Obj; };
const text = (value: unknown): string => { assert.equal(typeof value,"string"); return value as string; };
const hash = (bytes: Buffer): string => createHash("sha256").update(bytes).digest("hex");
export const BOUND_PATHS = [
  "crates/vize_maestro/src/server/diagnostic_publishing.rs",
  "crates/vize_maestro/src/server/diagnostic_publishing/complete.rs",
  "crates/vize_maestro/src/server/diagnostic_publishing/sync_feedback.rs",

  "crates/vize_maestro/src/ide/definition/service.rs",
  "crates/vize_maestro/src/ide/definition/service/private_setup_tests.rs",
  "crates/vize_maestro/src/ide/definition/service/private_setup_tests/original.rs",
  "crates/vize_maestro/src/ide/references.rs",
  "crates/vize_maestro/src/ide/references/canonical.rs",
  "crates/vize_maestro/src/ide/references/canonical/local_binding.rs",
  "crates/vize_maestro/src/ide/references/structural.rs",
  "crates/vize_maestro/src/ide/references/structural/snapshot.rs",
  "crates/vize_maestro/src/ide/references/structural/guard.rs",
  "crates/vize_maestro/src/ide/references/structural/template.rs",
  "crates/vize_maestro/src/ide/references/structural/edits.rs",
  "crates/vize_maestro/src/ide/references/structural/style.rs",
  "crates/vize_maestro/src/ide/references/structural/snapshot/registry.rs",
  "crates/vize_maestro/src/document/store/structural_navigation.rs",
  "crates/vize_maestro/src/ide/context.rs",
  "crates/vize_maestro/src/ide/definition/service/locations.rs",
  "crates/vize_maestro/src/ide/definition/service/import_target.rs",
  "crates/vize_maestro/src/ide/definition/service/import_target/alias.rs",
  "crates/vize_maestro/src/ide/definition/service/template_props.rs",
  "crates/vize_maestro/src/ide/corsa_support/rename_scope.rs",
  "tests/_fixtures/performance/lsp-dual-server/sources/App.v2.vue.txt",
  "npm/cli/content-mapper-helpers.d.cts",
  "npm/cli/content-mapper-helpers.d.mts",
  "npm/cli/content-mapper-helpers.d.ts",
  "npm/cli/generated/content-mapper-types.d.ts",
  "npm/cli/virtual-ts-globals.d.ts",
  "npm/cli/virtual-ts-patterns.d.ts",
  "npm/cli/virtual-ts-slots.d.ts",
  "npm/cli/virtual-ts.d.cts",
  "npm/cli/virtual-ts.d.mts",
  "npm/cli/virtual-ts.d.ts",
  "crates/vize_maestro/Cargo.toml",
  "crates/vize_canon/Cargo.toml",
  "Cargo.toml",
  "Cargo.lock",
  "rust-toolchain.toml",
  "package.json",
  "pnpm-lock.yaml",
  "pnpm-workspace.yaml",
  "npm/cli/package.json",
  "npm/cli/scripts/generate-content-mapper-types.ts",
  "crates/vize_maestro/src/test_support.rs",
  "tools/support/compat/github/full-rust-runtime.ts",
  "crates/vize_maestro/src/server/state/project_contexts.rs",
  "crates/vize_maestro/src/server/state/project_contexts/registry.rs",
  "crates/vize_maestro/src/server/state/corsa.rs",
  "crates/vize_maestro/src/ide/diagnostics.rs"
] as const;
export const SOURCE_PINS: Record<string, Record<string,string>> = {
  "51c31c6bfee4d0b6587b644c2aab39c6f3e83a1f": {
    ".github/workflows/lsp-scheduler-cost-carrier.yml": "4b803afdff687adfae2209554fa6301cf0d0a7241c21edbbf88539b16c2082f4",
    "crates/vize_maestro/src/server/initial_diagnostics.rs": "cb8495628c6974f4b4067185970a4a91e7769490a02951745c07e625a6be38de",
    "crates/vize_maestro/src/server/initial_diagnostics/pending.rs": "d341eedabbe966ae054fa28eb2a03ddfbb3272557f151bf7da93bc0fe1ed6fc6",
    "crates/vize_maestro/src/server/initial_diagnostics/preparation.rs": "8364c220e76f4c5a77c57b128965b14582b9934cfd82ffa6a53d9b5f0e54761d",
    "crates/vize_maestro/src/server/initial_diagnostics/queue.rs": "103706413c77bc4971e73e76937e20310f2be5fa7afa96a15fdc25da33d9a6ea",
    "crates/vize_maestro/src/server/initial_diagnostics/tests.rs": "44b4b21d5671b6e0df251a57774f7e713c84297a5f658e9abe60fb57479d6d86",
    "crates/vize_maestro/src/server/initial_diagnostics/worker.rs": "aadddef8201df7bc2f79afce24b230e4fc8b49514e8bd61efc62af9e25c9f80f",
    "tests/tooling/lsp-startup-local-interactive.test.ts": "9c26300dd9b1247409705dae306c65ca037a9448449a598fea830b5217752709"
  },
  "b52148128716418a08aa33b6d6e1c6e33961db56": {
    ".github/workflows/lsp-scheduler-cost-carrier.yml": "ff89a0dd80dbedea2cdade5d4de7f28117fa903003484468be5f6fb28b2489b8",
    "crates/vize_maestro/src/server/initial_diagnostics.rs": "75f8703e8b7e0b46761e4b1cf414e7816e61ab59e15b58694dda0a1e6a93c1c4",
    "crates/vize_maestro/src/server/initial_diagnostics/pending.rs": "62e5082c00ac99b996901c1ef3b92d79c756af54619585296b97a5f59fb1488f",
    "crates/vize_maestro/src/server/initial_diagnostics/preparation.rs": "3bc266e17890fa01ebc2cd97ac40ce822ea5b0be695504f5cda577fdd138ec66",
    "crates/vize_maestro/src/server/initial_diagnostics/queue.rs": "4289e9cec8ba76f027bca7155e345dacc93fa3617915690c101bf9972ff9d26e",
    "crates/vize_maestro/src/server/initial_diagnostics/tests.rs": "5a572166d9f8f58629815091539a54f3af984ccadb332266ec3231924a6121ed",
    "crates/vize_maestro/src/server/initial_diagnostics/tests/prepared_readiness.rs": "66506c7c0c47599a62c35d3a7618c4f7997c8cf063bb1464024df8f499b1fe74",
    "crates/vize_maestro/src/server/initial_diagnostics/worker.rs": "6db08598b8431df24dad38ca9bc6a9b178b4c5272b1446571e90cd164ec4940e",
    "tests/tooling/lsp-startup-local-interactive.test.ts": "262eee304961cf9f02f1517bd9ab183e377bed5c570dd45b8f2bc7b7ec86f664"
  }
};
export const CLOCK_PREFIX = "server::initial_diagnostics::tests::prepared_readiness::";
export const CLOCK_TESTS = ["current_completed_preparation_advances_only_the_same_open_slot", "stale_source_reopen_environment_and_retirement_keep_the_original_grace", "older_completion_never_advances_a_replaced_or_reopened_job", "sync_feedback_and_actual_preparation_cannot_be_overtaken", "retained_retries_keep_their_original_deadline_after_preparation", "edit_deadlines_missing_slots_and_elapsed_grace_are_unchanged"].map(name=>CLOCK_PREFIX+name);
export const NATIVE_TEST = "tests/tooling/lsp-startup-local-interactive.test.ts";
export function assertSources(repo: string): void {
  for (const [revision,pins] of Object.entries(SOURCE_PINS)) for (const [file,digest] of Object.entries(pins)) assert.equal(hash(execFileSync("git",["show",`${revision}:${file}`],{cwd:repo})),digest);
}
export function checkClockExecution(stdout: string): void {
  const clean = stdout.replace(/\u001b\[[0-9;]*m/g, "");
  assert.deepEqual([...clean.matchAll(/^test (\S+) \.\.\./gm)].map(row=>row[1]).sort(),[...CLOCK_TESTS].sort());
  const rows=[...clean.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured;/g)];
  assert.equal(rows.length,1); assert.deepEqual(rows[0]!.slice(1),["6","0","0","0"]);
}
export function checkNativeExecution(stdout: string): void {
  const expected=["cold CSS hover and exact local completion stay prompt while native preparation owns its scope", "JSX opt-out keeps SFC-looking strings inert and opt-in keeps native type hover"];
  assert.deepEqual([...stdout.matchAll(/^ok \d+ - (.+)$/gm)].map(row=>row[1]),expected); assert.doesNotMatch(stdout,/^not ok /m);
  for (const [label,count] of Object.entries({tests:2,suites:0,pass:2,fail:0,cancelled:0,skipped:0,todo:0})) {
    const rows=[...stdout.matchAll(new RegExp(`^# ${label} (\\d+)$`,"gm"))]; assert.equal(rows.length,1); assert.equal(rows[0]![1],String(count));
  }
}
export const SDK_CRATES = ["corsa", "corsa_client", "corsa_core", "corsa_jsonrpc", "corsa_lsp", "corsa_orchestrator", "corsa_runtime"];
export function libraryArtifact(values: Record<string, unknown>[], target: string, repo: string): Record<string, unknown> {
  assert.equal(values.filter(row => row["reason"] === "build-finished" && row["success"] === true).length, 1);
  const artifacts = values.filter(row => row["reason"] === "compiler-artifact");
  const libraries = artifacts.filter(row => object(row["target"])["name"] === "vize_maestro" && object(row["profile"])["test"] === true && typeof row["executable"] === "string");
  assert.equal(libraries.length, 1); const library = libraries[0]; assert(library !== undefined);
  const features = library["features"]; assert(Array.isArray(features) && features.includes("native"));
  assert.ok(artifacts.every(row => { const f = row["features"]; return Array.isArray(f) && !f.includes("profiling") && !f.includes("micro-profiling"); }));
  const meta = object(library["target"]); assert.deepEqual(meta["kind"], ["lib"]);
  assert.equal(library["manifest_path"], path.join(repo, "crates/vize_maestro/Cargo.toml"));
  assert.equal(meta["src_path"], path.join(repo, "crates/vize_maestro/src/lib.rs"));
  assert.equal(text(library["package_id"]).split("#")[0], "path+" + pathToFileURL(path.join(repo, "crates/vize_maestro")).href);
  const executable = text(library["executable"]); assert.ok(path.isAbsolute(executable));
  const relative = path.relative(target, executable); assert.ok(relative.startsWith("ci/deps/") && !relative.split(path.sep).includes(".."));
  const sdk = artifacts.filter(row => SDK_CRATES.includes(text(object(row["target"])["name"])));
  assert.deepEqual([...new Set(sdk.map(row => text(row["package_id"]).split("#")[1]))].sort(), SDK_CRATES.map(name => name + "@2.0.0-beta.4").sort());
  assert.ok(sdk.every(row => text(row["package_id"]).startsWith("registry+") && JSON.stringify(object(row["target"])["kind"]) === '["lib"]'));
  return library;
}

export function sdkArtifacts(rows: Record<string, unknown>[]): Record<string, unknown>[] {
  const artifacts = rows.filter(row => row["reason"] === "compiler-artifact");
  assert.ok(artifacts.every(row => { const f = row["features"]; return Array.isArray(f) && !f.includes("profiling") && !f.includes("micro-profiling"); }));
  const sdk = artifacts.filter(row => SDK_CRATES.includes(text(object(row["target"])["name"])));
  assert.deepEqual([...new Set(sdk.map(row => text(row["package_id"]).split("#")[1]))].sort(), SDK_CRATES.map(name => name + "@2.0.0-beta.4").sort());
  assert.ok(sdk.every(row => text(row["package_id"]).startsWith("registry+") && JSON.stringify(object(row["target"])["kind"]) === '["lib"]'));
  return sdk;
}
export function checkCapacity(value: {freeDiskBytes:number;physicalMemoryBytes:number;cpuCount:number}, minimumGiB:number): void {
  assert.ok(Number.isFinite(value.freeDiskBytes) && value.freeDiskBytes>=minimumGiB*2**30,"distinct build/measurement disk safety floor");
  assert.ok(value.physicalMemoryBytes>=32*2**30); assert.ok(value.cpuCount>=32);
}
export function capacity(repo: string, phase: string, minimumGiB: number, destination: string): Obj {
  const stat=fs.statfsSync(repo); const actual={freeDiskBytes:stat.bavail*stat.bsize,physicalMemoryBytes:os.totalmem(),cpuCount:os.cpus().length,availableParallelism:os.availableParallelism()};
  const read=(name:string):string|null=>{try{return fs.readFileSync(name,"utf8");}catch{return null;}};
  const value={phase,...actual,minimumFreeGiB:minimumGiB,scope:"capacity safety only; no RSS budget relaxation or PSS sampling",procMeminfo:read("/proc/meminfo"),procSelfCgroup:read("/proc/self/cgroup"),cgroupMemoryMax:read("/sys/fs/cgroup/memory.max"),cgroupCpuMax:read("/sys/fs/cgroup/cpu.max"),cgroupCpuSetEffective:read("/sys/fs/cgroup/cpuset.cpus.effective"),cgroupMemoryCurrent:read("/sys/fs/cgroup/memory.current"),cgroupMemoryPeak:read("/sys/fs/cgroup/memory.peak"),cgroupScope:"mount-root counters; isolation UNKNOWN, charged cache/kernel are not RSS"};
  fs.writeFileSync(destination,JSON.stringify(value,null,2)+"\n",{flag:"wx",mode:0o600});
  checkCapacity(actual,minimumGiB); return value;
}
export function verifyObservation(value: unknown, source:string,image:string): Obj {
  const row=object(value); assert.equal(row.state,"process-closed"); assert.equal(row.sourceRevision,source);
  assert.equal(object(row.binary).binarySha256,image); assert.equal(object(row.binary).sourceRevision,source); assert.equal(object(row.buildReceipt).recipe,SHIPPING_RECIPE); assert.equal(object(row.buildReceipt).sourceRevision,source);
  assert.ok(Array.isArray(row.sourceWitnesses)); assert.ok(row.sourceWitnesses.some(value => {const witness=object(value);return witness.path===NATIVE_TEST && witness.sha256===SOURCE_PINS[source]?.[NATIVE_TEST];}));
  assert.deepEqual(row.process,{exitStatus:0,signal:null,error:null}); const streams=object(row.streams); assert.deepEqual(Object.keys(streams).sort(),["client","server","stderr"]);
  for (const name of ["client","server","stderr"]) {const stream=object(streams[name]); assert.equal(stream.path,`${name}.bin`); assert.equal(stream.truncated,false); assert.equal(stream.capturedBytes,stream.observedBytes); assert.ok(Number(stream.capturedBytes)>=0); if(name!=="stderr") assert.equal(object(stream.framing).state,"complete");}
  return row;
}
export function preserveNativeCaptures(repo:string,dir:string,source:string,image:string): Obj {
  const root=path.join(repo,"target/differential/lsp-sessions"); const destination=path.join(dir,"startup-native-sessions");
  if(!fs.existsSync(root)) {
    fs.writeFileSync(path.join(dir,"startup-capture-validation.json"),JSON.stringify({complete:false,rows:[],errors:["no capture directory"],expected:3},null,2)+"\n",{flag:"wx",mode:0o600});
    assert.fail("source native raw capture directory is required");
  }
  fs.renameSync(root,destination); // Preserve every actual byte before any observation verdict.
  const names=fs.readdirSync(destination); const rows:Obj[]=[]; const errors:string[]=[];
  for(const name of names) try {
    assert.ok(fs.lstatSync(path.join(destination,name)).isDirectory()); const file=path.join(destination,name,"observation.json");
    const row=verifyObservation(JSON.parse(fs.readFileSync(file,"utf8")),source,image);
    for(const [stream,value] of Object.entries(object(row.streams))) {const bytes=fs.readFileSync(path.join(destination,name,`${stream}.bin`)); const metadata=object(value); assert.equal(bytes.length,metadata.capturedBytes); assert.equal(hash(bytes),metadata.sha256);}
    rows.push({directory:name,observationSha256:hash(fs.readFileSync(file))});
  } catch(error) {errors.push(error instanceof Error?error.message:String(error));}
  fs.writeFileSync(path.join(dir,"startup-capture-validation.json"),JSON.stringify({rows,errors,expected:3},null,2)+"\n",{flag:"wx",mode:0o600});
  assert.deepEqual(errors,[]); assert.equal(rows.length,3,"actual first session plus both JSX modes"); return {complete:true,rows};
}
