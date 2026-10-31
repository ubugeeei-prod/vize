import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";

const BASE = "3279664693067173a8da325f189dca917557fec5";
const SOURCE_PARENT = "bcc02cff09b2c23741b1bc8cc66b67c465211e57";
const CANDIDATE = "b73a8078190e7e7693334fe1faffc9b0a305d872";
const PREFIX = "ide::rename::native::private_tests::";
const TESTS = ["private_setup_rename_requires_the_selected_declaration_and_complete_plain_edits",
  "uncertain_or_public_rename_inputs_keep_the_original_route", "private_rename_refuses_stale_equal_version_reopens_and_foreign_owners",
  "private_setup_rename_matches_frozen_whole_native_workspace_edits"].map(name => PREFIX + name).sort();
const SOURCE_PATHS = ["crates/vize_maestro/src/ide/references.rs","crates/vize_maestro/src/ide/references/canonical.rs","crates/vize_maestro/src/ide/references/canonical/local_binding.rs","crates/vize_maestro/src/ide/references/structural.rs","crates/vize_maestro/src/ide/rename/native.rs","crates/vize_maestro/src/ide/rename/native/private_tests.rs","crates/vize_maestro/src/ide/rename/native/private_tests/fixtures.rs"];
const SDK_PATHS = ["npm/cli/content-mapper-helpers.d.cts","npm/cli/content-mapper-helpers.d.mts","npm/cli/content-mapper-helpers.d.ts","npm/cli/generated/content-mapper-types.d.ts","npm/cli/virtual-ts-globals.d.ts","npm/cli/virtual-ts-patterns.d.ts","npm/cli/virtual-ts-slots.d.ts","npm/cli/virtual-ts.d.cts","npm/cli/virtual-ts.d.mts","npm/cli/virtual-ts.d.ts"];
const WORKFLOW_PATHS = [".github/actions/lsp-private-rename-parity/run.ts",".github/workflows/lsp-current-baseline.yml"];
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
  assert.deepEqual([...stdout.matchAll(/^test (\S+) \.\.\. ok$/gm)].map(match => match[1]).sort(), TESTS);
  const summaries = [...stdout.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured;/g)];
  assert.equal(summaries.length, 1); assert.deepEqual(summaries[0]?.slice(1), ["4", "0", "0", "0"]);
}
function configuration(environment: NodeJS.ProcessEnv): void {
  assert.equal(environment["VIZE_TEST_REQUIRE_TSGO"], "1");
  for (const name of ["VIZE_TEST_DISABLE_TSGO", "VIZE_TEST_TSGO_PATH", "VIZE_CORSA_PATH", "VIZE_TSGO_PATH", "VIZE_TSRS_PATH", "VIZE_LSP_BIN", "VIZE_TRACE_EDITOR_PREPARATION"]) assert.ok(!environment[name], name + " must be absent");
}
function pairs(stderr: string): Record<string, unknown>[] {
  const rows = stderr.split("\n").filter(line => line.startsWith("VIZE_PRIVATE_RENAME_NATIVE "))
    .map(line => object(JSON.parse(line.slice("VIZE_PRIVATE_RENAME_NATIVE ".length))));
  assert.equal(rows.length, 13); const identities = new Set<string>(); let positive = 0, publicLocal = 0;
  for (const row of rows) {
    assert.deepEqual(Object.keys(row).sort(), ["actual", "newName", "old", "private", "query", "source"]);
    const source = text(row["source"]), query = text(row["query"]), newName = text(row["newName"]);
    assert.ok(source.includes(query)); assert.ok(newName.length > 0); assert.equal(typeof row["private"], "boolean");
    const identity = JSON.stringify([source, query, newName]); assert.ok(!identities.has(identity)); identities.add(identity);
    assert.deepEqual(row["actual"], row["old"]);
    if (row["private"]) {
      positive++; const packet = object(row["actual"]); const changes = object(packet["changes"]);
      assert.equal(Object.keys(changes).length, 1); const edits = Object.values(changes)[0]; assert.ok(Array.isArray(edits) && edits.length > 0);
      if (source.includes('<Child :message="message" />')) { publicLocal++; assert.ok(["message =", 'message"'].includes(query)); }
    }
  }
  assert.equal(positive, 5); assert.equal(publicLocal, 2); return rows;
}
if (process.argv[2] === "--controls") {
  const good = "running 4 tests\n" + TESTS.map(name => "test " + name + " ... ok\n").join("") + "test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n";
  execution(good);
  for (const bad of [good.replace(TESTS[0] ?? "", "unrelated"), good.replace("4 passed", "0 passed"), good + good,
    good.replace("0 ignored", "1 ignored"), good.replace(" ... ok", " ... ignored")]) assert.throws(() => execution(bad));
  configuration({ VIZE_TEST_REQUIRE_TSGO: "1" });
  for (const bad of [{}, { VIZE_TEST_REQUIRE_TSGO: "0" }, { VIZE_TEST_REQUIRE_TSGO: "1", VIZE_TEST_DISABLE_TSGO: "1" }, { VIZE_TEST_REQUIRE_TSGO: "1", VIZE_TEST_TSGO_PATH: "/foreign" }]) assert.throws(() => configuration(bad));
  const fakeRows = Array.from({ length: 13 }, (_, i) => ({ source: "local" + i, query: "local", newName: "next", private: i < 5, old: i < 5 ? { changes: { "file:///fixture.vue": [{ newText: "next", range: {} }] } } : null, actual: i < 5 ? { changes: { "file:///fixture.vue": [{ newText: "next", range: {} }] } } : null }));
  fakeRows[3] = { ...fakeRows[3]!, source: '<Child :message="message" /> const message = 1', query: "message =" };
  fakeRows[4] = { ...fakeRows[4]!, source: '<Child :message="message" /> const message = 1', query: 'message"' };
  const encode = (rows: unknown[]) => rows.map(row => "VIZE_PRIVATE_RENAME_NATIVE " + JSON.stringify(row)).join("\n");
  pairs(encode(fakeRows));
  for (const mutate of [() => fakeRows.slice(1), () => [...fakeRows, fakeRows[0]], () => fakeRows.map((row, i) => i === 0 ? { ...row, actual: null } : row), () => fakeRows.map((row, i) => i === 0 ? { ...row, old: null, actual: null } : row), () => fakeRows.map((row, i) => i === 4 ? fakeRows[3] : row)]) assert.throws(() => pairs(encode(mutate())));
  const tree = "tree " + "b".repeat(40), single = tree + "\nparent " + BASE;
  parent(single + "\n\nfixture", BASE);
  for (const bad of [tree + "\nparent other\n\nfixture", single + "\nparent " + BASE + "\n\nfixture",
    single + "\nparent\n\nfixture", single, tree + "\n\nroot"]) assert.throws(() => parent(bad, BASE));
  const revision = CANDIDATE, file = "pnpm-lock.yaml";
  const size = Number(git("cat-file", "-s", `${revision}:${file}`));
  assert.equal(size, 1150435); assert(size > 1024 * 1024);
  const actual = execFileSync("git", ["show", `${revision}:${file}`], { maxBuffer: size + 1 });
  completeBlob(revision, file, actual);
  const changed = Buffer.from(actual); changed[0] = (changed.at(0) ?? 0) ^ 1;
  assert.throws(() => completeBlob(revision, file, changed));
  process.stdout.write(JSON.stringify({ executionCases: 6, parentCases: 6, completeBlobCases: 2,
    completeBlobBytes: actual.length, completeBlobSha256: hash(actual), configurationCases: 5, pairCases: 6, passed: 25, qualificationCredit: 0 }) + "\n");
} else {
  const source = env("SOURCE_SHA"), workflow = env("WORKFLOW_SHA"), evidence = fs.realpathSync(env("EVIDENCE"));
  assert.equal(source, CANDIDATE); assert.match(workflow, /^[a-f0-9]{40}$/u); configuration(process.env);
  assert.equal(process.version, "v24.14.0"); assert.equal(process.platform, "linux");
  assert.equal(git("rev-parse", "HEAD"), source); assert.equal(env("EXECUTED_WORKFLOW_SHA"), workflow);
  assert.deepEqual(fs.readFileSync(text(process.argv[1])), execFileSync("git", ["show", `${workflow}:.github/actions/lsp-private-rename-parity/run.ts`]));
  parent(git("cat-file", "-p", source), SOURCE_PARENT); parent(git("cat-file", "-p", SOURCE_PARENT), BASE); parent(git("cat-file", "-p", workflow), source);
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
  const bindingPaths = [...SOURCE_PATHS, ...SDK_PATHS, "crates/vize_maestro/Cargo.toml", "crates/vize_canon/Cargo.toml", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "package.json", "pnpm-lock.yaml", "tools/support/compat/github/full-rust-runtime.ts"];
  const bindings = bindingPaths.map(file => {
    const bytes = fs.readFileSync(file); completeBlob(source, file, bytes);
    return { path: file, sha256: hash(bytes) };
  });
  save("before", { source, workflow, base: BASE, bindings, nativeImage, provider,
    sourceTree: git("rev-parse", source + "^{tree}"), rustc: execFileSync("rustc", ["-Vv"], { encoding: "utf8" }),
    cargo: execFileSync("cargo", ["-V"], { encoding: "utf8" }), qualificationCredit: 0 });
  assert.ok(!fs.existsSync(path.join(env("CARGO_TARGET_DIR"), "ci")), "owned ci build outputs must initially be absent");
  const built = run("build", "cargo", ["test", "--locked", "--profile", "ci", "-p", "vize_maestro", "--lib", PREFIX,
    "--no-run", "--message-format=json-render-diagnostics"], process.env);
  assert.deepEqual([built.code, built.signal, built.error], [0, null, null]);
  const rows = fs.readFileSync(path.join(evidence, "build.stdout"), "utf8").split("\n").filter(Boolean).map(line => object(JSON.parse(line)));
  assert.equal(rows.filter(row => row["reason"] === "build-finished" && row["success"] === true).length, 1);
  const libraries = rows.filter(row => row["reason"] === "compiler-artifact" && object(row["target"])["name"] === "vize_maestro"
    && object(row["profile"])["test"] === true && typeof row["executable"] === "string");
  assert.equal(libraries.length, 1); const library = libraries[0]; assert(library !== undefined);
  const features = library["features"]; assert(Array.isArray(features));
  assert.ok(features.includes("native")); assert(!features.includes("profiling") && !features.includes("micro-profiling"));
  assert.deepEqual(object(library["target"])["kind"], ["lib"]);
  const relativeExecutable = path.relative(fs.realpathSync(env("CARGO_TARGET_DIR")), text(library["executable"]));
  assert.ok(relativeExecutable.startsWith("ci/deps/") && !relativeExecutable.split(path.sep).includes(".."));
  const libraryImage = image(text(library["executable"])); save("library", { source, artifact: library, libraryImage });
  const result = run("test", libraryImage.path, [PREFIX, "--nocapture", "--test-threads=1"], process.env);
  save("after", { source, workflow, nativeImage: image(env("CORSA_PATH")), libraryImage: image(libraryImage.path), result, qualificationCredit: 0 });
  assert.deepEqual(image(env("CORSA_PATH")), nativeImage); assert.deepEqual(image(libraryImage.path), libraryImage);
  assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
  assert.deepEqual([result.code, result.signal, result.error], [0, null, null]);
  execution(fs.readFileSync(path.join(evidence, "test.stdout"), "utf8"));
  const nativePairs = pairs(fs.readFileSync(path.join(evidence, "test.stderr"), "utf8"));
  save("parity", { source, workflow, selectedTests: TESTS, executed: 4, ignored: 0, completeWholePairs: nativePairs, nativeImage, libraryImage, scope: "finite native source unit parity, not CLI/performance/public acceptance" });
}
