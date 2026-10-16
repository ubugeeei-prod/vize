import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { workspaceVersionFromContent } from "../../tools/support/release/publish_crates/config.ts";
import {
  crateManifestPath,
  publishManifestWithoutDevDependencies,
} from "../../tools/support/release/publish_crates/manifest.ts";

// Ported from the MoonBit `publish_crates` white-box test.
test("publish manifest retains complete normal/build sections and strips dotted dev tables", () => {
  const original =
    '[package]\nname = "fixture"\n[dependencies]\nnormal = "1"\n[dev-dependencies]\nold = "1"\n[dev-dependencies.assert_cmd]\nversion = "2"\n[target.\'cfg(unix)\'.dev-dependencies]\nold_target = "1"\n[target."cfg(feature = \\"a.b\\")".dev-dependencies.tempfile]\nversion = "3"\n[dependencies.normal]\nversion = "1"\n[build-dependencies]\ncc = "1"\n[target.\'cfg(feature = "a.dev-dependencies.fake")\'.dependencies]\nnormal_target = "2"\n[target.\'cfg(unix)\'.build-dependencies.cc]\nversion = "1"\n';
  const expected =
    '[package]\nname = "fixture"\n[dependencies]\nnormal = "1"\n[dependencies.normal]\nversion = "1"\n[build-dependencies]\ncc = "1"\n[target.\'cfg(feature = "a.dev-dependencies.fake")\'.dependencies]\nnormal_target = "2"\n[target.\'cfg(unix)\'.build-dependencies.cc]\nversion = "1"\n';
  assert.equal(publishManifestWithoutDevDependencies(original), expected);
});

test("publish manifest keeps the original final newline and spaced table headers", () => {
  assert.equal(
    publishManifestWithoutDevDependencies("[package]\n  [ dev-dependencies . x ]  \nv = 1\n[lib]"),
    "[package]\n[lib]",
  );
  assert.equal(
    publishManifestWithoutDevDependencies("[package]\n[dev-dependencies]\nv = 1\n"),
    "[package]\n",
  );
  assert.equal(publishManifestWithoutDevDependencies("a = 1\n"), "a = 1\n");
});

test("crate manifests resolve vendor forks, Davinci levels, then legacy crates", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "release-crate-manifest-"));
  const cwd = process.cwd();
  try {
    process.chdir(root);
    fs.mkdirSync("davinci/vize_l0", { recursive: true });
    fs.writeFileSync("davinci/vize_l0/Cargo.toml", "");
    assert.equal(crateManifestPath("vize_oxc_parser"), "vendor/oxc_parser/Cargo.toml");
    assert.equal(crateManifestPath("vize_lightningcss"), "vendor/lightningcss/Cargo.toml");
    for (const name of [
      "corsa_core",
      "corsa_jsonrpc",
      "corsa_client",
      "corsa_lsp",
      "corsa_orchestrator",
      "corsa",
    ]) {
      assert.equal(crateManifestPath(`vize_${name}`), `vendor/${name}/Cargo.toml`);
    }
    assert.equal(crateManifestPath("vize_l0"), "davinci/vize_l0/Cargo.toml");
    assert.equal(crateManifestPath("vize_canon"), "crates/vize_canon/Cargo.toml");
    assert.equal(crateManifestPath("toString"), "crates/toString/Cargo.toml");
  } finally {
    process.chdir(cwd);
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("the workspace version comes only from the trimmed workspace package section", () => {
  assert.equal(
    workspaceVersionFromContent(
      'version = "0.1.0"\n[workspace.package]\n  version = "0.443.0"  \n',
    ),
    "0.443.0",
  );
  assert.equal(workspaceVersionFromContent('[package]\nversion = "1"\n'), null);
  assert.equal(workspaceVersionFromContent('[workspace.package]\nversion = "a"b"\n'), "ab");
  assert.equal(workspaceVersionFromContent('[workspace.package]\nversion = "\n'), "");
});
