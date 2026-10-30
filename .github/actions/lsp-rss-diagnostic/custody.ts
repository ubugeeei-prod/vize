import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";

type JsonRecord = Record<string, unknown>;
type CommandResult = { stdout: string; stderr: string; status: number | null; signal: string | null };
type Image = { invoked: string; realpath: string; sha256: string; bytes: number };

function record(value: unknown): JsonRecord {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return Object.fromEntries(Object.entries(value));
}
function field(value: JsonRecord, key: string): string {
  const result = value[key];
  assert(typeof result === "string", key);
  return result;
}
function env(key: string): string {
  const value = process.env[key];
  assert(typeof value === "string" && value.length > 0, key);
  return value;
}
function readJson(file: string): unknown {
  const value: unknown = JSON.parse(fs.readFileSync(file, "utf8"));
  return value;
}
function hash(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}
function command(file: string, args: string[]): CommandResult {
  const result = spawnSync(file, args, { encoding: "utf8" });
  assert.equal(result.status, 0, `${file}: ${result.stderr}`);
  return { stdout: result.stdout, stderr: result.stderr, status: result.status, signal: result.signal };
}
function git(...args: string[]): string {
  return execFileSync("git", args, { encoding: "utf8" }).trim();
}
function image(invoked: string): Image {
  const realpath = fs.realpathSync(invoked);
  const bytes = fs.readFileSync(realpath);
  assert(fs.statSync(realpath).isFile());
  return { invoked, realpath, sha256: hash(bytes), bytes: bytes.length };
}
function executable(name: string): string {
  for (const directory of env("PATH").split(path.delimiter)) {
    const candidate = path.resolve(directory || ".", name);
    try {
      fs.accessSync(candidate, fs.constants.X_OK);
      if (fs.statSync(candidate).isFile()) return candidate;
    } catch { /* Continue the same PATH search used for a bare command. */ }
  }
  throw new Error(`Missing PATH executable: ${name}`);
}
function call(module: JsonRecord, key: string, args: unknown[]): unknown {
  const fn = module[key];
  assert(typeof fn === "function", key);
  const result: unknown = Reflect.apply(fn, undefined, args);
  return result;
}

const root = process.cwd();
const output = env("EVIDENCE");
const stage = process.argv[2];
assert(stage === "before" || stage === "after");
const roots = ["tests/performance", "tests/_helpers", "tests/tooling/support/lsp",
  "tests/differential", "tools/support/compat/davinci", "tools/config/vite-plus"];
const fixed = ["tests/_fixtures/vue-ecosystem-fixtures.json", "tests/package.json", "package.json",
  "pnpm-lock.yaml", "pnpm-workspace.yaml", "vite.config.ts", "vite-plus.config.ts",
  ".github/actions/check-vue-parity/action.yml"];
const names = execFileSync("git", ["ls-files", "-z", "--", ...roots, ...fixed],
  { encoding: "utf8" }).split("\0").filter(Boolean).sort();
const harness = names.map((file: string) => ({ path: file, sha256: hash(fs.readFileSync(file)) }));
for (const key of ["CORSA_PATH", "CORSA_EXECUTABLE", "TSGO_PATH", "TSGO_EXECUTABLE",
  "TSRS_PATH", "TSRS_EXECUTABLE", "VIZE_TEST_TSGO", "CORSA_BIN",
  "VIZE_LSP_CURRENT_BASELINE", "VIZE_LSP_REQUIRE_SOURCE_BUILD"]) assert(!process.env[key], key);
assert.equal(process.version, "v24.14.0");
assert.equal(os.platform(), "linux");
assert.equal(os.arch(), "x64");
assert.equal(os.cpus().length, 32);
assert.equal(env("CARGO_INCREMENTAL"), "0");
const typecheckModule: unknown = await import(pathToFileURL(path.join(root,
  "tests/_helpers/realworld-typecheck.ts")).href);
