import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";

const BASE = "abddb5fc25a2fc60dbedb3affcc22ec7a41803e7";
const TRACE = ["crates/vize_canon/src/corsa_bridge/preparation_trace.rs", "crates/vize_canon/src/corsa_bridge/bridge.rs",
  "crates/vize_canon/src/corsa_bridge/bridge/language_features.rs", "crates/vize_maestro/src/server/handlers.rs",
  "crates/vize_maestro/src/server/handlers/navigation_trace_formatter.rs", "crates/vize_maestro/src/ide/references/canonical.rs",
  "crates/vize_maestro/src/ide/references/structural.rs", "crates/vize_maestro/src/ide/rename/canonical.rs",
  "crates/vize_maestro/src/ide/rename/canonical/execute.rs",
  "crates/vize_maestro/src/ide/corsa_support/canonical/semantic_links/binding_roles.rs",
  "crates/vize_canon/src/sfc_typecheck/runner.rs"].sort();
const HELPERS = ["controller.ts", "log-join.ts", "custody.ts", "controls/log-join-controls.test.ts"].map(file => ".github/actions/lsp-navigation-trace/" + file);
const WORKFLOW = ".github/workflows/lsp-current-baseline.yml";
const ROOTS = ["tests/performance", "tests/_helpers", "tests/tooling/support/lsp", "tests/differential", "tools/support/compat/davinci", "tools/config/vite-plus",
  "tests/_fixtures/vue-ecosystem-fixtures.json", "tests/package.json", "package.json", "pnpm-lock.yaml", "pnpm-workspace.yaml", "vite.config.ts", "vite-plus.config.ts", ".github/actions/check-vue-parity/action.yml"];
