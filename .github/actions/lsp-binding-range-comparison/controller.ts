import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { adapterIdentity, INVOKED_BENCHMARK, ORIGINAL_BENCHMARK, SHIPPING_RECIPE, shippingBenchmark, shippingBinary, shippingCargoArtifact } from "./shipping-adapter.ts";

export const BASE_SOURCE = "3279664693067173a8da325f189dca917557fec5";
export const SOURCE_PARENT = "bcc02cff09b2c23741b1bc8cc66b67c465211e57";
export const CANDIDATE_SOURCE = "b73a8078190e7e7693334fe1faffc9b0a305d872";
export const ORDER = ["B", "C", "C", "B"] as const;
export type Role = (typeof ORDER)[number];
export const SOURCE_PATHS = [
  "crates/vize_maestro/src/ide/references.rs",
  "crates/vize_maestro/src/ide/references/canonical.rs",
  "crates/vize_maestro/src/ide/references/canonical/local_binding.rs",
  "crates/vize_maestro/src/ide/references/structural.rs",
  "crates/vize_maestro/src/ide/rename/native.rs",
  "crates/vize_maestro/src/ide/rename/native/private_tests.rs",
  "crates/vize_maestro/src/ide/rename/native/private_tests/fixtures.rs"
] as const;
const SDK_PATHS = ["npm/cli/content-mapper-helpers.d.cts", "npm/cli/content-mapper-helpers.d.mts", "npm/cli/content-mapper-helpers.d.ts", "npm/cli/generated/content-mapper-types.d.ts", "npm/cli/virtual-ts-globals.d.ts", "npm/cli/virtual-ts-patterns.d.ts", "npm/cli/virtual-ts-slots.d.ts", "npm/cli/virtual-ts.d.cts", "npm/cli/virtual-ts.d.mts", "npm/cli/virtual-ts.d.ts", "npm/cli/package.json", "npm/cli/scripts/generate-content-mapper-types.ts"];
export const TOOLING_PATHS = [
  ".github/workflows/lsp-current-baseline.yml",
  ".github/actions/lsp-binding-range-comparison/controller.ts",
  ".github/actions/lsp-binding-range-comparison/controls.test.ts",
  ".github/actions/lsp-binding-range-comparison/README.md",
  ".github/actions/lsp-binding-range-comparison/shipping-adapter.ts",
] as const;
export const FOCUSED_TEST = "ide::references::canonical::private_tests::public_or_property_linked_bindings_keep_project_navigation";
export const SNAPSHOT_PREFIX = "ide::references::structural::snapshot::tests::";
export const OWNERSHIP_PREFIX = "document::store::structural_navigation::tests::";
export const SNAPSHOT_TESTS = [
  "retained_facts_outlive_parser_arena_and_preserve_fresh_query_guards",
  "every_projection_coordinate_and_policy_change_refuses_the_old_entry",
  "two_process_leases_include_retired_readers_and_refusal_does_not_rebuild",
  "admission_is_atomic_across_threads_and_panics_do_not_poison_the_cell",
  "enum_constant_storage_is_not_retained_by_the_cache",
  "dependency_changes_are_observed_before_any_cache_lookup",
  "large_source_admission_refusal_returns_the_same_fresh_packet",
  "authored_reference_rename_and_prepare_anchors_are_complete",
  "full_reference_rename_and_prepare_packets_match_the_original_uncached_collector",
  "fairness::a_third_document_replaces_the_oldest_idle_publication",
  "fairness::active_and_retired_readers_keep_the_cap_while_idle_entries_can_rotate",
  "fairness::stale_publication_identities_cannot_evict_a_replacement_or_reopened_cell",
  "fairness::concurrent_publications_preserve_reader_leases_and_bounded_registration",
].map((name) => SNAPSHOT_PREFIX + name);
export const OWNERSHIP_TESTS = [
  "only_the_exact_resident_source_owner_is_admitted",
  "edit_get_mut_rename_close_and_reopen_detach_old_generations",
  "a_retired_owner_cannot_rejoin_a_concurrently_reopened_document",
].map((name) => OWNERSHIP_PREFIX + name);
export const OPERATIONS = ["completion", "hover", "definition", "references", "prepareRename", "rename"];
const SHA = /^[a-f0-9]{40}$/;
const DIGEST = /^[a-f0-9]{64}$/;
type Obj = Record<string, unknown>;
type Result = { status: number | null; signal: string | null; error: string | null };
const hash = (bytes: string | Buffer): string => createHash("sha256").update(bytes).digest("hex");
const record = (value: unknown): Obj => { assert.ok(value && typeof value === "object" && !Array.isArray(value)); return value as Obj; };
const array = (value: unknown): unknown[] => { assert.ok(Array.isArray(value)); return value; };
const equalPaths = (actual: readonly string[], expected: readonly string[]): void => assert.deepEqual([...actual].sort(), [...expected].sort());

