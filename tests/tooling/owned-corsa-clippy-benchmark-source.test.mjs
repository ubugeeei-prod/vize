import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { ensureHostedLiteralSource } from "../performance/support/warm-type-backed-git-reader-literal-source.ts";
import { ownedCorsaSource } from "../../tools/support/compat/github/owned-corsa-source-custody.mjs";
import { ownedCorsaReader } from "../../tools/support/compat/github/owned-corsa-source-reader.mjs";
import { authenticateOwnedCorsaAfterContext } from "../../tools/support/compat/github/owned-corsa-clippy-source.mjs";
import { authenticateOwnedCorsaPackageSource } from "../../tools/support/compat/github/owned-corsa-package-source.mjs";

const actualRoot = process.cwd();
const env = { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_ALLOW_PROTOCOL: "" };
const git = (root, args, input) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd: root, input, env });
const sha = (body) => createHash("sha256").update(body).digest("hex");
const context = JSON.parse(
  fs.readFileSync(
    new URL("../../tools/support/compat/github/owned-corsa-clippy-source.json", import.meta.url),
  ),
);
assert.equal(context.sourceCorrections.length, 2);
const [handles, benchmark] = context.sourceCorrections;
assert.equal(benchmark.file, "vendor/corsa/src/bin/bench_real_corsa/args.rs");
const body = (row, current) =>
  Buffer.from(row[current ? "currentAfter" : "retainedOldAfter"].rawBase64, "base64");
