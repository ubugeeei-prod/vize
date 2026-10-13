import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { ensureHostedLiteralSource } from "../performance/support/warm-type-backed-git-reader-literal-source.ts";
import {
  ownedCorsaSource,
  qualifyOwnedCorsaSource,
} from "../../tools/support/compat/github/owned-corsa-source-custody.mjs";
import { ownedCorsaReader } from "../../tools/support/compat/github/owned-corsa-source-reader.mjs";
import {
  authenticateOwnedCorsaAfterContext,
  assertOwnedCorsaAfterContext,
} from "../../tools/support/compat/github/owned-corsa-clippy-source.mjs";
import { authenticateOwnedCorsaPackageSource } from "../../tools/support/compat/github/owned-corsa-package-source.mjs";
import { nativeLexicalCompositionBodyPairs as historicalPairs } from "../performance/support/warm-type-backed-native-lexical-composition-host.ts";

const actualRoot = process.cwd();
const context = JSON.parse(
  fs.readFileSync(
    new URL("../../tools/support/compat/github/owned-corsa-clippy-source.json", import.meta.url),
  ),
);
const correction = context.sourceCorrections[0];
assert.equal(context.sourceCorrections.length, 2);
const oldBody = Buffer.from(correction.retainedOldAfter.rawBase64, "base64");
const newBody = Buffer.from(correction.currentAfter.rawBase64, "base64");
const sha = (body) => createHash("sha256").update(body).digest("hex");
const env = { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_ALLOW_PROTOCOL: "" };
const git = (root, args, input) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd: root, input, env });
assert.equal(historicalPairs.length, 121);
function commit(root, head, file, body) {
  git(root, ["read-tree", head]);
  const blob = git(root, ["hash-object", "-w", "--stdin"], body).toString().trim();
  git(root, ["update-index", "--add", "--cacheinfo", `100644,${blob},${file}`]);
  const tree = git(root, ["write-tree"]).toString().trim();
  const next = git(root, ["commit-tree", tree, "-p", head], "private handles source control\n")
    .toString()
    .trim();
  git(root, ["update-ref", "refs/heads/current", next]);
  return next;
}
function write(root, head, file, body) {
  fs.writeFileSync(path.join(root, file), body);
  return commit(root, head, file, body);
}
function fixture(run) {
  ensureHostedLiteralSource(actualRoot, ownedCorsaSource.baseline);
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-clippy-handles-"));
  try {
    let head = git(actualRoot, ["rev-parse", "HEAD"]).toString().trim();
    const reader = ownedCorsaReader(actualRoot, ownedCorsaSource.baseline, head);
    for (const { file } of ownedCorsaSource.rows) {
      const target = path.join(root, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, reader("after", file).body, { flag: "wx", mode: 0o644 });
    }
    git(root, ["init", "--quiet"]);
    git(root, ["config", "user.name", "Handles source control"]);
    git(root, ["config", "user.email", "source@example.invalid"]);
    const common = git(actualRoot, ["rev-parse", "--git-common-dir"]).toString().trim();
    fs.writeFileSync(
      path.join(root, ".git/objects/info/alternates"),
      path.resolve(actualRoot, common, "objects") + "\n",
    );
    git(root, ["update-ref", "refs/heads/current", head]);
    git(root, ["symbolic-ref", "HEAD", "refs/heads/current"]);
    for (const row of context.entries)
      head = write(root, head, row.file, Buffer.from(row.currentAfter.rawBase64, "base64"));
    // This predecessor campaign deliberately retains the whole old benchmark body.
    for (const row of context.sourceCorrections.slice(1))
      head = write(root, head, row.file, Buffer.from(row.retainedOldAfter.rawBase64, "base64"));
    head = write(root, head, correction.file, oldBody);
    run(root, head);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}
function qualify(root, extra = []) {
  const head = git(root, ["rev-parse", "HEAD"]).toString().trim();
  return qualifyOwnedCorsaSource({
    baseline: ownedCorsaSource.baseline,
    production: [...ownedCorsaSource.rows.map(({ file }) => file), ...extra],
    alreadyQualified: new Set(),
    reader: ownedCorsaReader(root, ownedCorsaSource.baseline, head),
    historicalPairs,
  });
}
function authenticate(root) {
  const head = git(root, ["rev-parse", "HEAD"]).toString().trim();
  const reader = ownedCorsaReader(root, ownedCorsaSource.baseline, head);
  return authenticateOwnedCorsaAfterContext(ownedCorsaSource.rows, (file) => reader("after", file));
}
function select(root, head) {
  git(root, ["update-ref", "refs/heads/current", head]);
}

void test("the complete four-call inverse preserves original archive identity and every other handles byte", () => {
  assert.equal(correction.clauses.length, 4);
  let forward = oldBody.toString();
  for (const [before, after] of correction.clauses) {
    assert.equal(forward.split(before).length - 1, 1);
    assert.equal(before.replace("&invalid", "invalid"), after);
    forward = forward.replace(before, after);
  }
  assert.deepEqual(Buffer.from(forward), newBody);
  assert.equal(oldBody.length - newBody.length, 4);
  let inverse = newBody.toString();
  for (const [before, after] of [...correction.clauses].reverse())
    inverse = inverse.replace(after, before);
  assert.deepEqual(Buffer.from(inverse), oldBody);
  assert.equal(sha(oldBody), correction.retainedArchiveMemberSHA256);
  const immutable = ownedCorsaSource.rows.find(({ file }) => file === correction.file);
  assert.equal(immutable.archiveSHA256, correction.retainedArchiveSHA256);
  assert.equal(immutable.after.sha256, sha(oldBody));
});

void test("only the complete corrected body with all six current manifests qualifies and actual source stays corrected", () => {
  fixture((root, head) => {
    const retainedSix = qualify(root);
    assert.equal(retainedSix.sourceContext.wholeCorrectedRustBodies, 0);
    const corrected = write(root, head, correction.file, newBody);
    const value = qualify(root);
    assert.equal(value.completeProductionBodies, 237);
    assert.equal(value.sourceContext.wholeManifestBodies, 6);
    assert.equal(value.sourceContext.wholeCorrectedRustBodies, 1);
    assert.deepEqual(value.sourceContext.correctedSourceFiles, [correction.file]);
    assert.equal(value.sourceContext.clippyExecutionCredit, false);
    assert.deepEqual(value.actualReader("after", correction.file).body, newBody);
    assert.deepEqual(value.historicalReader("after", correction.file).body, oldBody);
    const packagedSource = authenticateOwnedCorsaPackageSource(root, corrected);
    const packagedHandle = packagedSource.actualBodies.find(({ file }) => file === correction.file);
    assert.equal(packagedHandle.sha256, correction.currentAfter.sha256);
    assert.equal(packagedHandle.bytes, newBody.length);
    assert.notEqual(packagedHandle.sha256, correction.retainedOldAfter.sha256);
    for (const row of context.entries) {
      const original = Buffer.from(row.retainedOldAfter.rawBase64, "base64");
      head = write(root, corrected, row.file, original);
      assert.equal(authenticate(root), null, row.file);
      select(root, corrected);
      fs.writeFileSync(
        path.join(root, row.file),
        Buffer.from(row.currentAfter.rawBase64, "base64"),
      );
    }
    head = corrected;
    for (const row of context.entries)
      head = write(root, head, row.file, Buffer.from(row.retainedOldAfter.rawBase64, "base64"));
    assert.equal(authenticate(root), null, "all six old manifests with corrected source refuse");
    write(root, head, correction.file, oldBody);
    assert(qualify(root), "the entire original class remains admitted");
  });
});

void test("every partial repair, each restored extra borrow and foreign whole source refuses", () => {
  fixture((root, head) => {
    for (let mask = 1; mask < 15; mask++) {
      let partial = oldBody.toString();
      for (let i = 0; i < correction.clauses.length; i++)
        if (mask & (1 << i)) partial = partial.replace(...correction.clauses[i]);
      write(root, head, correction.file, Buffer.from(partial));
      assert.equal(authenticate(root), null, `partial repair ${mask}`);
      select(root, head);
    }
    for (const [before, after] of correction.clauses) {
      write(root, head, correction.file, Buffer.from(newBody.toString().replace(after, before)));
      assert.equal(authenticate(root), null, after);
      select(root, head);
    }
    for (const body of [
      Buffer.concat([newBody, Buffer.from("\n")]),
      Buffer.from(newBody.toString().replace("InvalidHandle", "ForeignHandle")),
    ]) {
      write(root, head, correction.file, body);
      assert.equal(authenticate(root), null);
      select(root, head);
    }
  });
});

void test("missing, physical-mode, symlink and unknown source controls refuse before projection", () => {
  fixture((root, head) => {
    head = write(root, head, correction.file, newBody);
    const target = path.join(root, correction.file);
    fs.chmodSync(target, 0o755);
    assert.throws(() => authenticate(root));
    fs.chmodSync(target, 0o644);
    fs.unlinkSync(target);
    assert.throws(() => authenticate(root));
    fs.symlinkSync(path.join(actualRoot, correction.file), target);
    assert.throws(() => authenticate(root));
    fs.unlinkSync(target);
    fs.writeFileSync(target, newBody, { mode: 0o644 });
    const extra = "vendor/corsa_client/src/api/unknown.rs";
    fs.writeFileSync(path.join(root, extra), Buffer.from("unknown source\n"));
    commit(root, head, extra, Buffer.from("unknown source\n"));
    assert.equal(qualify(root, [extra]), null);
    fs.unlinkSync(path.join(root, extra));
    select(root, head);
    assert(qualify(root));
  });
});

void test("a known full class reports every genuine mismatched provider path without accepting it", () => {
  fixture((root, head) => {
    head = write(root, head, correction.file, newBody);
    const files = ["vendor/corsa_core/src/lib.rs", "vendor/corsa_lsp/src/lib.rs"];
    for (const file of files)
      head = write(
        root,
        head,
        file,
        Buffer.concat([fs.readFileSync(path.join(root, file)), Buffer.from("\n")]),
      );
    const reader = ownedCorsaReader(root, ownedCorsaSource.baseline, head);
    const read = (file) => reader("after", file);
    assert.equal(authenticateOwnedCorsaAfterContext(ownedCorsaSource.rows, read), null);
    assert.throws(
      () => assertOwnedCorsaAfterContext(ownedCorsaSource.rows, read),
      (error) => {
        assert.match(error.message, /corsa_core\/src\/lib\.rs/u);
        assert.match(error.message, /corsa_lsp\/src\/lib\.rs/u);
        return true;
      },
    );
  });
});