export function sourceFor(role: Role): string {
  const source = role === "B" ? BASE_SOURCE : CANDIDATE_SOURCE;
  assert.match(source, SHA, "finite candidate source must be filled before publication");
  assert.notEqual(BASE_SOURCE, CANDIDATE_SOURCE);
  return source;
}

export function checkAncestry(candidateParent: string, sourceParentParent: string, workflowParent: string, changedSource: string[], changedTooling: string[]): void {
  assert.equal(candidateParent, SOURCE_PARENT); assert.equal(sourceParentParent, BASE_SOURCE);
  assert.equal(workflowParent, sourceFor("C"));
  equalPaths(changedSource, SOURCE_PATHS);
  equalPaths(changedTooling, TOOLING_PATHS);
}

function checkNames(stdout: string, expected: readonly string[]): void {
  const clean = stdout.replace(/\u001b\[[0-9;]*m/g, "");
  // Preserve the accepted announcement-plus-sole-summary rule for interleaved nocapture.
  equalPaths([...clean.matchAll(/^test (\S+) \.\.\./gm)].map((match) => match[1]!), expected);
  const summaries = [...clean.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured;/g)];
  assert.equal(summaries.length, 1);
  assert.deepEqual(summaries[0]!.slice(1), [String(expected.length), "0", "0", "0"]);
}
export function checkFocused(stdout: string, role: Role): void {
  sourceFor(role); checkNames(stdout, [FOCUSED_TEST]);
}
export function checkCandidateTests(stdout: string, group: "snapshot" | "ownership"): void {
  checkNames(stdout, group === "snapshot" ? SNAPSHOT_TESTS : OWNERSHIP_TESTS);
}
function range(value: unknown): void {
  const item = record(value); equalPaths(Object.keys(item), ["start", "end"]);
  for (const key of ["start", "end"]) {
    const position = record(item[key]); equalPaths(Object.keys(position), ["line", "character"]);
    for (const n of Object.values(position)) assert.ok(Number.isSafeInteger(n) && Number(n) >= 0);
  }
}
function packet(value: unknown, method: string): void {
  if (value === null) return;
  if (method === "prepare") { range(value); return; }
  for (const valueEntry of array(value)) {
    const entry = record(valueEntry);
    equalPaths(Object.keys(entry), method === "references" ? ["uri", "range"] : ["range", "newText"]);
    range(entry.range);
    if (method === "references") assert.equal(entry.uri, "file:///workspace/Packet.vue");
    else assert.equal(typeof entry.newText, "string");
  }
}
export function checkSnapshotRows(stdout: string): Obj {
  const expected: Obj[] = [];
  for (let caseIndex = 0; caseIndex < 12; caseIndex++) for (const newline of ["\n", "\r\n"])
    for (let slot = 0; slot < 4; slot++) {
      for (const include of [false, true]) expected.push({ case: caseIndex, newline, slot, method: "references", include });
      for (const name of ["renamed", "beta", "outside", "雪"]) expected.push({ case: caseIndex, newline, slot, method: "rename", name });
      expected.push({ case: caseIndex, newline, slot, method: "prepare" });
    }
  const markers = [...stdout.matchAll(/structural-snapshot-oracle ([^\r\n]*)/g)];
  assert.equal(markers.length, 672, "retain every original, current and warm whole packet");
  const positiveWarm: Record<string, number> = { references: 0, rename: 0, prepare: 0 };
  let hits = 0; let misses = 0;
  for (const [index, match] of markers.entries()) {
    const row = record(JSON.parse(match[1]!)); const identity = expected[index]!;
    equalPaths(Object.keys(row), [...Object.keys(identity), "original", "candidate", "warm", "warmHit"]);
    for (const [key, value] of Object.entries(identity)) assert.deepEqual(row[key], value);
    assert.deepEqual(row.candidate, row.original); assert.deepEqual(row.warm, row.original);
    const method = String(identity.method); packet(row.original, method);
    assert.equal(typeof row.warmHit, "boolean");
    if (row.warmHit) {
      hits++;
      if (row.original !== null && (method === "prepare" || array(row.original).length > 0)) positiveWarm[method]!++;
    } else misses++;
  }
  for (const [method, count] of Object.entries(positiveWarm)) assert.ok(count > 0, `real nonempty warm hits required for ${method}`);
  return { rows: markers.length, hits, misses, positiveWarm, scope: "owned Rust full-packet and actual cache-hit proof; not same-ELF CLI/native proof" };
}

