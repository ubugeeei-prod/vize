import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { verifyFrozenNativeTestRuntime } from "../../tools/support/compat/github/frozen-native-test-runtime.ts";

function fixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-historical-runtime-"));
  const name = "@typescript/typescript-linux-x64";
  const provider = path.join(root, "node_modules", name);
  fs.mkdirSync(path.join(provider, "lib"), { recursive: true });
  const manifest = (value: unknown) =>
    fs.writeFileSync(path.join(provider, "package.json"), JSON.stringify(value));
  const workspace = (value: unknown) =>
    fs.writeFileSync(path.join(root, "package.json"), JSON.stringify(value));
  workspace({ private: true, optionalDependencies: { [name]: "catalog:corsa-runtime" } });
  manifest({ name, version: "7.0.2" });
  fs.writeFileSync(path.join(provider, "lib", "tsc"), "not an authorized executable");
  return {
    root,
    name,
    provider,
    manifest,
    workspace,
    remove: () => fs.rmSync(root, { force: true, recursive: true }),
  };
}

test("test-provider pin refuses public workspaces, undeclared packages and wrong manifest identities", () => {
  const f = fixture();
  try {
    f.workspace({ private: false, optionalDependencies: { [f.name]: "catalog:corsa-runtime" } });
    assert.throws(() => verifyFrozenNativeTestRuntime(f.root), {
      code: "ERR_ASSERTION",
      actual: false,
      expected: true,
      operator: "strictEqual",
    });
    f.workspace({ private: true });
    assert.throws(() => verifyFrozenNativeTestRuntime(f.root), {
      code: "ERR_ASSERTION",
      actual: undefined,
      expected: "catalog:corsa-runtime",
      operator: "strictEqual",
    });
    f.workspace({ private: true, optionalDependencies: { [f.name]: "catalog:corsa-runtime" } });
    f.manifest({ name: "unrelated", version: "7.0.2" });
    assert.throws(() => verifyFrozenNativeTestRuntime(f.root), {
      code: "ERR_ASSERTION",
      actual: "unrelated",
      expected: f.name,
      operator: "strictEqual",
    });
    f.manifest({ name: f.name, version: "7.1.0-beta" });
    assert.throws(() => verifyFrozenNativeTestRuntime(f.root), {
      code: "ERR_ASSERTION",
      actual: "7.1.0-beta",
      expected: "7.0.2",
      operator: "strictEqual",
    });
  } finally {
    f.remove();
  }
});

test("test-provider pin refuses marker files and a foreign executable symlink", () => {
  const f = fixture();
  try {
    assert.throws(() => verifyFrozenNativeTestRuntime(f.root), {
      code: "ERR_ASSERTION",
      actual: createHash("sha256").update("not an authorized executable").digest("hex"),
      expected: "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
      operator: "strictEqual",
    });
    fs.rmSync(path.join(f.provider, "lib", "tsc"));
    const outside = path.join(f.root, "unowned-native");
    fs.writeFileSync(outside, "unowned");
    fs.symlinkSync(outside, path.join(f.provider, "lib", "tsc"));
    assert.throws(() => verifyFrozenNativeTestRuntime(f.root), {
      code: "ERR_ASSERTION",
      actual: path.dirname(fs.realpathSync(f.root)),
      expected: fs.realpathSync(f.provider),
      operator: "strictEqual",
    });
  } finally {
    f.remove();
  }
});
