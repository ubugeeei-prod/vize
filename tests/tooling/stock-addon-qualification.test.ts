import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { differences } from "../../tools/commands/levels/codegen-equivalence.ts";
import { verifyNativeAddons } from "../../tools/commands/levels/native-addon-identity.ts";
import {
  classifyStockAddonChanges,
  qualifyStockAddon,
  verifyStockRuntimeImages,
} from "../../tools/support/compat/npm/stock-addon-qualification.ts";

const gated = "crates/vize_atelier_sfc/src/bundler/blocks/products.rs";

void test("byte identity only qualifies a closed set of feature-gated changes", () => {
  const feature = { status: "M", path: gated };
  assert.equal(classifyStockAddonChanges([feature]).mode, "byte-identity");
  assert.equal(
    classifyStockAddonChanges([feature, { status: "A", path: "docs/feature.md" }]).mode,
    "byte-identity",
  );
  for (const file of [
    "crates/vize_atelier_sfc/src/parse/native.rs",
    "crates/vize_atelier_sfc/src/compile_script/inline/compiler/model.rs",
    "crates/vize_atelier_sfc/src/bundler/blocks.rs",
    "crates/vize_vitrine/src/napi/sfc/bundler.rs",
    "Cargo.toml",
    "Cargo.lock",
    "crates/vize_vitrine/build.rs",
    "tools/support/compat/npm/stock-addon-rustc.rs",
    "unknown/production.rs",
    "docs/input.vue",
  ]) {
    const change = { status: "M", path: file };
    const actual = classifyStockAddonChanges([feature, change]);
    assert.equal(actual.mode, "stock-runtime", file);
    assert.deepEqual(actual.production, [change]);
  }
  for (const status of ["D", "T", "R100"])
    assert.equal(classifyStockAddonChanges([{ status, path: gated }]).mode, "stock-runtime");
});

function repository(run: (root: string, git: (...args: string[]) => string) => void) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "stock-addon-qualification-"));
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
  try {
    git("init", "-q");
    git("config", "user.email", "fixture@example.invalid");
    git("config", "user.name", "Fixture");
    git("config", "commit.gpgsign", "false");
    run(root, git);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

void test("the Git receipt retains deleted and added paths of a rename", () => {
  repository((root, git) => {
    fs.mkdirSync(path.dirname(path.join(root, gated)), { recursive: true });
    fs.writeFileSync(path.join(root, gated), "gated source\n");
    git("add", ".");
    git("commit", "-qm", "base");
    const base = git("rev-parse", "HEAD");
    git("mv", gated, "production with spaces.rs");
    git("commit", "-qm", "renamed outside the feature");
    const receipt = qualifyStockAddon(root, base, git("rev-parse", "HEAD"));
    assert.deepEqual(receipt.changes, [
      { status: "D", path: gated },
      { status: "A", path: "production with spaces.rs" },
    ]);
    assert.equal(receipt.mode, "stock-runtime");
    assert.deepEqual(receipt.production, receipt.changes);
  });
});

void test("runtime qualification authenticates images and rejects forged scope or build metadata", () => {
  repository((root, git) => {
    fs.writeFileSync(path.join(root, "production.rs"), "before\n");
    fs.writeFileSync(path.join(root, "Cargo.lock"), "base locked dependencies\n");
    git("add", ".");
    git("commit", "-qm", "base");
    const base = git("rev-parse", "HEAD");
    fs.writeFileSync(path.join(root, "production.rs"), "after\n");
    fs.writeFileSync(path.join(root, "Cargo.lock"), "head locked dependencies\n");
    git("commit", "-qam", "production fix");
    const head = git("rev-parse", "HEAD");
    const side = (name: string, revision: string) => {
      const pluginHostBuildId = revision + ":" + "a".repeat(64);
      const bytes = Buffer.from(name + " changed machine code / " + pluginHostBuildId);
      fs.writeFileSync(path.join(root, name), bytes);
      return {
        revision,
        pluginHostBuildId,
        manifest: "crates/vize_vitrine/Cargo.toml",
        target: { name: "vize_vitrine", kind: ["cdylib"], source: "src/lib.rs" },
        lock: createHash("sha256")
          .update(execFileSync("git", ["show", `${revision}:Cargo.lock`], { cwd: root }))
          .digest("hex"),
        cargo: "cargo pinned",
        rustc: "rustc pinned",
        features: ["glyph", "legacy", "napi"],
        profile: {},
        members: [
          {
            key: "node-addon",
            name: "vize_vitrine",
            bytes: bytes.length,
            sha256: createHash("sha256").update(bytes).digest("hex"),
          },
        ],
      };
    };
    const report = {
      package: "vize_vitrine",
      product: "node-addon",
      base: side("base", base),
      head: side("head", head),
      differences: ["node-addon"],
      executables: { base: "base", head: "head" },
    };
    report.differences = differences(report.base, report.head);
    const file = path.join(root, "report.json");
    const receipt = path.join(root, "qualification.json");
    const qualification = qualifyStockAddon(root, base, head);
    const write = () => {
      fs.writeFileSync(file, JSON.stringify(report));
      fs.writeFileSync(receipt, JSON.stringify(qualification));
    };
    const verify = () => verifyStockRuntimeImages(file, receipt, root);
    write();
    assert.equal(verify().head, path.join(root, "head"));
    assert.notEqual(report.base.lock, report.head.lock);
    assert.throws(() => verifyNativeAddons(file));
    report.head.lock = "0".repeat(64);
    report.differences = differences(report.base, report.head);
    write();
    assert.throws(verify);
    report.head = side("head", head);
    report.differences = differences(report.base, report.head);
    write();
    fs.appendFileSync(path.join(root, "head"), "foreign suffix");
    assert.throws(verify);
    report.head = side("head", head);
    write();
    for (const mutation of [
      { ...qualification, base: head },
      { ...qualification, mode: "byte-identity" },
      { ...qualification, changes: [] },
      { ...qualification, production: [] },
    ]) {
      fs.writeFileSync(receipt, JSON.stringify(mutation));
      assert.throws(verify);
    }
    write();
    report.head.pluginHostBuildId = head + ":" + "b".repeat(64);
    write();
    assert.throws(verify, /missing native cache identity/u);
    report.head = side("head", head);
    report.head.rustc = "foreign compiler";
    report.differences = differences(report.base, report.head);
    write();
    assert.throws(verify);

    fs.mkdirSync(path.dirname(path.join(root, gated)), { recursive: true });
    fs.writeFileSync(path.join(root, gated), "feature only\n");
    git("add", gated);
    git("commit", "-qm", "isolated feature");
    const featureHead = git("rev-parse", "HEAD");
    report.base = side("base", head);
    report.head = side("head", featureHead);
    report.differences = differences(report.base, report.head);
    fs.writeFileSync(file, JSON.stringify(report));
    fs.writeFileSync(receipt, JSON.stringify(qualifyStockAddon(root, head, featureHead)));
    assert.throws(verify, /byte-identity/u);
  });
});