export function validateReport(value: unknown, source: string, imageSha256: string): void {
  const report = record(value);
  assert.equal(report.schemaVersion, 3); assert.equal(report.status, "passed"); assert.equal(report.commit, source);
  const config = record(report.configuration);
  assert.equal(config.freshSamplesPerProduct, 100); assert.equal(config.warmSamplesPerProduct, 100);
  assert.equal(config.schedule, "ABBA by round; both products receive byte-identical primary method/params sequences");
  assert.equal(config.diagnosticQuietMs, 250); assert.equal(config.diagnosticDeadlineMs, 60_000); assert.equal(config.diagnosticBarriers, 2);
  const receipt = record(report.sourceToolReceipt);
  const { sha256: receiptDigest, ...receiptPayload } = receipt;
  assert.equal(receiptDigest, hash(JSON.stringify(receiptPayload))); assert.equal(receipt.sourceSha, source);
  shippingBinary(receipt.vizeBinary, source, imageSha256);
  const products = record(report.products); equalPaths(Object.keys(products), ["vize", "vue-language-tools"]);
  for (const phase of ["fresh", "warm"] as const) {
    const left = array(record(products.vize)[phase]); const right = array(record(products["vue-language-tools"])[phase]);
    assert.equal(left.length, 100); assert.equal(right.length, 100);
    for (const [index, value] of left.entries()) {
      const paired = record(right[index]); const sample = record(value);
      assert.deepEqual(sample.oracleReceipt, paired.oracleReceipt, "whole operation packet equivalence");
      for (const item of [sample, paired]) {
        const timings = record(item.timingsMs); equalPaths(Object.keys(timings), OPERATIONS);
        assert.ok(Object.values(timings).every((n) => typeof n === "number" && Number.isFinite(n) && n >= 0));
        const oracle = record(item.oracleReceipt); const { sha256: digest, ...payload } = oracle;
        assert.equal(digest, hash(JSON.stringify(payload))); assert.deepEqual(oracle.passedOperations, OPERATIONS);
        assert.match(String(item.revisionReceiptSha256), DIGEST);
        const rss = record(item.rss); assert.ok(Number.isSafeInteger(rss.totalKiB) && Number(rss.totalKiB) > 0);
        assert.ok(Number.isSafeInteger(rss.processes) && Number(rss.processes) > 0);
      }
    }
    assert.equal(array(record(report.oracleEquivalenceReceipts)[phase]).length, 100);
  }
  for (const product of Object.values(products)) {
    const evidence = record(product);
    assert.equal(array(evidence.freshDiagnostics).length, 100); assert.equal(array(evidence.warmDiagnostics).length, 1);
  }
  assert.equal(array(record(report.cacheResetReceipt).rows).length, 200);
  assert.ok(array(record(report.cacheResetReceipt).rows).every((row) => record(row).absenceVerified === true));
  const primary = record(record(report.wire).primaryOperationReceipts);
  assert.equal(array(primary.fresh).length, 100); assert.equal(array(primary.warm).length, 1);
  assert.equal(array(report.warmupBoundaryReceipts).length, 2);
}

