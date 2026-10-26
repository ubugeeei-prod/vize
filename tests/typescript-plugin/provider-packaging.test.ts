import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  assertPackageOutput,
  checkBuildReceipt,
  digest,
  providerFiles,
  stageProvider,
  targets,
  type BuildReceipt,
} from "../../tools/support/editors/typescript-provider/artifacts.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const source = "a".repeat(40),
  token = "b".repeat(32),
  version = JSON.parse(
    fs.readFileSync(path.join(root, "npm/typescript-plugin/package.json"), "utf8"),
  ).version;
function artifact(target: string) {
  const image = Buffer.from(`controlled test image for ${target}: ${token}`);
  const receipt: BuildReceipt = {
    schema: "vize.nativeBuildReceipt.v1",
    source,
    releaseVersion: version,
    rustTarget: target,
    nodeFile: targets[target].node,
    bytes: image.length,
    sha256: digest(image),
    nativeBuildToken: token,
  };
  return { image, receipt };
}
void test("platform outputs survive the next prepublish and cannot enter another target archive", () => {
  assertPackageOutput(root, path.join(root, ".artifacts/vscode/vize-linux-x64.vsix"));
  for (const output of ["dist/vize-linux-x64.vsix", "vize-win32-x64.vsix", "native/prior.vsix"])
    assert.throws(() => assertPackageOutput(root, path.join(root, "editors/vscode", output)));
});
void test("all eight controlled native artifacts retain one exact provider inventory", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize native 日本語 paths "));
  try {
    const artifacts = path.join(temporary, "artifacts");
    fs.mkdirSync(artifacts);
    for (const target of Object.keys(targets)) {
      const { image, receipt } = artifact(target);
      fs.writeFileSync(path.join(artifacts, receipt.nodeFile), image);
      fs.writeFileSync(
        path.join(artifacts, `build-receipt.${target}.json`),
        JSON.stringify(receipt),
      );
    }
    for (const target of Object.keys(targets)) {
      const stage = path.join(temporary, target);
      const receipt = stageProvider(root, artifacts, stage, source, target);
      assert.equal(receipt.targetPlatform, targets[target].platform);
      assert.deepEqual(
        fs.readdirSync(stage).filter((name) => name.endsWith(".node")),
        [targets[target].node],
      );
      for (const file of receipt.files)
        assert.equal(digest(fs.readFileSync(path.join(stage, file.path))), file.sha256);
      for (const name of providerFiles)
        assert.deepEqual(
          fs.readFileSync(path.join(stage, name)),
          fs.readFileSync(path.join(root, "npm/native", name)),
        );
    }
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
void test("source, release, target, physical image and same-build token mismatches fail closed", () => {
  const target = "x86_64-unknown-linux-gnu";
  const { image, receipt } = artifact(target);
  checkBuildReceipt(receipt, image, source, version, target);
  for (const changed of [
    { source: "c".repeat(40) },
    { releaseVersion: "0.473.0" },
    { rustTarget: "aarch64-apple-darwin" },
    { nodeFile: "../other.node" },
    { bytes: image.length + 1 },
    { sha256: "d".repeat(64) },
    { nativeBuildToken: "e".repeat(32) },
    { nativeBuildToken: "bad" },
    { schema: "unknown" },
  ])
    assert.throws(() =>
      checkBuildReceipt({ ...receipt, ...changed } as BuildReceipt, image, source, version, target),
    );
  assert.throws(() =>
    checkBuildReceipt(receipt, Buffer.from("different physical build"), source, version, target),
  );
  assert.throws(() => checkBuildReceipt(receipt, image, source, version, "unknown"));
});
void test("authenticated VSIX layout never executes a workspace nativePath override", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vsix-provider-trust-"));
  try {
    const extension = path.join(temporary, "extension");
    const plugin = path.join(extension, "node_modules/@vizejs/typescript-vue-plugin");
    fs.mkdirSync(plugin, { recursive: true });
    for (const name of ["provider.cjs", "package.json"])
      fs.copyFileSync(path.join(root, "npm/typescript-plugin", name), path.join(plugin, name));
    const manifest = {
      name: "vize",
      publisher: "ubugeeei",
      version,
      main: "./dist/extension.cjs",
      contributes: { typescriptServerPlugins: [{ name: "@vizejs/typescript-vue-plugin" }] },
    };
    fs.writeFileSync(path.join(extension, "package.json"), JSON.stringify(manifest));
    const override = path.join(temporary, "workspace-provider.cjs"),
      marker = path.join(temporary, "executed.txt");
    fs.writeFileSync(
      override,
      `require('node:fs').writeFileSync(${JSON.stringify(marker)}, 'executed'); module.exports = {};`,
    );
    const require = createRequire(import.meta.url);
    const { loadNative } = require(path.join(plugin, "provider.cjs"));
    const universal = loadNative(temporary, override);
    assert.match(universal.error, /universal VSIX/);
    assert.equal(universal.unavailable, true);
    assert.equal(fs.existsSync(marker), false);
    manifest.version = `${version}-mismatch`;
    fs.writeFileSync(path.join(extension, "package.json"), JSON.stringify(manifest));
    // A version mismatch remains an error; it must not fall through to a
    // user-supplied module merely because the authenticated layout is broken.
    assert.match(loadNative(temporary, override).error, /versions differ/);
    assert.equal(fs.existsSync(marker), false);
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
