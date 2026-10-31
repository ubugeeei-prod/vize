import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";

const BASE = "a91bd1ec348fe804412ef14913e8e3a25987fa4a";
const TEST = "lsp_client::editor_lsp::native_vue::logical_probe::actual_logical_projection_requires_the_independent_configured_lsp_to_agree";
const SOURCE_PATHS = ["crates/vize_canon/src/lsp_client/editor_lsp/native_vue.rs",
  "crates/vize_canon/src/lsp_client/editor_lsp/native_vue/logical_probe.rs"].sort();
const WORKFLOW_PATHS = [".github/actions/lsp-logical-probe/run.ts", ".github/workflows/lsp-current-baseline.yml"].sort();
const hash = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
function object(value: unknown): Record<string, unknown> {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return Object.fromEntries(Object.entries(value));
}
function text(value: unknown): string { assert(typeof value === "string"); return value; }
function env(name: string): string { return text(process.env[name]); }
function git(...args: string[]): string { return execFileSync("git", args, { encoding: "utf8" }).trim(); }
function completeBlob(revision: string, file: string, physical: Buffer): Buffer {
  const bytes = Number(git("cat-file", "-s", `${revision}:${file}`));
  assert(Number.isSafeInteger(bytes) && bytes >= 0 && bytes <= 4 * 1024 * 1024);
  assert.equal(physical.length, bytes);
  const immutable = execFileSync("git", ["show", `${revision}:${file}`], { maxBuffer: bytes + 1 });
  assert.deepEqual(physical, immutable);
  return immutable;
}
function parent(raw: string, expected: string): void {
  const end = raw.indexOf("\n\n"); assert(end > 0);
  const headers = raw.slice(0, end).split("\n");
  assert.match(headers[0] ?? "", /^tree [a-f0-9]{40}$/u);
  assert.deepEqual(headers.filter(line => line === "parent" || line.startsWith("parent ")), ["parent " + expected]);
}
function execution(stdout: string): void {
  assert.deepEqual([...stdout.matchAll(/^test (\S+) \.\.\./gm)].map(match => match[1]), [TEST]);
  const summaries = [...stdout.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured;/g)];
  assert.equal(summaries.length, 1);
  assert.deepEqual(summaries[0]?.slice(1), ["1", "0", "0", "0"]);
}
if (process.argv[2] === "--controls") {
  const good = `running 1 test\ntest ${TEST} ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n`;
  execution(good);
  for (const bad of [good.replace(TEST, "unrelated"), good.replace("1 passed", "0 passed"), good + good,
    good.replace("0 ignored", "1 ignored"), good.replace(`test ${TEST} ... ok\n`, "")]) assert.throws(() => execution(bad));
  const tree = "tree " + "b".repeat(40), single = tree + "\nparent " + BASE;
  parent(single + "\n\nfixture", BASE);
  for (const bad of [tree + "\nparent other\n\nfixture", single + "\nparent " + BASE + "\n\nfixture",
    single + "\nparent\n\nfixture", single, tree + "\n\nroot"]) assert.throws(() => parent(bad, BASE));
  const revision = "91bb406eb752e86f40dbd78156ccb29a81ad1e62", file = "pnpm-lock.yaml";
  const size = Number(git("cat-file", "-s", `${revision}:${file}`));
  assert.equal(size, 1150435); assert(size > 1024 * 1024);
  const actual = execFileSync("git", ["show", `${revision}:${file}`], { maxBuffer: size + 1 });
  completeBlob(revision, file, actual);
  const changed = Buffer.from(actual); changed[0] = (changed.at(0) ?? 0) ^ 1;
  assert.throws(() => completeBlob(revision, file, changed));
  process.stdout.write(JSON.stringify({ executionCases: 6, parentCases: 6, completeBlobCases: 2,
    completeBlobBytes: actual.length, completeBlobSha256: hash(actual), passed: 14, qualificationCredit: 0 }) + "\n");
} else {
  const source = env("SOURCE_SHA"), workflow = env("WORKFLOW_SHA"), evidence = fs.realpathSync(env("EVIDENCE"));
  assert.match(source, /^[a-f0-9]{40}$/u); assert.match(workflow, /^[a-f0-9]{40}$/u);
  assert.equal(process.version, "v24.14.0"); assert.equal(process.platform, "linux");
  assert.equal(git("rev-parse", "HEAD"), source); assert.equal(env("EXECUTED_WORKFLOW_SHA"), workflow);
  assert.deepEqual(fs.readFileSync(text(process.argv[1])), execFileSync("git", ["show", `${workflow}:.github/actions/lsp-logical-probe/run.ts`]));
  parent(git("cat-file", "-p", source), BASE); parent(git("cat-file", "-p", workflow), source);
  assert.deepEqual(git("diff", "--name-only", BASE, source).split("\n").sort(), SOURCE_PATHS);
  assert.deepEqual(git("diff", "--name-only", source, workflow).split("\n").sort(), WORKFLOW_PATHS);
  assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
  assert.equal(env("CARGO_INCREMENTAL"), "0");
  function save(name: string, value: unknown): void {
    fs.writeFileSync(path.join(evidence, name + ".json"), JSON.stringify(value, null, 2) + "\n", { flag: "wx", mode: 0o600 });
  }
  function image(file: string) {
    const realpath = fs.realpathSync(file), before = fs.statSync(realpath, { bigint: true });
    assert(before.isFile()); const bytes = fs.readFileSync(realpath), after = fs.statSync(realpath, { bigint: true });
    assert.equal(before.ino, after.ino); assert.equal(before.size, after.size); assert.equal(before.mtimeNs, after.mtimeNs);
    assert.equal(bytes.subarray(0, 4).toString("hex"), "7f454c46");
    return { path: realpath, bytes: bytes.length, sha256: hash(bytes) };
  }
  function run(name: string, executable: string, args: string[], environment = process.env) {
    const out = fs.openSync(path.join(evidence, name + ".stdout"), "wx", 0o600);
    const err = fs.openSync(path.join(evidence, name + ".stderr"), "wx", 0o600);
    try {
      const result = spawnSync(executable, args, { env: environment, stdio: ["ignore", out, err] });
      const status = { executable, args, code: result.status, signal: result.signal, error: result.error?.message ?? null };
      save(name + ".exit", status); return status;
    } finally { fs.closeSync(out); fs.closeSync(err); }
  }
  const provider = object(JSON.parse(fs.readFileSync("target/nextest/full/native-runtime-provider.json", "utf8")));
  assert.equal(provider["sourceSha"], source); assert.equal(provider["selection"], "checkout-locked-platform-package");
  const packageRow = object(provider["package"]), native = object(provider["executable"]);
  assert.equal(packageRow["name"], "@typescript/typescript-linux-x64"); assert.equal(packageRow["version"], "7.0.2");
  const nativeImage = image(env("CORSA_PATH")); assert.equal(nativeImage.path, native["path"]); assert.equal(nativeImage.sha256, native["sha256"]);
  assert.deepEqual(native["version"], { args: ["--version"], stdout: "Version 7.0.2\n", stderr: "", exitCode: 0, signal: null });
  const bindingPaths = [...SOURCE_PATHS, "crates/vize_canon/Cargo.toml", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
    "package.json", "pnpm-lock.yaml", "tools/support/compat/github/full-rust-runtime.ts",
    "crates/vize_canon/src/lsp_client/editor_lsp/native_vue/filesystem.rs"];
  const bindings = bindingPaths.map(file => {
    const bytes = fs.readFileSync(file); completeBlob(source, file, bytes);
    return { path: file, sha256: hash(bytes) };
  });
  save("before", { source, workflow, base: BASE, bindings, nativeImage, provider,
    sourceTree: git("rev-parse", source + "^{tree}"), rustc: execFileSync("rustc", ["-Vv"], { encoding: "utf8" }),
    cargo: execFileSync("cargo", ["-V"], { encoding: "utf8" }), qualificationCredit: 0 });
  const built = run("build", "cargo", ["test", "--locked", "--profile", "ci", "-p", "vize_canon", "--lib", TEST,
    "--no-run", "--message-format=json-render-diagnostics"], { ...process.env, VIZE_LOGICAL_PROBE_COMPILED_SOURCE: source });
  assert.deepEqual([built.code, built.signal, built.error], [0, null, null]);
  const rows = fs.readFileSync(path.join(evidence, "build.stdout"), "utf8").split("\n").filter(Boolean).map(line => object(JSON.parse(line)));
  assert.equal(rows.filter(row => row["reason"] === "build-finished" && row["success"] === true).length, 1);
  const libraries = rows.filter(row => row["reason"] === "compiler-artifact" && object(row["target"])["name"] === "vize_canon"
    && object(row["profile"])["test"] === true && typeof row["executable"] === "string");
  assert.equal(libraries.length, 1); const library = libraries[0]; assert(library !== undefined);
  const features = library["features"]; assert(Array.isArray(features));
  assert(!features.includes("profiling") && !features.includes("micro-profiling"));
  const libraryImage = image(text(library["executable"])); save("library", { source, artifact: library, libraryImage });
  const runtimeEnv: NodeJS.ProcessEnv = { ...process.env, EXPECTED_SOURCE: source, VIZE_LOGICAL_PROBE_CAPTURE: path.join(evidence, "decoded") };
  delete runtimeEnv["VIZE_LOGICAL_PROBE_COMPILED_SOURCE"];
  const result = run("test", libraryImage.path, [TEST, "--exact", "--ignored", "--nocapture"], runtimeEnv);
  save("after", { source, workflow, nativeImage: image(env("CORSA_PATH")), libraryImage: image(libraryImage.path), result, qualificationCredit: 0 });
  assert.deepEqual(image(env("CORSA_PATH")), nativeImage); assert.deepEqual(image(libraryImage.path), libraryImage);
  assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
  const decoded = path.join(evidence, "decoded");
  assert.deepEqual(JSON.parse(fs.readFileSync(path.join(decoded, "compiled-source.json"), "utf8")), { runtimeSource: source, compiledSource: source });
  assert.deepEqual(JSON.parse(fs.readFileSync(path.join(decoded, "library-image.json"), "utf8")), libraryImage);
  assert.deepEqual([result.code, result.signal, result.error], [0, null, null]);
  execution(fs.readFileSync(path.join(evidence, "test.stdout"), "utf8"));
}
