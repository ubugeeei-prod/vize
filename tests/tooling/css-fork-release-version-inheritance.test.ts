import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { parse as parseToml, stringify as stringifyToml } from "@iarna/toml";

import { updateWorkspaceManifestVersions } from "../../tools/support/release/release/versions.ts";
import { repoRoot } from "./support/release-command.ts";

const rootSource = fs.readFileSync(path.join(repoRoot, "Cargo.toml"), "utf8");
const root = parseToml(rootSource);
const fork = parseToml(
  fs.readFileSync(path.join(repoRoot, "vendor/lightningcss/Cargo.toml"), "utf8"),
);
type TomlTable = ReturnType<typeof parseToml>;
const workspace = root.workspace as { package: { version: string }; dependencies: TomlTable };
const current = workspace.package.version;
const target = "0.65535.0";
const dependencies = workspace.dependencies;
const forkDependencies = fork.dependencies as TomlTable;

test("the optional JSON source-map provider inherits the release-owned exact alias", () => {
  assert.deepEqual(dependencies.parcel_sourcemap, {
    package: "vize_parcel_sourcemap",
    version: `=${current}`,
    path: "vendor/parcel_sourcemap",
  });
  assert.deepEqual(forkDependencies.parcel_sourcemap, {
    workspace: true,
    features: ["json"],
    optional: true,
  });
  const features = fork.features as Record<string, string[]>;
  assert.deepEqual(features.sourcemap, ["parcel_sourcemap"]);
  assert.deepEqual(features.bundler, ["dashmap", "sourcemap", "rayon"]);
  assert.deepEqual(features.default, ["bundler", "nodejs", "sourcemap"]);
});

test("the real release rewrite resolves both forks and rejects the historical stale pin", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-fork-version-"));
  try {
    const input = path.join(directory, "release-input.toml");
    fs.writeFileSync(input, rootSource);
    const rewrite = updateWorkspaceManifestVersions(
      fs.readFileSync(input, "utf8"),
      current,
      target,
    );
    assert.equal(
      rewrite,
      rootSource
        .replace(`version = "${current}"`, `version = "${target}"`)
        .replaceAll(`version = "=${current}"`, `version = "=${target}"`),
    );
    const updated = parseToml(rewrite).workspace as typeof workspace;
    const updatedDependencies = updated.dependencies as typeof dependencies;
    for (const name of ["lightningcss", "parcel_sourcemap"]) {
      const provider = updatedDependencies[name] as { version: string };
      assert.equal(provider.version, `=${target}`);
    }

    // Minimal manifest-resolution controls: no upstream parser, archive, or code executes.
    const members = ["vendor/lightningcss", "vendor/parcel_sourcemap"];
    fs.writeFileSync(
      path.join(directory, "Cargo.toml"),
      stringifyToml({
        workspace: {
          members,
          resolver: "2",
          package: { version: updated.package.version },
          dependencies: { parcel_sourcemap: updatedDependencies.parcel_sourcemap },
        },
      }),
    );
    for (const member of members) {
      fs.mkdirSync(path.join(directory, member, "src"), { recursive: true });
      fs.writeFileSync(path.join(directory, member, "src/lib.rs"), "");
    }
    fs.writeFileSync(
      path.join(directory, members[1], "Cargo.toml"),
      stringifyToml({
        package: { name: "vize_parcel_sourcemap", version: { workspace: true }, edition: "2021" },
        features: { json: [] },
      }),
    );
    const writeConsumer = (provider: TomlTable) => {
      fs.writeFileSync(
        path.join(directory, members[0], "Cargo.toml"),
        stringifyToml({
          package: { name: "vize_lightningcss", version: { workspace: true }, edition: "2021" },
          dependencies: { parcel_sourcemap: provider },
          features: { default: ["sourcemap"], sourcemap: ["parcel_sourcemap"] },
        }),
      );
    };
    const metadata = () =>
      spawnSync("cargo", ["metadata", "--offline", "--format-version", "1"], {
        cwd: directory,
        encoding: "utf8",
        timeout: 30_000,
      });
    writeConsumer(forkDependencies.parcel_sourcemap as TomlTable);
    const healthy = metadata();
    assert.equal(healthy.error, undefined);
    assert.equal(healthy.signal, null);
    assert.equal(healthy.status, 0, healthy.stderr);
    const packet = JSON.parse(healthy.stdout);
    assert.deepEqual(
      packet.packages.map((pkg: { name: string; version: string }) => [pkg.name, pkg.version]),
      [
        ["vize_lightningcss", target],
        ["vize_parcel_sourcemap", target],
      ],
    );
    const sourceMap = packet.packages.find(
      (pkg: { name: string }) => pkg.name === "vize_parcel_sourcemap",
    );
    const node = packet.resolve.nodes.find((entry: { id: string }) => entry.id === sourceMap.id);
    assert.deepEqual(node.features, ["json"]);

    writeConsumer({
      package: "vize_parcel_sourcemap",
      version: `=${current}`,
      path: "../parcel_sourcemap",
      features: ["json"],
      optional: true,
    });
    const stale = metadata();
    assert.equal(stale.error, undefined);
    assert.equal(stale.signal, null);
    assert.equal(stale.status, 101);
    assert.equal(stale.stdout, "");
    assert.ok(stale.stderr.includes(`vize_parcel_sourcemap = "=${current}"`), stale.stderr);
    assert.ok(stale.stderr.includes(target), stale.stderr);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
