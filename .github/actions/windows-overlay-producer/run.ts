import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SOURCE = "587b2d94249dcd1cb12402adca85f07d59f19d50";
const PREVIOUS = "03c07ec433d1a55b2588db4c7aadf6fa4b005124";
const BASE = "a2eae573558c19b1279d80db6f17fd77b1a1e633";
const MAIN = "274777b2b18a8e43280a109cfbc4bb2a7286ee00";
const SOURCE_FILES = [
  "crates/vize_canon/src/corsa_bridge/script_document.rs",
  "crates/vize_canon/src/corsa_bridge/vue_dependencies.rs",
  "crates/vize_canon/src/corsa_bridge/vue_dependencies/source.rs",
  "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias.rs",
  "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context.rs",
  "crates/vize_canon/src/corsa_bridge/vue_document/build.rs",
  "crates/vize_canon/src/corsa_bridge/vue_document/build/tests.rs",
  "crates/vize_canon/src/corsa_bridge/vue_document/build/tests/windows_overlays.rs",
  "tests/_fixtures/differential/typechecker/manifest.json",
  "tests/_fixtures/differential/typechecker/windows-verbatim-overlay-11145/README.md",
  "tests/_fixtures/differential/typechecker/windows-verbatim-overlay-11145/input.json",
  "tests/tooling/differential-dialect-coverage.test.ts",
  "tests/tooling/windows-verbatim-overlay-fixture.test.ts"
];
const HELPER = ".github/actions/windows-overlay-producer/run.ts";
const WORKFLOW = ".github/workflows/lsp-current-baseline.yml";
export const FQN = "corsa_bridge::vue_document::build::tests::windows_overlays::verbatim_windows_overlay_inputs_preserve_materialized_revision_bytes";
type Obj = Record<string, unknown>;
type Exit = { status: number | null; signal: string | null; error: string | null };
const obj = (value: unknown): Obj => { assert(value !== null && typeof value === "object" && !Array.isArray(value)); return Object.fromEntries(Object.entries(value)); };
const str = (value: unknown): string => { assert(typeof value === "string"); return value; };
const sha = (bytes: Buffer): string => createHash("sha256").update(bytes).digest("hex");
const normal = (value: string): string => path.win32.normalize(value).replace(/^\\\\\?\\(?=[a-z]:\\)/i, "").toLowerCase();
function parents(raw: string, expected: readonly string[]): void {
  const end = raw.indexOf("\n\n"); assert(end > 0); const rows = raw.slice(0, end).split("\n");
  assert.match(rows[0] ?? "", /^tree [a-f0-9]{40}$/); assert.deepEqual(rows.filter(row => row.startsWith("parent ") || row === "parent"), expected.map(value => "parent " + value));
}
function success(exit: Exit): void { assert.deepEqual(exit, { status: 0, signal: null, error: null }); }
export function selectArtifact(raw: string, repository: string, target: string): Obj {
  const rows = raw.split(/\r?\n/).filter(Boolean).map(line => obj(JSON.parse(line)));
  assert.equal(rows.filter(row => row.reason === "build-finished" && row.success === true).length, 1);
  const artifacts = rows.filter(row => row.reason === "compiler-artifact" && obj(row.target).name === "vize_canon"
    && obj(row.profile).test === true && typeof row.executable === "string");
  assert.equal(artifacts.length, 1, "one actual Canon library test artifact required"); const artifact = artifacts[0]!;
  assert.equal(normal(str(artifact.manifest_path)), normal(path.win32.join(repository, "crates/vize_canon/Cargo.toml")));
  const executable = normal(str(artifact.executable)); assert.equal(path.win32.dirname(executable), normal(path.win32.join(target, "ci/deps")));
  assert.match(path.win32.basename(executable), /^vize_canon-[a-f0-9]+\.exe$/); assert.deepEqual(obj(artifact.target).kind, ["lib"]);
  assert.equal(artifact.fresh, false, "fresh private target may not reuse an ambient test binary"); return artifact;
}
export function exactTest(stdout: string, exit: Exit): void {
  success(exit); const text = stdout.replace(/\x1b\[[0-9;]*[A-Za-z]/g, "");
  const running = [...text.matchAll(/^running (\d+) tests?\r?$/gm)]; assert.equal(running.length, 1); assert.equal(running[0]?.[1], "1");
  assert.deepEqual([...text.matchAll(/^test (\S+) \.\.\./gm)].map(row => row[1]), [FQN]);
  const summaries = [...text.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured;/g)];
  assert.equal(summaries.length, 1); assert.deepEqual(summaries[0]!.slice(1), ["1", "0", "0", "0"]);
}
export function sameImage(before: Obj, after: Obj): void { assert.deepEqual(after, before, "actual test image changed while executing"); }
function controls(): void {
  const ok: Exit = { status: 0, signal: null, error: null }, stdout = `running 1 test\ntest ${FQN} ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 99 filtered out\n`;
  const repository = "D:\\repo", target = "D:\\temp\\owned", row = { reason: "compiler-artifact", target: { name: "vize_canon", kind: ["lib"] }, manifest_path: "D:\\repo\\crates\\vize_canon\\Cargo.toml", profile: { test: true }, fresh: false, executable: "D:\\temp\\owned\\ci\\deps\\vize_canon-abc123.exe" };
  const packet = (value: unknown): string => JSON.stringify(value) + '\n{"reason":"build-finished","success":true}\n';
  const cases: (() => void)[] = [
    () => exactTest(stdout, ok),
    () => assert.throws(() => exactTest(stdout.replace("running 1 test", "running 0 tests").replace("1 passed", "0 passed"), ok)),
    () => assert.throws(() => exactTest(stdout.replace(FQN, "foreign_test"), ok)),
    () => assert.throws(() => exactTest(stdout.replace("1 passed; 0 failed; 0 ignored", "0 passed; 0 failed; 1 ignored"), ok)),
    () => assert.throws(() => exactTest(stdout + stdout, ok)),
    () => assert.throws(() => exactTest(stdout, { ...ok, status: 1 })),
    () => {
      const head = "tree " + "a".repeat(40) + "\nparent " + PREVIOUS + "\nparent " + MAIN + "\n\nmessage";
      parents(head, [PREVIOUS, MAIN]); assert.throws(() => parents(head, [MAIN, PREVIOUS])); assert.throws(() => parents(head, [PREVIOUS]));
    },
    () => selectArtifact(packet(row), repository, target),
    () => selectArtifact(packet({ ...row, manifest_path: "\\\\?\\" + row.manifest_path, executable: "\\\\?\\" + row.executable }), repository, target),
    () => assert.throws(() => selectArtifact(packet({ ...row, manifest_path: "D:\\foreign\\Cargo.toml" }), repository, target)),
    () => assert.throws(() => selectArtifact(packet({ ...row, executable: "D:\\foreign\\vize_canon-abc123.exe" }), repository, target)),
    () => assert.throws(() => selectArtifact(JSON.stringify(row) + "\n" + packet(row), repository, target)),
    () => assert.throws(() => selectArtifact(packet({ ...row, profile: { test: false } }), repository, target)),
    () => assert.throws(() => sameImage({ sha256: "a", bytes: 1 }, { sha256: "b", bytes: 1 })),
  ];
  for (const check of cases) check(); console.log(JSON.stringify({ schema: "vize.windows-overlay-pure-controller-controls", passed: cases.length, cases: cases.length, executedRustTests: 0, sdkOrLspCredit: 0 }));
}
function main(): void {
  assert.equal(process.platform, "win32"); assert.equal(process.arch, "x64"); assert.equal(process.version, "v24.14.0");
  const required = (name: string): string => { const value = process.env[name]; assert(value !== undefined && value !== "", name); return value; };
  assert.match(SOURCE, /^[a-f0-9]{40}$/, "pending corrected source cannot execute");
  const workflow = required("WORKFLOW_SHA"), repository = fs.realpathSync(process.cwd()), output = fs.realpathSync(required("EVIDENCE")), target = path.resolve(required("CARGO_TARGET_DIR"));
  assert.equal(required("SOURCE_SHA"), SOURCE); assert.equal(required("EXECUTED_WORKFLOW_SHA"), workflow); assert.match(workflow, /^[a-f0-9]{40}$/);
  assert(!fs.existsSync(target)); fs.mkdirSync(target); assert.notEqual(normal(target), normal(output));
  const save = (name: string, value: unknown): void => fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
  const git = (...args: string[]): Buffer => { const result = spawnSync("git", args, { cwd: repository, maxBuffer: 4 * 1024 * 1024 }); assert.equal(result.status, 0, result.stderr.toString()); return result.stdout; };
  assert.equal(git("rev-parse", "HEAD").toString().trim(), SOURCE); assert.equal(git("status", "--porcelain", "--untracked-files=no").length, 0);
  parents(git("cat-file", "-p", SOURCE).toString(), [PREVIOUS, MAIN]); parents(git("cat-file", "-p", PREVIOUS).toString(), [BASE]); parents(git("cat-file", "-p", workflow).toString(), [SOURCE]);
  assert.deepEqual(git("diff", "--name-only", MAIN, SOURCE).toString().trim().split(/\r?\n/).sort(), SOURCE_FILES.slice().sort());
  assert.deepEqual(git("diff", "--name-only", SOURCE, workflow).toString().trim().split(/\r?\n/).sort(), [HELPER, WORKFLOW].sort());
  assert.deepEqual(fs.readFileSync(fileURLToPath(import.meta.url)), git("show", `${workflow}:${HELPER}`));
  function version(command: string, args: string[]): string {
    const result = spawnSync(command, args, { encoding: "utf8" }); success({ status: result.status, signal: result.signal, error: result.error?.message ?? null });
    assert.equal(result.stderr, ""); assert(result.stdout.length > 0); return result.stdout;
  }
  function snapshot(): Obj {
    const files = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo/config.toml", "crates/vize_canon/Cargo.toml", ...SOURCE_FILES].map(file => {
      const bytes = fs.readFileSync(file); assert.deepEqual(bytes, git("show", `${SOURCE}:${file}`)); return { path: file, sha256: sha(bytes) };
    });
    assert.match(fs.readFileSync("rust-toolchain.toml", "utf8"), /^channel = "1\.99\.1"$/m);
    assert.equal(files.find(file => file.path === "tests/_fixtures/differential/typechecker/windows-verbatim-overlay-11145/input.json")?.sha256, "09e9e861a991d5ba37cd88c87a3a70a86ab2443cbf1b4443243515bb7c8af4f4");
    const rustc = version("rustc", ["-Vv"]), cargo = version("cargo", ["-V"]); assert.match(rustc, /^rustc 1\.99\.1 /);
    return { source: SOURCE, tree: git("rev-parse", "HEAD^{tree}").toString().trim(), files, rustc, cargo };
  }
  const before = snapshot(); save("before.json", { ...before, workflow, target, node: process.version, platform: process.platform, helperSha256: sha(fs.readFileSync(fileURLToPath(import.meta.url))), qualification: "pure Windows producer/materialization bytes only; no SDK, LSP, CLI, release or performance credit" });
  function run(name: string, executable: string, args: string[]): Exit {
    const stdout = fs.openSync(path.join(output, name + ".stdout"), "wx"), stderr = fs.openSync(path.join(output, name + ".stderr"), "wx"); const started = process.hrtime.bigint(); let exit: Exit;
    try { const result = spawnSync(executable, args, { cwd: repository, env: process.env, stdio: ["ignore", stdout, stderr] }); exit = { status: result.status, signal: result.signal, error: result.error?.message ?? null }; }
    finally { fs.closeSync(stdout); fs.closeSync(stderr); }
    save(name + ".exit.json", { executable, args, cwd: repository, startedNs: String(started), endedNs: String(process.hrtime.bigint()), ...exit }); return exit;
  }
  const built = run("cargo-test-build", "cargo", ["test", "--locked", "--profile", "ci", "-p", "vize_canon", "--lib", FQN, "--no-run", "--message-format=json-render-diagnostics"]); success(built);
  const artifact = selectArtifact(fs.readFileSync(path.join(output, "cargo-test-build.stdout"), "utf8"), repository, target), executable = fs.realpathSync(str(artifact.executable));
  assert.equal(normal(path.dirname(executable)), normal(fs.realpathSync(path.join(target, "ci/deps"))), "actual image must remain in the owned target");
  function image(): Obj {
    const stat = fs.statSync(executable, { bigint: true }); assert(stat.isFile()); const bytes = fs.readFileSync(executable); assert.equal(bytes.subarray(0, 2).toString(), "MZ");
    const after = fs.statSync(executable, { bigint: true }); assert.equal(after.size, stat.size); assert.equal(after.mtimeNs, stat.mtimeNs); assert.equal(after.ctimeNs, stat.ctimeNs);
    return { path: executable, bytes: Number(stat.size), sha256: sha(bytes), dev: String(stat.dev), ino: String(stat.ino), mtimeNs: String(stat.mtimeNs), ctimeNs: String(stat.ctimeNs) };
  }
  const first = image(); save("test-image-before.json", { artifact, image: first });
  const exit = run("exact-test", executable, [FQN, "--exact", "--nocapture"]), last = image(); save("test-image-after.json", last);
  const after = snapshot(); save("after.json", after); save("terminal-source.json", { source: git("rev-parse", "HEAD").toString().trim(), trackedStatus: git("status", "--porcelain", "--untracked-files=no").toString() });
  sameImage(first, last); assert.deepEqual(after, before); exactTest(fs.readFileSync(path.join(output, "exact-test.stdout"), "utf8"), exit);
  assert.equal(git("status", "--porcelain", "--untracked-files=no").length, 0);
  save("acceptance.json", { accepted: true, source: SOURCE, workflow, fqn: FQN, actualTests: 1, passed: 1, failed: 0, ignored: 0, testImage: first, scope: "pure Windows producer/materialization byte law; SDK/LSP/CLI/public/performance credit zero" });
}
if (process.argv[2] === "--controls") controls(); else main();
