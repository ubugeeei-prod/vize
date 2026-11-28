import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import {
  CHANNEL_DEPENDENCY_EXCEPTIONS,
  channelDependencyProblems,
  type DependencyManifest,
} from "../../tools/support/release/channel-dependencies.ts";
import { jsrPackages } from "../../tools/support/release/jsr/packages.ts";
import {
  channelArtifacts,
  NATIVE_PLATFORM_PACKAGES,
  npmProduct,
  PRODUCT_CHANNELS,
  productChannel,
  RELEASE_CHANNELS,
  renderStabilityPage,
  STABILITY_LOCALES,
  stabilityPagePath,
} from "../../tools/support/release/product-channels.ts";

const root = path.resolve(import.meta.dirname, "../..");

interface Manifest extends DependencyManifest {
  file: string;
  private?: boolean;
  publishConfig?: { access?: string };
  engines?: { vscode?: string };
  metadata?: { vize?: { channel?: string } };
}

function trackedManifests(directory: string): Manifest[] {
  return execFileSync("git", ["ls-files", "--", directory], { cwd: root, encoding: "utf8" })
    .split("\n")
    .filter((file) => file === `${directory}/package.json` || file.endsWith("/package.json"))
    .map((file) => ({
      ...(JSON.parse(readFileSync(path.join(root, file), "utf8")) as Omit<Manifest, "file">),
      file,
    }));
}

const publicNpm = trackedManifests("npm").filter(
  (manifest) => manifest.private !== true && manifest.publishConfig?.access === "public",
);

function registered(registry: string): string[] {
  return channelArtifacts()
    .filter((artifact) => artifact.registry === registry)
    .map(({ name }) => name)
    .toSorted();
}

void test("every registered artifact has exactly one product, channel and owner", () => {
  const artifacts = channelArtifacts();
  const keys = artifacts.map(({ registry, name }) => `${registry}:${name}`);
  assert.deepEqual(
    keys.filter((key, index) => keys.indexOf(key) !== index),
    [],
    "an artifact is listed under two products",
  );
  for (const [product, entry] of Object.entries(PRODUCT_CHANNELS)) {
    assert.ok(RELEASE_CHANNELS.includes(entry.channel), product);
    assert.match(entry.owner, /^[a-z0-9-]+\/\S+$/u, product);
    for (const locale of STABILITY_LOCALES) assert.ok(entry.label[locale].trim(), product);
  }
});

void test("every publishable npm package and native binary is registered", () => {
  assert.deepEqual(
    registered("npm"),
    [...publicNpm.map(({ name }) => name), ...NATIVE_PLATFORM_PACKAGES].toSorted(),
  );
  const native = publicNpm.find(({ name }) => name === "@vizejs/native")!;
  assert.deepEqual(
    Object.keys(native.optionalDependencies ?? {}).toSorted(),
    [...NATIVE_PLATFORM_PACKAGES].toSorted(),
  );
});

// Manifests whose exact bytes an enforced byte-custody record pins keep those bytes; the
// registry alone carries their channel until the record moves on.
const custodyPinned = new Set(
  readFileSync(path.join(root, "tools/config/byte-custody.tsv"), "utf8")
    .split("\n")
    .map((line) => line.split("\t"))
    .filter(([, , kind]) => kind === "enforced")
    .map(([file]) => file),
);

void test("npm manifests record the channel of their registered product", () => {
  for (const manifest of publicNpm) {
    const product = npmProduct(manifest.name);
    assert.ok(product, manifest.name);
    const recorded = manifest.metadata?.vize?.channel;
    if (custodyPinned.has(manifest.file) && recorded === undefined) continue;
    assert.equal(recorded, productChannel(product), manifest.file);
  }
});

void test("every publishable crate is registered", () => {
  // docs-stability.test.ts proves this table equals the publishable Cargo packages.
  const stability = readFileSync(path.join(root, "docs/content/stability.md"), "utf8");
  const table = stability
    .split("<!-- rust-crate-support:start -->")[1]
    .split("<!-- rust-crate-support:end -->")[0];
  const crates = [...table.matchAll(/^\| `(vize_[a-z0-9_]+)`/gmu)].map((match) => match[1]);
  assert.ok(crates.length > 0);
  assert.deepEqual(registered("crates.io"), crates.toSorted());
});