function json(file: string): unknown { return JSON.parse(fs.readFileSync(file, "utf8")); }
function save(file: string, value: unknown): void { fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`, { flag: "wx", mode: 0o600 }); }
function fileHash(file: string): string { assert.ok(fs.lstatSync(file).isFile()); return hash(fs.readFileSync(file)); }
function copyOwned(source: string, destination: string): void {
  if (fs.existsSync(destination)) assert.ok(fs.lstatSync(destination).isFile());
  fs.copyFileSync(source, destination); fs.chmodSync(destination, 0o700);
}
function git(repo: string, ...args: string[]): string {
  const result = spawnSync("git", args, { cwd: repo, encoding: "utf8", maxBuffer: 8 * 1024 * 1024 });
  assert.equal(result.status, 0, result.stderr); return result.stdout.trim();
}
function pathsBetween(repo: string, left: string, right: string): string[] { return git(repo, "diff", "--name-only", left, right).split("\n").filter(Boolean); }
export function parentFromCommit(raw: string): string {
  const header = raw.split("\n\n", 1)[0]!;
  const parents = header.split("\n").filter((line) => line.startsWith("parent ")).map((line) => line.slice(7));
  assert.equal(parents.length, 1, "finite source must have one immutable parent");
  assert.match(parents[0]!, SHA); return parents[0]!;
}
function immutableParent(repo: string, source: string): string { return parentFromCommit(git(repo, "cat-file", "-p", source)); }
function clean(repo: string, source: string): void {
  assert.equal(git(repo, "rev-parse", "HEAD"), source);
  assert.equal(git(repo, "status", "--porcelain=v1", "--untracked-files=no"), "");
}
function snapshot(repo: string): Obj {
  const tree = git(repo, "ls-tree", "-r", "-z", "--format=%(objectmode) %(path)", "HEAD");
  const files = tree.split("\0")
    .filter((entry) => /^100(644|755) /.test(entry)).map((entry) => entry.slice(7)).filter((name) =>
      ["tests/_helpers/", "tests/tooling/support/", "tests/differential/", "tests/performance/support/", "tests/_fixtures/performance/lsp-dual-server/", "tests/tooling/lsp-dual-server-"].some((prefix) => name.startsWith(prefix)) ||
      ["tests/performance/lsp-dual-server-benchmark.ts", "tests/package.json", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "package.json", "pnpm-lock.yaml", "pnpm-workspace.yaml", ".cargo/config.toml", ...SDK_PATHS, ...SOURCE_PATHS].includes(name));
  return { source: git(repo, "rev-parse", "HEAD"), tree: git(repo, "rev-parse", "HEAD^{tree}"),
    trackedTreeSha256: hash(tree), files: files.map((name) => ({ path: name, sha256: fileHash(path.join(repo, name)) })) };
}
function run(repo: string, dir: string, label: string, executable: string, args: string[], env: NodeJS.ProcessEnv): Result {
  const stdout = fs.openSync(path.join(dir, `${label}.stdout`), "wx", 0o600);
  const stderr = fs.openSync(path.join(dir, `${label}.stderr`), "wx", 0o600);
  const startNs = process.hrtime.bigint(); let result: Result;
  try {
    const child = spawnSync(executable, args, { cwd: repo, env, stdio: ["ignore", stdout, stderr] });
    result = { status: child.status, signal: child.signal, error: child.error?.message ?? null };
  } finally { fs.closeSync(stdout); fs.closeSync(stderr); }
  save(path.join(dir, `${label}.exit.json`), { executable, args, cwd: repo, startNs: String(startNs), endNs: String(process.hrtime.bigint()), ...result });
  return result;
}
function succeeded(result: Result): void { assert.deepEqual(result, { status: 0, signal: null, error: null }); }
function regularIdentity(file: string): Obj {
  const stat = fs.statSync(file, { bigint: true }); assert.ok(stat.isFile());
  return { realpath: fs.realpathSync(file), bytes: Number(stat.size), dev: String(stat.dev), ino: String(stat.ino),
    mtimeNs: String(stat.mtimeNs), ctimeNs: String(stat.ctimeNs), sha256: fileHash(fs.realpathSync(file)) };
}

function cargoArtifact(file: string, executable: string): Obj {
  const rows = fs.readFileSync(file, "utf8").split("\n").filter(Boolean).map((line) => record(JSON.parse(line)));
  const bins = rows.filter((row) => row.reason === "compiler-artifact" && record(row.target).name === "vize" && row.executable === executable);
  assert.equal(bins.length, 1, "one actual Cargo CLI artifact is required");
  const bin = bins[0]!;
  shippingCargoArtifact(bin);
  const maestro = rows.filter((row) => row.reason === "compiler-artifact" && record(row.target).name === "vize_maestro");
  assert.ok(maestro.length > 0);
  assert.ok(maestro.every((row) => array(row.features).includes("native")));
  assert.ok(maestro.every((row) => array(row.features).every((feature) => feature !== "profiling" && feature !== "micro-profiling")));
  assert.ok(rows.some((row) => row.reason === "build-finished" && row.success === true));
  return { cli: bin, maestro };
}

function resolveProgram(repo: string, name: string): string {
  for (const directory of (process.env.PATH ?? "").split(path.delimiter)) {
    const candidate = path.resolve(directory || repo, name);
    try { fs.accessSync(candidate, fs.constants.X_OK); if (fs.statSync(candidate).isFile()) return fs.realpathSync(candidate); }
    catch { /* Continue the same PATH search used for the child command. */ }
  }
  throw new Error(`missing executable: ${name}`);
}

function selectedTool(repo: string, dir: string, name: "rustc" | "cargo"): string {
  succeeded(run(repo, dir, `selected-${name}`, "rustup", ["which", name], process.env));
  const output = fs.readFileSync(path.join(dir, `selected-${name}.stdout`), "utf8");
  const selected = output.trimEnd();
  assert.ok(path.isAbsolute(selected) && !/[\r\n]/.test(selected)); assert.equal(output, `${selected}\n`);
  return fs.realpathSync(selected);
}

async function main(): Promise<void> {
  const repo = fs.realpathSync(process.cwd()); const workflow = process.env.GITHUB_SHA ?? "";
  assert.equal(workflow, process.env.EXPECTED_WORKFLOW); assert.match(workflow, SHA);
  assert.equal(process.env.RUNNER_OS, "Linux"); assert.equal(process.platform, "linux"); assert.equal(process.arch, "x64");
  assert.equal(process.env.NODE_OPTIONS, "--disable-warning=DEP0040 --disable-warning=DEP0169");
  assert.equal(process.env.CARGO_INCREMENTAL, "0"); assert.equal(process.env.CARGO_BUILD_JOBS, "16"); assert.equal(process.env.RAYON_NUM_THREADS, "8");
  assert.equal(process.env.REQUESTED_CANDIDATE, sourceFor("C"));
  clean(repo, workflow);
  const parents = { candidate: immutableParent(repo, sourceFor("C")), sourceParent: immutableParent(repo, SOURCE_PARENT), workflow: immutableParent(repo, workflow) };
  checkAncestry(parents.candidate, parents.sourceParent, parents.workflow, pathsBetween(repo, BASE_SOURCE, sourceFor("C")), pathsBetween(repo, sourceFor("C"), workflow));
  for (const name of Object.keys(process.env)) {
    if (name.startsWith("LSP_DUAL_") || ["VIZE_LSP_BIN", "CORSA_PATH", "CORSA_EXECUTABLE", "VIZE_CORSA_PATH", "TSGO_PATH", "TSGO_EXECUTABLE", "VIZE_TSGO_PATH", "TSRS_PATH", "TSRS_EXECUTABLE", "VIZE_TSRS_PATH", "VIZE_TRACE_EDITOR_PREPARATION", "VIZE_TEST_TSGO_PATH"].includes(name)) assert.ok(!process.env[name], `explicit ambient ${name} refused`);
  }
  const temp = fs.realpathSync(process.env.RUNNER_TEMP ?? "");
  const suffix = `${process.env.GITHUB_RUN_ID}-${process.env.GITHUB_RUN_ATTEMPT}`;
  const evidence = path.join(temp, `binding-range-comparison-${suffix}`); fs.mkdirSync(evidence, { mode: 0o700 });
  const images = path.join(temp, `binding-range-images-${suffix}`); fs.mkdirSync(images, { mode: 0o700 });
  const originalBenchmark = fs.readFileSync(path.join(repo, ORIGINAL_BENCHMARK), "utf8");
  for (const role of ["B", "C"] as const) {
    const immutable = spawnSync("git", ["show", `${sourceFor(role)}:${ORIGINAL_BENCHMARK}`], { cwd: repo, maxBuffer: 4 * 1024 * 1024 });
    assert.equal(immutable.status, 0); assert.deepEqual(immutable.stdout, Buffer.from(originalBenchmark));
  }
  assert.equal(git(repo, "ls-files", "--", INVOKED_BENCHMARK), "");
  const adapterFile = path.join(repo, INVOKED_BENCHMARK);
  fs.writeFileSync(adapterFile, shippingBenchmark(originalBenchmark), { flag: "wx", mode: 0o600 });
  const benchmarkAdapter = () => adapterIdentity(fs.readFileSync(path.join(repo, ORIGINAL_BENCHMARK), "utf8"), fs.readFileSync(adapterFile, "utf8"));
  fs.writeFileSync(path.join(evidence, "original-benchmark.ts.txt"), originalBenchmark, { flag: "wx", mode: 0o600 });
  fs.writeFileSync(path.join(evidence, "invoked-benchmark.ts.txt"), fs.readFileSync(adapterFile), { flag: "wx", mode: 0o600 });
  const adapterDiff = spawnSync("git", ["diff", "--no-index", "--", ORIGINAL_BENCHMARK, INVOKED_BENCHMARK], { cwd: repo });
  assert.equal(adapterDiff.status, 1); assert.equal(adapterDiff.stderr.length, 0);
  fs.writeFileSync(path.join(evidence, "benchmark-adapter.diff"), adapterDiff.stdout, { flag: "wx", mode: 0o600 });
  const stageDir = path.join(repo, "target", "ci");
  for (const part of [path.join(repo, "target"), stageDir]) {
    if (fs.existsSync(part)) assert.ok(fs.lstatSync(part).isDirectory() && !fs.lstatSync(part).isSymbolicLink());
    else fs.mkdirSync(part);
  }
  const stage = path.join(stageDir, "vize"); const receipt = `${stage}.differential-build.json`;
  assert.ok(!fs.existsSync(stage) && !fs.existsSync(receipt), "fresh checkout staging outputs must be absent");
  const native = path.join(repo, "node_modules/@typescript/typescript-linux-x64/lib/tsc");
  const nativeIdentity = regularIdentity(native);
  const nativeManifest = record(json(path.join(path.dirname(fs.realpathSync(native)), "../package.json")));
  assert.equal(nativeManifest.name, "@typescript/typescript-linux-x64"); assert.equal(nativeManifest.version, "7.0.2");
  const env = { ...process.env, CORSA_PATH: native, VIZE_LSP_BIN: stage };
  for (const name of ["RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_RUSTC", "CARGO_BUILD_RUSTC_WRAPPER", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER"])
    assert.ok(!process.env[name], `alternate compiler selection ${name} refused`);
  for (const [label, executable, args] of [
    ["node-version", process.execPath, ["--version"]], ["vp-version", "vp", ["--version"]],
    ["rustc-version", "rustc", ["-vV"]], ["cargo-version", "cargo", ["--version"]],
  ] as const) succeeded(run(repo, evidence, label, executable, [...args], process.env));
  const toolPaths = { node: process.execPath, vpLauncher: resolveProgram(repo, "vp"),
    rustc: selectedTool(repo, evidence, "rustc"), cargo: selectedTool(repo, evidence, "cargo") };
  const toolIdentity = (): Obj => Object.fromEntries(Object.entries(toolPaths).map(([name, file]) => [name, regularIdentity(file)]));
  const authority = { workflow, base: BASE_SOURCE, candidate: sourceFor("C"), immutableParents: parents, order: ORDER, sourcePaths: SOURCE_PATHS, toolingPaths: TOOLING_PATHS,
    tools: toolIdentity(), native: nativeIdentity, benchmarkAdapter: benchmarkAdapter(), recipe: SHIPPING_RECIPE,
    controlledEnvironment: { CARGO_INCREMENTAL: "0", CARGO_BUILD_JOBS: "16", RAYON_NUM_THREADS: "8", NODE_OPTIONS: process.env.NODE_OPTIONS,
      rustflagsSha256: hash(process.env.RUSTFLAGS ?? ""), encodedRustflagsSha256: hash(process.env.CARGO_ENCODED_RUSTFLAGS ?? "") },
    platform: { arch: os.arch(), release: os.release(), cpus: os.cpus().map(({ model }) => model) } };
  const helperDirectory = path.dirname(fileURLToPath(import.meta.url));
  const helperHashes: Obj[] = [];
  for (const name of ["controller.ts", "controls.test.ts", "README.md", "shipping-adapter.ts"]) {
    const expected = spawnSync("git", ["show", `${workflow}:.github/actions/lsp-binding-range-comparison/${name}`], { cwd: repo });
    assert.equal(expected.status, 0);
    const staged = path.join(helperDirectory, name); assert.equal(fileHash(staged), hash(expected.stdout));
    helperHashes.push({ name, sha256: fileHash(staged) });
  }
  save(path.join(evidence, "helper-hashes.json"), helperHashes);
  save(path.join(evidence, "authority.json"), authority);
  const campaign: Obj = { ...authority, attempts: [], complete: false, allOriginalPassed: false };
  const originals = new Map<Role, { image: string; digest: string; receipt: string; source: string }>();
  let primaryFailure: string | null = null;
  try {
    succeeded(run(repo, evidence, "native-version", native, ["--version"], env));
    assert.equal(fs.readFileSync(path.join(evidence, "native-version.stdout"), "utf8"), "Version 7.0.2\n");
    assert.equal(fs.statSync(path.join(evidence, "native-version.stderr")).size, 0);
    const controls = run(path.join(repo, "tests"), evidence, "original-controls", "vp", ["run", "test:performance:lsp-dual-server:controls"], process.env); succeeded(controls);
    for (const role of ["B", "C"] as const) {
      const source = sourceFor(role); git(repo, "checkout", "--detach", source); clean(repo, source);
      const dir = path.join(evidence, `build-${role}`); fs.mkdirSync(dir);
      const target = path.join(temp, `binding-range-build-${suffix}-${role}`); assert.ok(!fs.existsSync(target));
      const cargoEnv = { ...env, CARGO_TARGET_DIR: target, CARGO_INCREMENTAL: "0" };
      const sourceBefore = snapshot(repo);
      assert.ok(!fs.existsSync(path.join(target, "release/vize")));
      save(path.join(dir, "before-build.json"), { ...sourceBefore, target, cliAbsent: true, recipe: SHIPPING_RECIPE, benchmarkAdapter: benchmarkAdapter() });
      succeeded(run(repo, dir, "cargo-build", "cargo", ["build", "--release", "-p", "vize", "--message-format=json-render-diagnostics"], cargoEnv));
      const actual = path.join(target, "release/vize"); const image = path.join(images, `${role}-first-vize`);
      const artifact = cargoArtifact(path.join(dir, "cargo-build.stdout"), actual);
      fs.copyFileSync(actual, image, fs.constants.COPYFILE_EXCL); fs.chmodSync(image, 0o500);
      const digest = fileHash(image); copyOwned(image, stage);
      succeeded(run(repo, dir, "build-receipt", "vp", ["exec", "node", "--input-type=module", "-e", "const m = await import('./tests/differential/build-receipt.ts'); console.log(m.writeBuildReceipt(process.cwd(), m.SHIPPING_BUILD_RECIPE));"], env));
      const receiptText = fs.readFileSync(receipt, "utf8"); const build = record(JSON.parse(receiptText));
      assert.equal(build.recipe, SHIPPING_RECIPE); assert.equal(build.sourceRevision, source); assert.equal(build.binarySha256, digest);
      const version = run(repo, dir, "first-version", stage, ["--version"], env); succeeded(version);
      assert.equal(fs.readFileSync(path.join(dir, "first-version.stdout"), "utf8"), `${build.cliVersion}\n`); assert.equal(fs.statSync(path.join(dir, "first-version.stderr")).size, 0);
      const first = { source, imageSha256: digest, cargoArtifact: artifact, actualArtifact: regularIdentity(actual), stagedImage: regularIdentity(stage), buildReceipt: build, recipe: SHIPPING_RECIPE,
        actualArguments: ["build", "--release", "-p", "vize", "--message-format=json-render-diagnostics"], metadataOnlyArgument: "--message-format=json-render-diagnostics" };
      save(path.join(dir, "first-image.json"), first);
      const steps: { label: string; filter: string; exact: boolean; group: "focused" | "snapshot" | "ownership" }[] = [
        { label: "focused-test", filter: FOCUSED_TEST, exact: true, group: "focused" },
        { label: "snapshot-tests", filter: SNAPSHOT_PREFIX, exact: false, group: "snapshot" },
        { label: "ownership-tests", filter: OWNERSHIP_PREFIX, exact: false, group: "ownership" },
      ];
      const units = steps.map((step) => {
        const args = ["test", "--profile", "ci", "-p", "vize_maestro", "--lib", step.filter, "--", ...(step.exact ? ["--exact"] : []), "--nocapture"];
        const result = run(repo, dir, step.label, "cargo", args, cargoEnv);
        const postName = step.group === "focused" ? "post-test-image.json" : `post-${step.label}-image.json`;
        const restoredName = step.group === "focused" ? "restored-first-image.json" : `restored-${step.label}-first-image.json`;
        save(path.join(dir, postName), { image: fs.existsSync(actual) ? regularIdentity(actual) : null, focusedResult: result, group: step.group,
          proofScope: "Rust library test context only; no same-ELF CLI/native admission" });
        copyOwned(image, actual); copyOwned(image, stage);
        assert.equal(fileHash(actual), digest); assert.equal(fileHash(stage), digest);
        save(path.join(dir, restoredName), { actual: regularIdentity(actual), staged: regularIdentity(stage) });
        return { ...step, result, stdout: fs.readFileSync(path.join(dir, `${step.label}.stdout`), "utf8") };
      });
      for (const unit of units) {
        succeeded(unit.result);
        if (unit.group === "focused") checkFocused(unit.stdout, role);
        else {
          checkCandidateTests(unit.stdout, unit.group);
          if (unit.group === "snapshot") save(path.join(dir, "snapshot-packet-validation.json"), checkSnapshotRows(unit.stdout));
        }
      }
      assert.deepEqual(snapshot(repo), sourceBefore);
      originals.set(role, { image, digest, receipt: receiptText, source });
    }
    const attempts = campaign.attempts as Obj[];
    let commonProviderInputs: unknown;
    for (const [index, role] of ORDER.entries()) {
      const original = originals.get(role)!; git(repo, "checkout", "--detach", original.source); clean(repo, original.source);
      const dir = path.join(evidence, `attempt-${index + 1}-${role}`); fs.mkdirSync(dir);
      copyOwned(original.image, stage); assert.ok(fs.lstatSync(receipt).isFile()); fs.writeFileSync(receipt, original.receipt);
      const before = { ...snapshot(repo), imageSha256: fileHash(stage), native: regularIdentity(native), tools: toolIdentity(), receiptSha256: fileHash(receipt), benchmarkAdapter: benchmarkAdapter() };
      assert.equal(before.imageSha256, original.digest); assert.deepEqual(before.native, nativeIdentity); assert.deepEqual(before.tools, authority.tools);
      save(path.join(dir, "before.json"), before);
      const args = ["exec", "node", INVOKED_BENCHMARK, "--fresh", "100", "--warm", "100", "--out", path.join(dir, "report")];
      const exit = run(repo, dir, "original-benchmark", "vp", args, env); const errors: string[] = [];
      try {
        succeeded(exit); const report = record(json(path.join(dir, "report/report.json")));
        validateReport(report, original.source, original.digest);
        const tools = record(report.tools);
        assert.equal(record(tools.vueLanguageTools).command, process.execPath);
        assert.equal(record(tools.typescriptLanguageServer).command, process.execPath);
        const providerInputs = { tools, provider: record(report.sourceToolReceipt).provider };
        if (commonProviderInputs === undefined) commonProviderInputs = providerInputs;
        else assert.deepEqual(providerInputs, commonProviderInputs, "original Node/JS provider inputs must stay identical across B,C,C,B");
      }
      catch (error) { errors.push(error instanceof Error ? error.message : String(error)); }
      const after = { ...snapshot(repo), imageSha256: fileHash(stage), native: regularIdentity(native), tools: toolIdentity(), receiptSha256: fileHash(receipt), benchmarkAdapter: benchmarkAdapter() };
      save(path.join(dir, "after.json"), after); assert.deepEqual(after, before, "source/provider/first-image custody changed during original benchmark");
      attempts.push({ index, role, source: original.source, imageSha256: original.digest, exit, errors, originalPassed: errors.length === 0,
        reportSha256: fs.existsSync(path.join(dir, "report/report.json")) ? fileHash(path.join(dir, "report/report.json")) : null });
    }
    campaign.complete = attempts.length === 4; campaign.allOriginalPassed = attempts.length === 4 && attempts.every((attempt) => attempt.originalPassed === true);
    assert.equal(campaign.allOriginalPassed, true, "every original ABBA attempt must pass; failures are retained, never selected away");
  } catch (error) { primaryFailure = error instanceof Error ? error.stack ?? error.message : String(error); throw error; }
  finally { campaign.failure = primaryFailure; save(path.join(evidence, "campaign.json"), campaign); }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error: unknown) => { console.error(error); process.exitCode = 1; });
}