const selected = call(record(typecheckModule), "resolveTsgoBinary", []);
assert(typeof selected === "string");
const provider = image(selected);
assert.equal(provider.realpath, fs.realpathSync("node_modules/@typescript/typescript-linux-x64/lib/tsc"));
const runtimeModule: unknown = await import(pathToFileURL(path.join(root,
  "tests/tooling/support/lsp/runtime-environment.ts")).href);
const delivered = record(call(record(runtimeModule), "sourceLspRuntimeEnvironment", [root]));
assert.equal(fs.realpathSync(field(delivered, "CORSA_PATH")), provider.realpath);
assert.equal(fs.readFileSync(provider.realpath).subarray(0, 4).toString("hex"), "7f454c46");
const binary = image("target/ci/vize");
const receipt = record(readJson("target/ci/vize.differential-build.json"));
assert.equal(field(receipt, "sourceRevision"), env("SOURCE_SHA"));
assert.equal(field(receipt, "recipe"), "cargo build --profile ci -p vize --features legacy");
assert.equal(field(receipt, "binarySha256"), binary.sha256);

// The invoked PATH launcher is authoritative; package entries are recorded as
// delegation inputs, never substituted for an executable that was not invoked.
const vpLauncher = image(executable("vp"));
const vpVersion = command(vpLauncher.invoked, ["--version"]);
const selectedNodeRaw = command(vpLauncher.invoked, ["exec", "node", "--input-type=module", "-e",
  "process.stdout.write(JSON.stringify({execPath:process.execPath,version:process.version}))"]);
const selectedNode = record(JSON.parse(selectedNodeRaw.stdout));
assert.equal(field(selectedNode, "version"), "v24.14.0");
const vpNode = image(field(selectedNode, "execPath"));
const projectPackage = fs.realpathSync("node_modules/vite-plus");
const projectMetadata = record(readJson(path.join(projectPackage, "package.json")));
const projectEntry = path.resolve(projectPackage, field(record(projectMetadata["bin"]), "vp"));
assert(projectEntry.startsWith(projectPackage + path.sep));
const projectInputs = [path.join(projectPackage, "package.json"), projectEntry,
  path.join(projectPackage, "dist/bin.js")].map((file: string) => image(file));

const snapshot = {
  source: git("rev-parse", "HEAD"), tree: git("rev-parse", "HEAD^{tree}"),
  role: env("ROLE"), workflow: env("WORKFLOW_SHA"), harness,
  focusedOracleSources: ["tests.rs", "contracts.rs"].map((name: string) => {
    const file = `crates/vize_maestro/src/ide/workspace_symbols/${name}`;
    return { path: file, present: fs.existsSync(file), sha256: fs.existsSync(file) ? hash(fs.readFileSync(file)) : null };
  }),
  binary: { ...binary, version: command(binary.realpath, ["--version"]) },
  nativeImage: { ...provider, version: command(provider.realpath, ["--version"]) },
  vp: { launcher: vpLauncher, version: vpVersion, selectedNode: vpNode,
    selectedNodeVersion: field(selectedNode, "version"), projectDelegationInputs: projectInputs },
  tools: { node: process.version, rustc: command("rustc", ["-Vv"]), cargo: command("cargo", ["-V"]) },
  hardware: { platform: os.platform(), arch: os.arch(), cpus: os.cpus().length,
    model: os.cpus()[0]?.model ?? null, kernel: os.release() },
  env: { RAYON_NUM_THREADS: env("RAYON_NUM_THREADS"), VIZE_PERF_BUDGET_SCALE: env("VIZE_PERF_BUDGET_SCALE"),
    CARGO_INCREMENTAL: env("CARGO_INCREMENTAL"), RUSTFLAGS: process.env["RUSTFLAGS"] ?? null },
};
assert.equal(snapshot.source, env("SOURCE_SHA"));
const original = path.join(output, "before.json");
if (fs.existsSync(original)) assert.deepEqual(snapshot, readJson(original));
fs.writeFileSync(path.join(output, `${stage}.json`), JSON.stringify(snapshot, null, 2) + "\n");
