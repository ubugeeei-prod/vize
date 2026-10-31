import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";
import { sources, validateSourceAuthority } from "./runtime-driver.ts";

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
const candidate = "106251f048c1dc39cd50c0c23e93420963e4b163";
const selected = validateSourceAuthority(process.env["ROLE"], process.env["SOURCE_SHA"], git("rev-parse", "HEAD"));
const profilingSourceSha256 = "b3244db3b1595f04698d9720bea58f3690df9302190814ba61f9f307b0ce9934";
assert.equal(hash(fs.readFileSync("crates/vize_maestro/src/ide/workspace_symbols/allocation_observation.rs")), profilingSourceSha256);
const roots = ["tests/performance", "tests/_helpers", "tests/tooling/support/lsp",
  "tests/differential", "tools/support/compat/davinci", "tools/config/vite-plus"];
const fixed = ["tests/_fixtures/vue-ecosystem-fixtures.json", "tests/package.json", "package.json",
  "pnpm-lock.yaml", "pnpm-workspace.yaml", "vite.config.ts", "vite-plus.config.ts",
  ".github/actions/check-vue-parity/action.yml"];
const names = execFileSync("git", ["ls-files", "-z", "--", ...roots, ...fixed],
  { encoding: "utf8" }).split("\0").filter(Boolean).sort();
const harness = names.map((file: string) => ({ path: file, sha256: hash(fs.readFileSync(file)) }));
assert.equal(harness.filter((entry) => entry.path === "tools/support/compat/davinci/lib/lsp-profile-launch.ts").length, 1);
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
const selectedProvider = call(record(typecheckModule), "resolveTsgoBinary", []);
assert(typeof selectedProvider === "string");
const provider = image(selectedProvider);
assert.equal(provider.realpath, fs.realpathSync("node_modules/@typescript/typescript-linux-x64/lib/tsc"));
const runtimeModule: unknown = await import(pathToFileURL(path.join(root,
  "tests/tooling/support/lsp/runtime-environment.ts")).href);
const delivered = record(call(record(runtimeModule), "sourceLspRuntimeEnvironment", [root]));
assert.equal(fs.realpathSync(field(delivered, "CORSA_PATH")), provider.realpath);
assert.equal(fs.readFileSync(provider.realpath).subarray(0, 4).toString("hex"), "7f454c46");
const binary = image("target/ci/vize");
assert.equal(fs.readFileSync(path.join(output, "source-build-image.sha256"), "utf8"), `${binary.sha256}  target/ci/vize\n`);
const nativeTest = fs.readFileSync(path.join(output, "native-profile-test.stdout"));
const libraryOracle = "ide::workspace_symbols::tests::macro_declaration_range_targets_quoted_name_content";
const nativeTestText = nativeTest.toString("utf8");
assert.equal(nativeTestText.split("\n").filter((line) => line === `test ${libraryOracle} ... ok`).length, 1);
assert.equal(nativeTestText.match(/^test result: ok\. 1 passed; 0 failed; 0 ignored;/gm)?.length, 1);
const nativeTestImage = fs.readFileSync(path.join(output, "native-test-image.sha256"), "utf8");
assert.match(nativeTestImage, /^[a-f0-9]{64}  target\/ci\/vize\n$/);
const nativeTestSha256 = nativeTestImage.slice(0, 64);
const nativeTestIdentity = image(path.join(env("CARGO_TARGET_DIR"), "native-profile-test-vize"));
assert.equal(nativeTestIdentity.sha256, nativeTestSha256);
const nativeTestMetadata = fs.statSync(nativeTestIdentity.realpath, { bigint: true });
const buildArguments = fs.readFileSync(path.join(output, "source-build-argv.txt"), "utf8").trimEnd().split("\n");
assert.deepEqual(buildArguments, ["cargo", "build", "--profile", "ci", "-p", "vize", "--features",
  "legacy,profiling,vize_maestro/micro-profiling", "--message-format=json-render-diagnostics"]);
const buildLog = path.join(output, "source-build.jsonl");
const cargoRecords: JsonRecord[] = fs.readFileSync(buildLog, "utf8").split("\n").filter(Boolean)
  .map((line: string): JsonRecord => { const value: unknown = JSON.parse(line); return record(value); });
const artifacts = cargoRecords.filter((entry: JsonRecord) => entry["reason"] === "compiler-artifact");
const measuredArtifacts = artifacts.filter((entry: JsonRecord) => typeof entry["executable"] === "string"
  && fs.realpathSync(entry["executable"]) === binary.realpath);
