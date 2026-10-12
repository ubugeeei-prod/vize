import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { parse as parseToml, stringify as stringifyToml } from "@iarna/toml";

import {
  authenticateProducerMetadata,
  authenticateProducerSource,
} from "../../tools/support/dependencies/css-engine-producer-manifests.ts";
import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";

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

test("only whole reviewed manifests extend all nine original producer authorities", () => {
  const metadata = authenticateProducerMetadata(repoRoot);
  const checksum = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
  const fixture = path.join(repoRoot, "tests/_fixtures/differential/css-engine-vendor-exit");
  const sealBytes = fs.readFileSync(path.join(fixture, "bundler-before-whole.json"));
  assert.equal(
    checksum(sealBytes),
    "d139e32de20854c5068a38bb3803884a12db81484fe032ba4c45c575774c1da1",
  );
  const sealedSources: { path: string; sha256: string }[] = JSON.parse(sealBytes.toString())
    .twoProfiles[0].allTenSourceCopies;
  const legacy = fs.readFileSync(
    path.join(fixture, "release-inheritance/bundler-after-original.ts.txt"),
  );
  assert.equal(
    checksum(legacy),
    "a126866ead8f2dd961926a217da4d18891bb3a7f92af7fc29b5179f2a70641d6",
  );
  const legacyDirectory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-original-authority-"));
  try {
    const executable = path.join(legacyDirectory, "original.ts");
    fs.writeFileSync(executable, legacy);
    const refusal = spawnSync(process.execPath, [executable, "--authenticate"], {
      cwd: repoRoot,
      encoding: "utf8",
    });
    assert.equal(refusal.error, undefined);
    assert.equal(refusal.signal, null);
    assert.equal(refusal.status, 1);
    assert.equal(refusal.stdout, "");
    assert.ok(refusal.stderr.includes("AssertionError"), refusal.stderr);
    assert.ok(refusal.stderr.includes("Cargo.toml"), refusal.stderr);
    assert.ok(
      refusal.stderr.includes(checksum(fs.readFileSync(path.join(repoRoot, "Cargo.toml")))),
      refusal.stderr,
    );
    assert.ok(
      refusal.stderr.includes("c8fe5d4b91f49524fb4f8cd06c7453affd936b2373c29b622e94a01dc7a0793f"),
      refusal.stderr,
    );
  } finally {
    fs.rmSync(legacyDirectory, { recursive: true, force: true });
  }
  for (const [sourcePath, file] of [
    ["Cargo.toml", "root-original.toml"],
    ["vendor/lightningcss/Cargo.toml", "lightningcss-original.toml"],
  ]) {
    const original = fs.readFileSync(path.join(fixture, "release-inheritance", file));
    const current = fs.readFileSync(path.join(repoRoot, sourcePath));
    const sealed = sealedSources.find((row) => row.path === sourcePath);
    assert(sealed);
    assert.equal(checksum(original), sealed.sha256);
    assert.deepEqual(authenticateProducerSource(sourcePath, original, sealed.sha256), {
      path: sourcePath,
      actualSha256: sealed.sha256,
      originalSha256: sealed.sha256,
      reviewedContinuation: false,
    });
    assert.deepEqual(authenticateProducerSource(sourcePath, current, sealed.sha256, metadata), {
      path: sourcePath,
      actualSha256: checksum(current),
      originalSha256: sealed.sha256,
      reviewedContinuation: true,
    });
    for (const raw of [original, current]) {
      assert.throws(() =>
        authenticateProducerSource(
          sourcePath,
          Buffer.concat([raw, Buffer.from("# unreviewed byte change\n")]),
          sealed.sha256,
        ),
      );
    }
    assert.throws(() => authenticateProducerSource(sourcePath, current, checksum(current)));
    assert.throws(() => authenticateProducerSource("unknown.toml", current, sealed.sha256));
    assert.throws(() => authenticateProducerSource("__proto__", current, sealed.sha256));
    assert.throws(() => authenticateProducerSource(sourcePath, current, "0".repeat(64)));
    const mutation =
      sourcePath === "Cargo.toml"
        ? current
            .toString()
            .replace('path = "vendor/parcel_sourcemap"', 'path = "vendor/unreviewed"')
        : current
            .toString()
            .replace(
              '[dependencies.parcel_sourcemap]\nworkspace = true\nfeatures = ["json"]',
              '[dependencies.parcel_sourcemap]\nworkspace = true\nfeatures = ["json", "unreviewed"]',
            );
    assert.notEqual(mutation, current.toString());
    assert.throws(() =>
      authenticateProducerSource(sourcePath, Buffer.from(mutation), sealed.sha256),
    );
  }

  // Execute the real source-after authenticator; every other original input still must match.
  const result = spawnSync(
    process.execPath,
    ["tools/support/dependencies/css-engine-bundler-after.ts", "--authenticate"],
    { cwd: repoRoot, encoding: "utf8" },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stderr);
  const receipt = JSON.parse(result.stdout);
  const expectedSources = sealedSources.filter(
    (row) => row.path !== "vendor/lightningcss/src/bundler.rs",
  );
  assert.deepEqual(
    receipt.producerSourceAuthorities.map((row: { path: string }) => row.path),
    expectedSources.map((row) => row.path),
  );
  assert.equal(receipt.producerSourceAuthorities.length, 9);
  assert.equal(
    receipt.producerSourceAuthorities.filter(
      (row: { reviewedContinuation: boolean }) => row.reviewedContinuation,
    ).length,
    2 +
      Number(
        receipt.producerMetadata.files.find((row: { path: string }) => row.path === "Cargo.lock")
          .changed,
      ),
  );
  for (const [index, row] of receipt.producerSourceAuthorities.entries()) {
    assert.equal(row.originalSha256, expectedSources[index].sha256);
    assert.equal(row.actualSha256, checksum(fs.readFileSync(path.join(repoRoot, row.path))));
  }
  assert.equal(receipt.publicationQualification, false);
});

test("the real release rewrite resolves both forks and rejects the historical stale pin", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-fork-version-"));
  try {
    const input = path.join(directory, "release-input.toml");
    fs.writeFileSync(input, rootSource);
    const rewrite = runMoonScript("release", [
      "--print-workspace-manifest-update",
      input,
      current,
      target,
    ]);
    assert.equal(rewrite.error, undefined);
    assert.equal(rewrite.signal, null);
    assert.equal(rewrite.status, 0, rewrite.stderr);
    assert.equal(
      rewrite.stdout,
      rootSource
        .replace(`version = "${current}"`, `version = "${target}"`)
        .replaceAll(`version = "=${current}"`, `version = "=${target}"`),
    );
    const updated = parseToml(rewrite.stdout).workspace as typeof workspace;
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
