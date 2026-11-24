import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse as parseToml } from "@iarna/toml";

import { workspaceVersionFromContent } from "../../tools/support/release/publish_crates/config.ts";
import { publishedCrates } from "../../tools/support/release/publish_crates/main.ts";
import {
  crateManifestPath,
  publishManifestWithoutDevDependencies,
} from "../../tools/support/release/publish_crates/manifest.ts";

interface DependencyTables {
  dependencies?: Record<string, unknown>;
  "build-dependencies"?: Record<string, unknown>;
}

interface CrateManifest extends DependencyTables {
  features?: Record<string, string[]>;
  target?: Record<string, DependencyTables>;
  "dev-dependencies"?: Record<string, { features?: string[] }>;
}

test("every staged release manifest retains the dependencies used by its public features", () => {
  for (const crate of publishedCrates) {
    const original = fs.readFileSync(crateManifestPath(crate), "utf8");
    const staged = parseToml(publishManifestWithoutDevDependencies(original)) as CrateManifest;
    const dependencies = new Set<string>();
    for (const scope of [staged, ...Object.values(staged.target ?? {})]) {
      for (const kind of ["dependencies", "build-dependencies"] as const) {
        for (const name of Object.keys(scope[kind] ?? {})) dependencies.add(name);
      }
    }
    for (const [feature, entries] of Object.entries(staged.features ?? {})) {
      for (const entry of entries) {
        const dependency = entry.startsWith("dep:")
          ? entry.slice(4)
          : entry.includes("/")
            ? entry.split("/")[0].replace(/\?$/u, "")
            : null;
        if (dependency !== null) {
          assert.ok(dependencies.has(dependency), `${crate}: ${feature} refers to ${entry}`);
        }
      }
    }
  }
});

test("the graph-query corpus enables IR views through its development dependency", () => {
  const manifest = parseToml(
    fs.readFileSync(crateManifestPath("vize_l2_to_l3"), "utf8"),
  ) as CrateManifest;
  assert.deepEqual(manifest.features?.["graph-queries"], [
    "vize_l3/graph-queries",
    "vize_l2/ir-views",
  ]);
  assert.deepEqual(manifest["dev-dependencies"]?.vize_l1_to_l2.features, ["ir-views"]);
});

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
    assert.equal(crateManifestPath("vize_oxc_parser"), "upstream/forks/oxc_parser/Cargo.toml");
    assert.equal(crateManifestPath("vize_lightningcss"), "upstream/forks/lightningcss/Cargo.toml");
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
