import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { sourceLspRuntimeEnvironment } from "./support/lsp/runtime-environment.ts";
import { stableTypeScriptRuntimePath } from "./support/typecheck-dependency.ts";

test("source LSP fixtures carry only their checkout platform provider into a foreign workspace", () => {
  const fixture = providerFixture();
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-runtime-workspace-"));
  try {
    const input = { PATH: "/foreign/bin", RETAINED: "unchanged" };
    const environment = sourceLspRuntimeEnvironment(fixture.root, input);
    assert.deepEqual(environment, { ...input, CORSA_PATH: fs.realpathSync(fixture.binary) });
    assert.deepEqual(input, { PATH: "/foreign/bin", RETAINED: "unchanged" });
    const child = spawnSync(
      process.execPath,
      ["-e", "process.stdout.write(process.env.CORSA_PATH)"],
      {
        cwd: workspace,
        env: environment,
        encoding: "utf8",
      },
    );
    assert.equal(child.status, 0, child.stderr);
    assert.equal(child.signal, null);
    assert.equal(child.stdout, fs.realpathSync(fixture.binary));
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
    fixture.dispose();
  }
});

for (const variable of [
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
]) {
  test(`source LSP runtime keeps explicit ${variable} even when it is unavailable`, () => {
    const fixture = providerFixture();
    try {
      const environment = { [variable]: "/deliberately/missing/runtime", RETAINED: "unchanged" };
      assert.deepEqual(sourceLspRuntimeEnvironment(fixture.root, environment), environment);
    } finally {
      fixture.dispose();
    }
  });
}

test("an absent checkout provider never selects PATH, a parent package, or another platform", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-runtime-absent-"));
  try {
    const root = path.join(directory, "source");
    fs.mkdirSync(root);
    fs.mkdirSync(path.join(directory, "node_modules/@typescript/typescript-foreign/lib"), {
      recursive: true,
    });
    const environment = { PATH: directory, RETAINED: "unchanged" };
    assert.deepEqual(sourceLspRuntimeEnvironment(root, environment), environment);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("an installed provider with the wrong identity or missing executable fails before launch", () => {
  const fixture = providerFixture();
  try {
    fs.writeFileSync(
      fixture.metadata,
      JSON.stringify({ name: "foreign-provider", version: "7.0.2" }),
    );
    assert.throws(() => sourceLspRuntimeEnvironment(fixture.root, {}), /selected platform package/);
    fs.writeFileSync(fixture.metadata, JSON.stringify({ name: fixture.name, version: "7.0.2" }));
    fs.unlinkSync(fixture.binary);
    assert.throws(() => sourceLspRuntimeEnvironment(fixture.root, {}), /ENOENT/);
  } finally {
    fixture.dispose();
  }
});

function providerFixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-runtime-provider-"));
  const name = `@typescript/typescript-${process.platform}-${process.arch}`;
  const directory = path.join(root, "node_modules", name);
  const binary = stableTypeScriptRuntimePath(root);
  fs.mkdirSync(path.dirname(binary), { recursive: true });
  const metadata = path.join(directory, "package.json");
  fs.writeFileSync(metadata, JSON.stringify({ name, version: "7.0.2" }));
  // Synthetic file ownership law only; no test executes this as a native checker.
  fs.writeFileSync(binary, "synthetic provider bytes\n", { mode: 0o755 });
  return {
    root,
    name,
    directory,
    binary,
    metadata,
    dispose: () => fs.rmSync(root, { recursive: true, force: true }),
  };
}