void test("every editor extension, JSR mirror and released npm directory is registered", () => {
  const extensions = trackedManifests("editors")
    .filter((manifest) => manifest.engines?.vscode !== undefined)
    .map(({ name }) => name)
    .toSorted();
  assert.deepEqual(registered("vscode-marketplace"), extensions);
  for (const name of registered("open-vsx")) assert.ok(extensions.includes(name), name);

  for (const { npmName } of jsrPackages) assert.ok(npmProduct(npmName), npmName);

  const release = parse(readFileSync(path.join(root, ".github/workflows/release.yml"), "utf8")) as {
    env: { RELEASE_JS_PACKAGES: string };
  };
  const directories = release.env.RELEASE_JS_PACKAGES.split(" ");
  assert.ok(directories.length > 0);
  for (const directory of [...directories, "npm/wasm"]) {
    const manifest = publicNpm.find(({ file }) => file === `${directory}/package.json`);
    assert.ok(manifest, directory);
    assert.ok(npmProduct(manifest.name), directory);
  }
});

void test("the stability pages show the current channel table in every locale", () => {
  for (const locale of STABILITY_LOCALES)
    assert.equal(
      readFileSync(stabilityPagePath(root, locale), "utf8"),
      renderStabilityPage(root, locale),
      `${locale}: run node tools/commands/release/product-channels.ts`,
    );
});

void test("dependencies between Vize packages follow the channel range policy", () => {
  assert.deepEqual(channelDependencyProblems(publicNpm), []);
  for (const exception of CHANNEL_DEPENDENCY_EXCEPTIONS) assert.ok(exception.reason.trim());
});

void test("the channel range policy rejects new cross-channel coupling", () => {
  const base = publicNpm.map(({ name, dependencies, optionalDependencies, peerDependencies }) => ({
    name,
    dependencies,
    optionalDependencies,
    peerDependencies,
  }));
  const withEdge = (name: string, edge: Partial<DependencyManifest>) =>
    base.map((manifest) =>
      manifest.name === name
        ? {
            ...manifest,
            dependencies: { ...manifest.dependencies, ...edge.dependencies },
            peerDependencies: { ...manifest.peerDependencies, ...edge.peerDependencies },
          }
        : manifest,
    );

  assert.deepEqual(
    channelDependencyProblems(
      withEdge("@vizejs/vite-plugin", { dependencies: { "@vizejs/ui": "workspace:*" } }),
    ),
    [
      '@vizejs/vite-plugin (v1-alpha) dependencies @vizejs/ui (early) "workspace:*": a hard dependency stays in its channel or points at v1 alpha',
    ],
  );
  assert.deepEqual(
    channelDependencyProblems(
      withEdge("@vizejs/vite-plugin", { dependencies: { "@vizejs/native": "^0.800.0" } }),
    ),
    [
      '@vizejs/vite-plugin (v1-alpha) dependencies @vizejs/native (v1-alpha) "^0.800.0": use workspace:*',
    ],
  );
  assert.deepEqual(
    channelDependencyProblems(
      withEdge("@vizejs/fmt", { peerDependencies: { "@vizejs/lsp": "workspace:*" } }),
    ),
    [
      '@vizejs/fmt (v0) peerDependencies @vizejs/lsp (v1-alpha) "workspace:*": a peer across channels takes an open >= lower bound',
    ],
  );
  assert.deepEqual(
    channelDependencyProblems(
      withEdge("@vizejs/fmt", { peerDependencies: { "@vizejs/lsp": ">=0.800.0" } }),
    ),
    [],
  );

  const withoutMusea = base.map((manifest) =>
    manifest.name === "@vizejs/nuxt"
      ? {
          ...manifest,
          dependencies: Object.fromEntries(
            Object.entries(manifest.dependencies ?? {}).filter(
              ([name]) => name !== "@vizejs/vite-plugin-musea",
            ),
          ),
        }
      : manifest,
  );
  assert.deepEqual(channelDependencyProblems(withoutMusea), [
    "stale exception: @vizejs/nuxt -> @vizejs/vite-plugin-musea",
  ]);
  assert.equal(npmProduct("@vizejs/not-a-package"), undefined);
});