assert.equal(measuredArtifacts.length, 1, "Cargo must report exactly the measured executable");
const measured = measuredArtifacts[0]; assert(measured !== undefined && measured["fresh"] === false);
const features = measured["features"]; assert(Array.isArray(features));
for (const feature of ["legacy", "profiling", "maestro"]) assert(features.includes(feature));
const maestroArtifacts = artifacts.filter((entry: JsonRecord) => record(entry["target"])["name"] === "vize_maestro");
assert.equal(maestroArtifacts.length, 1);
const maestro = maestroArtifacts[0]; assert(maestro !== undefined && maestro["fresh"] === false);
assert(Array.isArray(maestro["features"]) && maestro["features"].includes("micro-profiling"));
const finished = cargoRecords.filter((entry: JsonRecord) => entry["reason"] === "build-finished");
assert.equal(finished.length, 1); assert.equal(finished[0]?.["success"], true);
const build = { schema: "vize.lsp.allocation-diagnostic-build", version: 1, qualificationCredit: 0,
  argv: buildArguments, cargoLogSha256: hash(fs.readFileSync(buildLog)), binarySha256: binary.sha256,
  artifact: { executable: measured["executable"], features, profile: measured["profile"], fresh: measured["fresh"] },
  maestroArtifact: { features: maestro["features"], profile: maestro["profile"], fresh: maestro["fresh"] },
  focusedLibraryOracle: { selector: libraryOracle, packetPairs: 848, positiveAnchors: 10, retainedTrackerStages: 7,
    argv: ["cargo", "test", "--profile", "ci", "-p", "vize_maestro", "--features", "legacy,micro-profiling",
      "--lib", libraryOracle, "--", "--exact", "--nocapture"], stdoutSha256: hash(nativeTest),
    stderrSha256: hash(fs.readFileSync(path.join(output, "native-profile-test.stderr"))), passed: 1, failed: 0, ignored: 0,
    postLibraryCliImage: { ...nativeTestIdentity, dev: nativeTestMetadata.dev.toString(), ino: nativeTestMetadata.ino.toString(),
      mode: nativeTestMetadata.mode.toString(), mtimeNs: nativeTestMetadata.mtimeNs.toString(), ctimeNs: nativeTestMetadata.ctimeNs.toString() },
    postLibraryCliByteIdenticalToMeasuredImage: nativeTestSha256 === binary.sha256,
    sameElfCliStdioProof: false,
    scope: "This is a focused library oracle, never CLI stdio proof even if the post-library CLI image equals the measured image. The post-library CLI remains in the private target; only its hash/metadata are exported. The first actual Cargo-build image is restored and hash-checked. The original churn terminal profile with all 56 counters and 10 spans plus proc image custody validates the measured CLI/export separately." },
  profileManifestSha256: hash(fs.readFileSync("Cargo.toml")),
  reason: "Same underlying mimalloc with existing profiling wrapper; separately instrumented diagnostic ELF, no ordinary legacy build-receipt credit" };

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
  sourceAuthority: { candidate, ...sources, selected, profilingSourceSha256 },
  candidateSources: ["crates/vize_croquis/src/script_parser.rs",
    "crates/vize_croquis/src/script_parser/contracts.rs", "crates/vize_croquis/src/script_parser/parse.rs",
    "crates/vize_maestro/src/ide/workspace_symbols.rs", "crates/vize_maestro/src/ide/workspace_symbols/contracts.rs",
    "crates/vize_maestro/src/ide/workspace_symbols/sources.rs", "crates/vize_maestro/src/ide/workspace_symbols/tests.rs",
    "crates/vize_maestro/src/server/state/workspace_project_files/symbols.rs"].map((file: string) =>
    ({ path: file, sha256: hash(fs.readFileSync(file)) })),
  diagnosticSources: ["crates/vize/src/main.rs", "crates/vize_maestro/src/ide/workspace_symbols.rs",
    "crates/vize_maestro/src/ide/workspace_symbols/allocation_observation.rs",
    "crates/vize_maestro/src/server/state/workspace_project_files/symbols.rs"].map((file: string) =>
    ({ path: file, sha256: hash(fs.readFileSync(file)) })),
  build,
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
fs.writeFileSync(path.join(output, `${stage}.json`), JSON.stringify(snapshot, null, 2) + "\n", { flag: "wx" });
