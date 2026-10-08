import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  BUILD_RECIPE,
  LEGACY_BUILD_RECIPE,
  SHIPPING_BUILD_RECIPE,
  writeBuildReceipt,
} from "../differential/build-receipt.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";

function fixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-source-binding-"));
  const git = (...args: string[]) => execFileSync("git", args, { cwd: root });
  git("init", "-q");
  git("config", "user.name", "CI Test");
  git("config", "user.email", "ci@example.invalid");
  fs.writeFileSync(path.join(root, "Cargo.toml"), '[workspace.package]\nversion = "1.0.0"\n');
  git("add", ".");
  git("commit", "-qm", "source revision");
  for (const [profile, identity] of [
    ["debug", "old-debug"],
    ["ci", "fresh-source"],
  ]) {
    const binary = path.join(root, "target", profile, "vize");
    fs.mkdirSync(path.dirname(binary), { recursive: true });
    fs.writeFileSync(
      binary,
      `#!/bin/sh\nif [ "$1" = "--version" ]; then echo 'vize 1.0.0'; else echo '${identity}'; fi\n`,
      { mode: 0o755 },
    );
  }
  const binary = path.join(root, "target/ci/vize");
  const receipt = writeBuildReceipt(root);
  const probed: string[] = [];
  const resolve = (bound = binary, buildRecipe = BUILD_RECIPE) =>
    resolveVizeLaunchCommand(
      (candidate) => {
        probed.push(candidate);
        return spawnSync(candidate, ["--version"]).status === 0;
      },
      bound,
      { required: true, repoRoot: root, buildRecipe },
    );
  return { root, binary, receipt, probed, resolve };
}

test("required LSP source binding executes the receipted CI binary despite a runnable old debug binary", () => {
  const f = fixture();
  try {
    assert.equal(
      execFileSync(path.join(f.root, "target/debug/vize"), ["lsp"], { encoding: "utf8" }).trim(),
      "old-debug",
    );
    const [binary, ...args] = f.resolve();
    assert.equal(execFileSync(binary, args, { encoding: "utf8" }).trim(), "fresh-source");
    assert.deepEqual(f.probed, [f.binary]);
    assert.throws(() => f.resolve(path.join(f.root, "target/debug/vize")));
    assert.deepEqual(f.probed, [f.binary]);
  } finally {
    fs.rmSync(f.root, { recursive: true, force: true });
  }
});

test("legacy LSP source binding requires its closed recipe and keeps the default contract strict", () => {
  const f = fixture();
  try {
    assert.throws(() => f.resolve(f.binary, LEGACY_BUILD_RECIPE));
    writeBuildReceipt(f.root, LEGACY_BUILD_RECIPE);
    assert.throws(() => f.resolve());
    assert.throws(
      () => f.resolve(f.binary, "cargo build --release"),
      /unknown source-build recipe/,
    );
    assert.throws(
      () => writeBuildReceipt(f.root, "cargo build --release"),
      /unknown source-build recipe/,
    );
    assert.deepEqual(f.probed, []);
    const [binary, ...args] = f.resolve(f.binary, LEGACY_BUILD_RECIPE);
    assert.equal(execFileSync(binary, args, { encoding: "utf8" }).trim(), "fresh-source");
    assert.deepEqual(f.probed, [f.binary]);
    const receipt = JSON.parse(fs.readFileSync(f.receipt, "utf8"));
    receipt.recipe = BUILD_RECIPE;
    fs.writeFileSync(f.receipt, JSON.stringify(receipt));
    assert.throws(() => f.resolve(f.binary, LEGACY_BUILD_RECIPE));
    assert.deepEqual(f.probed, [f.binary]);
  } finally {
    fs.rmSync(f.root, { recursive: true, force: true });
  }
});

test("shipping recipe binding rejects a CI receipt and still refuses stale staged bytes", () => {
  const f = fixture();
  try {
    assert.throws(() => f.resolve(f.binary, SHIPPING_BUILD_RECIPE));
    assert.deepEqual(f.probed, []);
    writeBuildReceipt(f.root, SHIPPING_BUILD_RECIPE);
    assert.throws(() => f.resolve());
    const [binary, ...args] = f.resolve(f.binary, SHIPPING_BUILD_RECIPE);
    assert.equal(execFileSync(binary, args, { encoding: "utf8" }).trim(), "fresh-source");
    assert.deepEqual(f.probed, [f.binary]);
    fs.appendFileSync(f.binary, "# changed staged bytes\n");
    assert.throws(() => f.resolve(f.binary, SHIPPING_BUILD_RECIPE), /binarySha256/);
    assert.deepEqual(f.probed, [f.binary]);
  } finally {
    fs.rmSync(f.root, { recursive: true, force: true });
  }
});

test("missing, corrupt or stale required LSP source bindings fail before any debug fallback", () => {
  const f = fixture();
  try {
    const binary = fs.readFileSync(f.binary);
    const receipt = fs.readFileSync(f.receipt);
    fs.rmSync(f.binary);
    assert.throws(() => f.resolve(), /ENOENT/);
    fs.writeFileSync(f.binary, binary, { mode: 0o755 });
    fs.appendFileSync(f.binary, "# corrupt bytes\n");
    assert.throws(() => f.resolve(), /binarySha256/);
    fs.writeFileSync(f.binary, binary, { mode: 0o755 });
    fs.rmSync(f.receipt);
    assert.throws(() => f.resolve(), /ENOENT/);
    fs.writeFileSync(f.receipt, receipt);
    const corruptReceipt = JSON.parse(receipt.toString()) as Record<string, unknown>;
    corruptReceipt.binarySha256 = "0".repeat(64);
    fs.writeFileSync(f.receipt, JSON.stringify(corruptReceipt));
    assert.throws(() => f.resolve(), /binarySha256/);
    fs.writeFileSync(f.receipt, receipt);
    assert.throws(() => f.resolve(""), /VIZE_LSP_BIN is required/);
    fs.writeFileSync(path.join(f.root, "source-change"), "new source revision\n");
    execFileSync("git", ["add", "source-change"], { cwd: f.root });
    execFileSync("git", ["commit", "-qm", "new source revision"], { cwd: f.root });
    assert.throws(() => f.resolve(), /sourceRevision/);
    assert.deepEqual(f.probed, []);
    fs.writeFileSync(f.binary, "#!/bin/sh\nexit 1\n", { mode: 0o755 });
    writeBuildReceipt(f.root);
    assert.throws(() => f.resolve(), /cannot launch/);
    assert.deepEqual(f.probed, [f.binary]);
  } finally {
    fs.rmSync(f.root, { recursive: true, force: true });
  }
});