function write(root, head, file, bytes) {
  fs.writeFileSync(path.join(root, file), bytes);
  git(root, ["read-tree", head]);
  const blob = git(root, ["hash-object", "-w", "--stdin"], bytes).toString().trim();
  git(root, ["update-index", "--add", "--cacheinfo", `100644,${blob},${file}`]);
  const tree = git(root, ["write-tree"]).toString().trim();
  const commit = git(root, ["commit-tree", tree, "-p", head], "private benchmark source control\n")
    .toString()
    .trim();
  git(root, ["update-ref", "refs/heads/current", commit]);
  return commit;
}
function fixture(run) {
  ensureHostedLiteralSource(actualRoot, ownedCorsaSource.baseline);
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-clippy-benchmark-"));
  try {
    let head = git(actualRoot, ["rev-parse", "HEAD"]).toString().trim();
    const reader = ownedCorsaReader(actualRoot, ownedCorsaSource.baseline, head);
    for (const { file } of ownedCorsaSource.rows) {
      const target = path.join(root, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, reader("after", file).body, { flag: "wx", mode: 0o644 });
    }
    git(root, ["init", "--quiet"]);
    git(root, ["config", "user.name", "Benchmark source control"]);
    git(root, ["config", "user.email", "source@example.invalid"]);
    const common = git(actualRoot, ["rev-parse", "--git-common-dir"]).toString().trim();
    fs.writeFileSync(
      path.join(root, ".git/objects/info/alternates"),
      path.resolve(actualRoot, common, "objects") + "\n",
    );
    git(root, ["update-ref", "refs/heads/current", head]);
    git(root, ["symbolic-ref", "HEAD", "refs/heads/current"]);
    for (const row of [...context.entries, ...context.sourceCorrections])
      head = write(root, head, row.file, body(row, false));
    run(root, head);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}
function authenticate(root) {
  const head = git(root, ["rev-parse", "HEAD"]).toString().trim();
  const reader = ownedCorsaReader(root, ownedCorsaSource.baseline, head);
  return authenticateOwnedCorsaAfterContext(ownedCorsaSource.rows, (file) => reader("after", file));
}
function select(root, head) {
  git(root, ["update-ref", "refs/heads/current", head]);
}

void test("the benchmark block and let recover the complete archive body and preserve every outer-order byte", () => {
  assert.equal(benchmark.clauses.length, 1);
  const [before, after] = benchmark.clauses[0];
  assert.equal(before, '        for path in [base.join("packages/typescript/tsconfig.json")] {');
  assert.equal(
    after,
    '        {\n            let path = base.join("packages/typescript/tsconfig.json");',
  );
  const old = body(benchmark, false),
    current = body(benchmark, true);
  assert.equal(old.toString().split(before).length, 2);
  assert.equal(current.toString().split(after).length, 2);
  assert.deepEqual(Buffer.from(old.toString().replace(before, after)), current);
  assert.deepEqual(Buffer.from(current.toString().replace(after, before)), old);
  assert.equal(sha(old), benchmark.retainedArchiveMemberSHA256);
  const row = ownedCorsaSource.rows.find(({ file }) => file === benchmark.file);
  assert.equal(row.archiveSHA256, benchmark.retainedArchiveSHA256);
  assert.equal(row.archiveMemberSHA256, sha(old));
  assert.equal(row.after.sha256, sha(old));
});

void test("all four full237 classes remain distinct and the current package stages both actual corrected bodies", () => {
  fixture((root, head) => {
    assert.equal(authenticate(root).context, null);
    for (const row of context.entries) head = write(root, head, row.file, body(row, true));
    assert.equal(authenticate(root).context.wholeCorrectedRustBodies, 0);
    head = write(root, head, handles.file, body(handles, true));
    const retained = authenticate(root);
    assert.equal(retained.context.wholeCorrectedRustBodies, 1);
    assert.deepEqual(retained.context.correctedSourceFiles, [handles.file]);
    assert.deepEqual(retained.actualAfter(benchmark.file).body, body(benchmark, false));
    head = write(root, head, benchmark.file, body(benchmark, true));
    const current = authenticate(root);
    assert.equal(current.context.wholeCurrentBodies, 237);
    assert.equal(current.context.wholeManifestBodies, 6);
    assert.equal(current.context.wholeCorrectedRustBodies, 2);
    assert.deepEqual(current.context.correctedSourceFiles, [handles.file, benchmark.file]);
    assert.equal(current.context.clippyExecutionCredit, false);
    const packaged = authenticateOwnedCorsaPackageSource(root, head);
    assert.equal(packaged.actualBodies.length, 237);
    for (const row of context.sourceCorrections) {
      assert.deepEqual(current.actualAfter(row.file).body, body(row, true));
      assert.deepEqual(current.historicalAfter(row.file).body, body(row, false));
      const actual = packaged.actualBodies.find(({ file }) => file === row.file);
      assert.deepEqual(actual, {
        file: row.file,
        mode: "100644",
        blob: row.currentAfter.blob,
        bytes: row.currentAfter.bytes,
        sha256: row.currentAfter.sha256,
      });
    }
    const snapshot = current.historicalAfter(benchmark.file).body;
    snapshot[0] ^= 1;
    assert.deepEqual(current.historicalAfter(benchmark.file).body, body(benchmark, false));
  });
});

void test("args-only, old manifests with repairs, partial, foreign, missing, mode, symlink and replacements refuse", () => {
  fixture((root, head) => {
    write(root, head, benchmark.file, body(benchmark, true));
    assert.equal(authenticate(root), null, "old six with args only");
    head = write(root, head, benchmark.file, body(benchmark, false));
    for (const row of context.entries) head = write(root, head, row.file, body(row, true));
    write(root, head, benchmark.file, body(benchmark, true));
    assert.equal(authenticate(root), null, "new six with args only");
    head = write(root, head, benchmark.file, body(benchmark, false));
    head = write(root, head, handles.file, body(handles, true));
    const full = write(root, head, benchmark.file, body(benchmark, true));
    for (const bytes of [
      Buffer.from(body(benchmark, true).toString().replace("let path =", "let other =")),
      Buffer.from(
        body(benchmark, true).toString().replace("datasets.push(path);", "datasets.push(base);"),
      ),
      Buffer.concat([body(benchmark, true), Buffer.from("\n")]),
    ]) {
      write(root, full, benchmark.file, bytes);
      assert.equal(authenticate(root), null);
    }
    select(root, full);
    fs.writeFileSync(path.join(root, benchmark.file), body(benchmark, true));
    head = full;
    for (const row of context.entries) head = write(root, head, row.file, body(row, false));
    assert.equal(authenticate(root), null, "old six with both repairs");
    select(root, full);
    for (const row of context.entries) fs.writeFileSync(path.join(root, row.file), body(row, true));
    head = full;
    for (const row of context.entries.slice(0, 1))
      head = write(root, head, row.file, body(row, false));
    assert.equal(authenticate(root), null, "partial six with both repairs");
    select(root, full);
    for (const row of [...context.entries, benchmark])
      fs.writeFileSync(path.join(root, row.file), body(row, true));
    const target = path.join(root, benchmark.file);
    fs.chmodSync(target, 0o755);
    assert.throws(() => authenticate(root));
    fs.chmodSync(target, 0o644);
    fs.unlinkSync(target);
    assert.throws(() => authenticate(root));
    fs.symlinkSync(path.join(actualRoot, benchmark.file), target);
    assert.throws(() => authenticate(root));
    fs.unlinkSync(target);
    fs.writeFileSync(target, body(benchmark, true), { mode: 0o644 });
    const replacement = write(root, full, benchmark.file, body(benchmark, false));
    select(root, full);
    fs.writeFileSync(target, body(benchmark, true));
    git(root, ["update-ref", "refs/replace/" + full, replacement]);
    assert.throws(() => authenticate(root), /replacement refs refuse/u);
  });
});
