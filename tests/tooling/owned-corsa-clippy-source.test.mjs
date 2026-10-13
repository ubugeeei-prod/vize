import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
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
import { authenticateOwnedCorsaPackageSource } from "../../tools/support/compat/github/owned-corsa-package-source.mjs";
import { nativeLexicalCompositionBodyPairs } from "../performance/support/warm-type-backed-native-lexical-composition-host.ts";
const originalRoot = process.cwd();
const env = { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_ALLOW_PROTOCOL: "" };
const git = (root, args, input) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd: root, input, env });
const context = JSON.parse(
  fs.readFileSync(
    new URL("../../tools/support/compat/github/owned-corsa-clippy-source.json", import.meta.url),
  ),
);

function fixture(run) {
  ensureHostedLiteralSource(originalRoot, ownedCorsaSource.baseline);
  ensureHostedLiteralSource(originalRoot, context.retainedOldAfterSource);
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vendor-source-"));
  try {
    let head = git(originalRoot, ["rev-parse", "HEAD"]).toString().trim();
    const reader = ownedCorsaReader(originalRoot, ownedCorsaSource.baseline, head);
    for (const row of ownedCorsaSource.rows) {
      const target = path.join(root, row.file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, reader("after", row.file).body, { mode: 0o644, flag: "wx" });
    }
    git(root, ["init", "--quiet"]);
    git(root, ["config", "user.name", "Vendor source control"]);
    git(root, ["config", "user.email", "source@example.invalid"]);
    const common = git(originalRoot, ["rev-parse", "--git-common-dir"]).toString().trim();
    fs.writeFileSync(
      path.join(root, ".git/objects/info/alternates"),
      path.resolve(originalRoot, common, "objects") + "\n",
    );
    git(root, ["update-ref", "refs/heads/current", head]);
    git(root, ["symbolic-ref", "HEAD", "refs/heads/current"]);
    // Reconstruct the retained old class even when the current checkout contains the new class.
    for (const row of context.entries) {
      const old = Buffer.from(row.retainedOldAfter.rawBase64, "base64");
      fs.writeFileSync(path.join(root, row.file), old);
      head = sourceCommit(root, head, row.file, old);
    }
    for (const row of context.sourceCorrections) {
      const old = Buffer.from(row.retainedOldAfter.rawBase64, "base64");
      fs.writeFileSync(path.join(root, row.file), old);
      head = sourceCommit(root, head, row.file, old);
    }
    const testContext = JSON.parse(
      fs.readFileSync(
        new URL("../../tools/support/compat/github/owned-corsa-test-source.json", import.meta.url),
      ),
    );
    for (const row of testContext.entries) {
      const old = Buffer.from(row.retainedBefore.rawBase64, "base64");
      fs.writeFileSync(path.join(root, row.file), old);
      head = sourceCommit(root, head, row.file, old);
    }
    run(root, head);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}
function sourceCommit(root, head, file, body) {
  git(root, ["read-tree", head]);
  const blob = git(root, ["hash-object", "-w", "--stdin"], body).toString().trim();
  git(root, ["update-index", "--add", "--cacheinfo", `100644,${blob},${file}`]);
  const tree = git(root, ["write-tree"]).toString().trim();
  const commit = git(root, ["commit-tree", tree, "-p", head], "private source control\n")
    .toString()
    .trim();
  git(root, ["update-ref", "refs/heads/current", commit]);
  return commit;
}

function qualify(root) {
  const head = git(root, ["rev-parse", "HEAD"]).toString().trim();
  return qualifyOwnedCorsaSource({
    baseline: ownedCorsaSource.baseline,
    production: ownedCorsaSource.rows.map(({ file }) => file),
    alreadyQualified: new Set(),
    reader: ownedCorsaReader(root, ownedCorsaSource.baseline, head),
    historicalPairs: nativeLexicalCompositionBodyPairs,
  });
}
function current(root, head) {
  for (const row of context.entries) {
    const body = Buffer.from(row.currentAfter.rawBase64, "base64");
    fs.writeFileSync(path.join(root, row.file), body);
    head = sourceCommit(root, head, row.file, body);
  }
  return head;
}

void test("both complete real237 classes preserve old capsules and current six source identities", () => {
  fixture((root, head) => {
    const retained = qualify(root);
    assert.equal(retained.completeProductionBodies, 237);
    assert.equal(retained.sourceContext, undefined);
    const after = current(root, head);
    const value = qualify(root);
    assert.equal(value.completeProductionBodies, 237);
    assert.equal(value.sourceContext.wholeManifestBodies, 6);
    assert.equal(value.sourceContext.clippyExecutionCredit, false);
    for (const row of context.entries) {
      const old = Buffer.from(row.retainedOldAfter.rawBase64, "base64");
      const now = Buffer.from(row.currentAfter.rawBase64, "base64");
      assert.deepEqual(Buffer.concat([old, Buffer.from(context.appendix)]), now);
      assert.deepEqual(value.actualReader("after", row.file).body, now);
    }
    const source = authenticateOwnedCorsaPackageSource(root, after);
    assert.equal(source.actualBodies.length, 237);
    for (const row of context.entries) {
      const actual = source.actualBodies.find(({ file }) => file === row.file);
      assert.equal(actual.sha256, row.currentAfter.sha256);
      assert.notEqual(actual.sha256, row.retainedOldAfter.sha256);
    }
    const capsule = value.historicalReader("after", "vendor/corsa_core/Cargo.toml");
    assert.equal(
      createHash("sha256").update(capsule.body).digest("hex"),
      "b92c9f94f932a73e6d34e93c8cea493b744322b5c4892d6bf0b79654fd0c7e13",
    );
  });
});

void test("every real six-manifest partial prefix and mixed old/current class refuses", () => {
  fixture((root, head) => {
    for (let count = 0; count < context.entries.length; count++) {
      const row = context.entries[count];
      const body = Buffer.from(row.currentAfter.rawBase64, "base64");
      fs.writeFileSync(path.join(root, row.file), body);
      head = sourceCommit(root, head, row.file, body);
      if (count < context.entries.length - 1) assert.equal(qualify(root), null);
    }
    const coherent = head;
    for (const row of context.entries) {
      const old = Buffer.from(row.retainedOldAfter.rawBase64, "base64");
      fs.writeFileSync(path.join(root, row.file), old);
      sourceCommit(root, coherent, row.file, old);
      assert.equal(qualify(root), null);
      git(root, ["update-ref", "refs/heads/current", coherent]);
      fs.writeFileSync(
        path.join(root, row.file),
        Buffer.from(row.currentAfter.rawBase64, "base64"),
      );
    }
  });
});

void test("real unknown policy body, physical mode/missing/symlink and replacement refs refuse", () => {
  fixture((root, head) => {
    head = current(root, head);
    const row = context.entries[0];
    const file = path.join(root, row.file);
    const body = fs.readFileSync(file);
    const unknown = Buffer.concat([body, Buffer.from('disallowed_future = "allow"\n')]);
    fs.writeFileSync(file, unknown);
    sourceCommit(root, head, row.file, unknown);
    assert.equal(qualify(root), null);
    git(root, ["update-ref", "refs/heads/current", head]);
    fs.writeFileSync(file, body);
    fs.chmodSync(file, 0o755);
    assert.throws(() => qualify(root));
    fs.chmodSync(file, 0o644);
    fs.unlinkSync(file);
    assert.throws(() => qualify(root));
    fs.symlinkSync(path.join(originalRoot, row.file), file);
    assert.throws(() => qualify(root));
    fs.unlinkSync(file);
    fs.writeFileSync(file, body, { mode: 0o644 });
    const extra = "vendor/corsa_core/unknown-source.rs";
    const foreign = sourceCommit(root, head, extra, Buffer.from("foreign source\n"));
    assert.throws(
      () => authenticateOwnedCorsaPackageSource(root, foreign),
      /complete six-package raw source census/u,
    );
    git(root, ["update-ref", "refs/heads/current", head]);
    git(root, ["update-ref", "refs/replace/" + head, foreign]);
    assert.throws(() => qualify(root), /replacement refs refuse/u);
  });
});

void test("physical metadata body/mode/missing/symlink controls refuse before module qualification", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-clippy-data-"));
  try {
    const helper = fs.readFileSync(
      new URL("../../tools/support/compat/github/owned-corsa-clippy-source.mjs", import.meta.url),
    );
    const body = fs.readFileSync(
      new URL("../../tools/support/compat/github/owned-corsa-clippy-source.json", import.meta.url),
    );
    fs.writeFileSync(path.join(root, "owned-corsa-clippy-source.mjs"), helper);
    for (const name of ["owned-corsa-test-source.mjs", "owned-corsa-test-source.json"])
      fs.copyFileSync(
        new URL("../../tools/support/compat/github/" + name, import.meta.url),
        path.join(root, name),
      );
    const target = path.join(root, "owned-corsa-clippy-source.json");
    const run = () =>
      spawnSync(
        process.execPath,
        [
          "--input-type=module",
          "-e",
          `await import(${JSON.stringify(path.join(root, "owned-corsa-clippy-source.mjs"))})`,
        ],
        { env, encoding: "utf8" },
      );
    fs.writeFileSync(target, body, { mode: 0o644 });
    assert.equal(run().status, 0);
    fs.writeFileSync(target, Buffer.concat([body, Buffer.from("\n")]));
    assert.equal(run().status, 1);
    fs.writeFileSync(target, body);
    fs.chmodSync(target, 0o755);
    assert.equal(run().status, 1);
    fs.unlinkSync(target);
    assert.equal(run().status, 1);
    fs.symlinkSync(
      new URL("../../tools/support/compat/github/owned-corsa-clippy-source.json", import.meta.url),
      target,
    );
    assert.equal(run().status, 1);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