const sha = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
function env(name: string): string { const value = process.env[name]; assert(value !== undefined && value.length > 0, name); return value; }
function object(value: unknown): Record<string, unknown> { assert(value !== null && typeof value === "object" && !Array.isArray(value)); return Object.fromEntries(Object.entries(value)); }
function git(...args: string[]): Buffer { return execFileSync("git", args, { maxBuffer: 8 * 1024 * 1024 }); }
function text(...args: string[]): string { return git(...args).toString("utf8").trim(); }
function names(...args: string[]): string[] { return text(...args).split("\n").filter(Boolean).sort(); }
function verifyParent(raw: string, expected: string): void {
  const separator = raw.indexOf("\n\n"); assert(separator > 0, "commit header separator required");
  const headers = raw.slice(0, separator).split("\n"); assert.match(headers[0] ?? "", /^tree [a-f0-9]{40}$/u);
  const parents = headers.filter(line => line === "parent" || line.startsWith("parent "));
  assert.deepEqual(parents, ["parent " + expected], "one immutable expected commit parent required");
}
function assertOrdinaryFeatures(rows: Record<string, unknown>[]): void {
  const targets = ["vize", "vize_canon", "vize_l0", "vize_carton"];
  const selected = rows.filter(row => row["reason"] === "compiler-artifact"
    && targets.includes(String(object(row["target"])["name"])));
  assert.deepEqual([...new Set(selected.map(row => object(row["target"])["name"]))].sort(), targets.slice().sort());
  for (const row of selected) {
    const features = row["features"]; assert(Array.isArray(features) && features.every(value => typeof value === "string"));
    assert(!features.includes("profiling") && !features.includes("micro-profiling"), "ordinary allocator/features required");
  }
}
if (process.argv[2] === "--ordinary-controls") {
  const rows = ["vize", "vize_canon", "vize_l0", "vize_carton"].map(name => ({ reason: "compiler-artifact", target: { name }, features: ["default"] }));
  assertOrdinaryFeatures(rows);
  for (const name of ["vize", "vize_canon", "vize_l0", "vize_carton"]) {
    for (const feature of ["profiling", "micro-profiling"]) {
      assert.throws(() => assertOrdinaryFeatures(rows.map(row => row.target.name === name ? { ...row, features: [feature] } : row)));
    }
    assert.throws(() => assertOrdinaryFeatures(rows.filter(row => row.target.name !== name)));
  }
  process.stdout.write(JSON.stringify({ schema: "vize.navigation-ordinary-controls", cases: 13, passed: 13, qualificationCredit: 0 }) + "\n");
  process.exit(0);
}
if (process.argv[2] === "--parent-controls") {
  const parent = "a".repeat(40), tree = "tree " + "b".repeat(40), single = tree + "\nparent " + parent;
  verifyParent(single + "\nauthor fixture\n\nparent " + "c".repeat(40), parent);
  verifyParent(single + "\ngpgsig fixture\n parent " + "c".repeat(40) + "\n\nmessage", parent);
  for (const raw of [tree + "\n\nroot", single + "\nparent " + parent + "\n\nmerge", tree + "\nparent invalid\n\nmessage",
    single + "\nmessage-without-header-separator", "blob fixture\nparent " + parent + "\n\nmessage"]) assert.throws(() => verifyParent(raw, parent));
  process.stdout.write(JSON.stringify({ schema: "vize.navigation-parent-controls", cases: 7, passed: 7, qualificationCredit: 0 }) + "\n");
  process.exit(0);
}
function write(name: string, value: unknown): void { fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n", { flag: "wx", mode: 0o600 }); }
function metadata(file: string) { const stat = fs.statSync(file, { bigint: true }); assert(stat.isFile()); return {
  dev: String(stat.dev), ino: String(stat.ino), bytes: Number(stat.size), mtimeNs: String(stat.mtimeNs), ctimeNs: String(stat.ctimeNs) }; }
function image(file: string, elf = true) {
  const realpath = fs.realpathSync(file), before = metadata(realpath), bytes = fs.readFileSync(realpath);
  assert.deepEqual(metadata(realpath), before); assert(Number.isSafeInteger(before.bytes) && before.bytes > 0);
  if (elf) assert.equal(bytes.subarray(0, 4).toString("hex"), "7f454c46"); return { realpath, sha256: sha(bytes), ...before };
}
function executable(command: string): string {
  for (const entry of env("PATH").split(path.delimiter)) {
    const candidate = path.resolve(entry, command);
    try { fs.accessSync(candidate, fs.constants.X_OK); if (fs.statSync(candidate).isFile()) return candidate; } catch { /* Try the next PATH entry. */ }
  }
  throw new Error("invoked PATH launcher missing");
}
const mode = process.argv[2]; assert(mode === "before" || mode === "after");
const source = env("SOURCE_SHA"), workflow = env("WORKFLOW_SHA"), output = fs.realpathSync(env("EVIDENCE"));
assert.match(source, /^[a-f0-9]{40}$/); assert.match(workflow, /^[a-f0-9]{40}$/); assert.equal(env("EXECUTED_WORKFLOW_SHA"), workflow);
assert.equal(process.version, "v24.14.0"); assert.equal(os.platform(), "linux"); assert.equal(os.arch(), "x64"); assert.equal(os.cpus().length, 32);
assert.equal(env("CARGO_INCREMENTAL"), "0"); assert.equal(text("rev-parse", "HEAD"), source);
assert.equal(text("status", "--porcelain", "--untracked-files=no"), "");
verifyParent(git("cat-file", "-p", source).toString("utf8"), BASE);
verifyParent(git("cat-file", "-p", workflow).toString("utf8"), source);
assert.deepEqual(names("diff", "--name-only", BASE, source), TRACE);
assert.deepEqual(names("diff", "--name-only", source, workflow), [...HELPERS, WORKFLOW].sort());
const traceFiles = TRACE.map(file => {
  const bytes = fs.readFileSync(file); assert.deepEqual(bytes, git("show", source + ":" + file));
  assert(bytes.toString("utf8").split("\n").length - 1 <= 500, file); return { path: file, sha256: sha(bytes) };
});
const helpers = HELPERS.map(file => {
  const bytes = fs.readFileSync(path.join(output, file.split("/").slice(3).join("/")));
  assert.deepEqual(bytes, git("show", workflow + ":" + file)); return { path: file, sha256: sha(bytes) };
});
const harness = names("ls-tree", "-r", "--name-only", source, "--", ...ROOTS).map(file => {
  const bytes = fs.readFileSync(file); assert.deepEqual(bytes, git("show", source + ":" + file)); assert.deepEqual(bytes, git("show", BASE + ":" + file));
  return { path: file, sha256: sha(bytes) };
});
assert.equal(process.env["LSP_DUAL_VIZE_PROFILE_DIR"], undefined);
assert.equal(process.env["LSP_DUAL_VUE_CPU_PROFILE_DIR"], undefined);
const serverImage = image("target/ci/vize"), nodeImage = image(process.execPath), nativeImage = image(env("CORSA_PATH"));
assert.equal(serverImage.realpath, fs.realpathSync(env("VIZE_LSP_BIN")));
const receipt = object(JSON.parse(fs.readFileSync("target/ci/vize.differential-build.json", "utf8")));
assert.equal(receipt["sourceRevision"], source); assert.equal(receipt["recipe"], "cargo build --profile ci -p vize");
assert.equal(receipt["binarySha256"], serverImage.sha256); assert.equal(receipt["binaryPath"], "target/ci/vize");
assert.equal(fs.readFileSync(path.join(output, "first-image.sha256"), "utf8").trim().split(/\s+/)[0], serverImage.sha256);
const cargo = fs.readFileSync(path.join(output, "first-cargo.jsonl")); assert(cargo.length <= 8 * 1024 * 1024);
const rows = cargo.toString("utf8").split("\n").filter(Boolean).map(line => object(JSON.parse(line)));
assert.equal(rows.filter(row => row["reason"] === "build-finished" && row["success"] === true).length, 1);
const cli = rows.filter(row => row["reason"] === "compiler-artifact" && object(row["target"])["name"] === "vize"
  && typeof row["executable"] === "string" && fs.realpathSync(row["executable"]) === serverImage.realpath);
assert.equal(cli.length, 1); const features = cli[0]?.["features"]; assert(Array.isArray(features));
assertOrdinaryFeatures(rows);
const vpInvoked = executable("vp"), vpLauncher = { invoked: vpInvoked, ...image(vpInvoked, false) };
const vpVersion = execFileSync(vpLauncher.invoked, ["--version"], { encoding: "utf8" }).trim();
const toolchain = { rustc: execFileSync("rustc", ["-Vv"], { encoding: "utf8" }), cargo: execFileSync("cargo", ["-V"], { encoding: "utf8" }) };
const snapshot = { schema: "vize.navigation-trace-custody", version: 1, mode, qualificationCredit: 0, baseSource: BASE, source, workflow,
  sourceTree: text("rev-parse", source + "^{tree}"), traceFiles, helpers, harness, harnessSha256: sha(Buffer.from(JSON.stringify(harness))),
  serverImage, nodeImage, nativeImage, vpLauncher, receipt, firstCargoSha256: sha(cargo), actualCliFeatures: features, toolchain, vpVersion,
  runtime: { runId: env("GITHUB_RUN_ID"), runAttempt: env("GITHUB_RUN_ATTEMPT"), runnerImageOS: process.env["ImageOS"] ?? null,
    runnerImageVersion: process.env["ImageVersion"] ?? null, incremental: env("CARGO_INCREMENTAL"), rustFlags: process.env["RUSTFLAGS"] ?? null },
  scope: "Diagnostic source I/ordinary-ci CLI with trace overhead; not uninstrumented B, release performance or same-image library-test qualification. No new benchmark RPC." };
if (mode === "before") {
  const units = fs.readFileSync(path.join(output, "formatter-test.jsonl"), "utf8");
  assert.match(units, /running 1 test/u); assert.match(units, /test result: ok\. 1 passed; 0 failed; 0 ignored/u);
  const unitArtifacts = units.split("\n").filter(line => line.startsWith("{")).map(line => object(JSON.parse(line)))
    .filter(row => row["reason"] === "compiler-artifact" && object(row["target"])["name"] === "vize_maestro"
      && object(row["profile"])["test"] === true && typeof row["executable"] === "string");
  assert.equal(unitArtifacts.length, 1); const unitPath = unitArtifacts[0]?.["executable"]; assert(typeof unitPath === "string");
  const unitImage = image(unitPath);
  const formatter = fs.readFileSync(path.join(output, "formatter.raw")); assert(formatter.length > 0 && formatter.length <= 256 * 1024);
  write("formatter-evidence.json", { rawBytes: formatter.length, sha256: sha(formatter), unitImage, measuredCliImage: serverImage,
    imagesByteEqual: unitImage.sha256 === serverImage.sha256, sameImageProtocolQualification: false,
    scope: "Actual locked Full formatter library fixture; distinct from measured first CLI image." });
  write("before.json", snapshot);
  const raw = path.join(output, "observed", "raw"); assert.equal(fs.realpathSync(raw), raw); assert.equal(fs.readdirSync(raw).length, 0);
  write("config.json", { schema: "vize.navigation-trace-config", version: 1, repoRoot: fs.realpathSync("."), outputRoot: raw,
    sourceRevision: source, workflowRevision: workflow, runId: env("GITHUB_RUN_ID"), runAttempt: env("GITHUB_RUN_ATTEMPT"), serverImage, nodeImage,
    wireSourceSha256: sha(fs.readFileSync("tests/performance/support/dual-lsp-wire.ts")), benchmarkSourceSha256: sha(fs.readFileSync("tests/performance/lsp-dual-server-benchmark.ts")) });
} else {
  const before = object(JSON.parse(fs.readFileSync(path.join(output, "before.json"), "utf8")));
  for (const key of ["source", "workflow", "sourceTree", "traceFiles", "helpers", "harnessSha256", "serverImage", "nodeImage", "nativeImage", "vpLauncher", "receipt", "firstCargoSha256", "actualCliFeatures", "toolchain", "vpVersion", "runtime"] as const)
    assert.deepEqual(snapshot[key], before[key], key);
  write("after.json", snapshot);
}
