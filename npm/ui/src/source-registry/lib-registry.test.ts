import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync, mkdirSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { test } from "vite-plus/test";

import { uiLibRegistryOptions } from "../../scripts/build-source-registry.ts";
import {
  buildLibRegistry,
  computeContentHash,
  isTestSourcePath,
  sha256Hex,
} from "../../scripts/source-registry-bundle/bundle.ts";
import {
  isRelativeSpecifier,
  scanModuleSpecifiers,
} from "../../scripts/source-registry-bundle/imports.ts";
import { validateJsonSchemaSubset } from "../../scripts/source-registry-bundle/json-schema-subset.ts";
import type { LibRegistryItem } from "../../scripts/source-registry-bundle/types.ts";
import {
  createPackageLibRegistryInput,
  serializeLibRegistryManifest,
  writeLibRegistry,
} from "../../scripts/source-registry-bundle/write.ts";
import { uiFamilyCatalog } from "../catalog/family-catalog.ts";

const packageRoot = path.resolve(import.meta.dirname, "../..");
const sourceRoot = path.join(packageRoot, "src");
const bundle = buildLibRegistry(createPackageLibRegistryInput(uiLibRegistryOptions()));
const { manifest } = bundle;
const itemsByName = new Map(manifest.items.map((item) => [item.name, item]));

function readJson(file: string): unknown {
  return JSON.parse(readFileSync(file, "utf8"));
}

function closureFiles(item: LibRegistryItem): ReadonlySet<string> {
  const files = new Set(item.files.map((file) => file.path));
  for (const name of item.registryDependencies) {
    for (const file of itemsByName.get(name)?.files ?? []) files.add(file.path);
  }
  return files;
}

test("publishes one registry item per catalog family with package identity", () => {
  const packageJson = readJson(path.join(packageRoot, "package.json"));
  assert.equal(manifest.schemaVersion, 1);
  assert.equal(manifest.registryKind, "vize-lib");
  assert.deepEqual(manifest.package, {
    name: Reflect.get(Object(packageJson), "name"),
    version: Reflect.get(Object(packageJson), "version"),
  });
  assert.equal(manifest.kind, "ui");
  assert.equal(manifest.defaultTargetDirectory, "src/components/vize");
  assert.deepEqual(
    manifest.items.map((item) => item.name),
    uiFamilyCatalog.map((entry) => entry.canonicalName),
  );
  for (const entry of uiFamilyCatalog) {
    const item = itemsByName.get(entry.canonicalName);
    assert.ok(item);
    assert.equal(item.kind, "ui");
    assert.equal(item.title, entry.title);
    assert.equal(item.packageSubpath, entry.packageSubpath);
    assert.equal(`src/${item.entry}`, entry.entryFile);
    for (const alias of entry.aliases) {
      if (alias !== entry.canonicalName) assert.ok(item.aliases.includes(alias), alias);
    }
  }
});

test("is byte-for-byte deterministic", () => {
  const again = buildLibRegistry(createPackageLibRegistryInput(uiLibRegistryOptions()));
  assert.equal(serializeLibRegistryManifest(again), serializeLibRegistryManifest(bundle));
});

test("conforms to the published JSON Schema", () => {
  const schema = readJson(path.join(packageRoot, "../cli/schemas/vize-lib-registry.schema.json"));
  assert.deepEqual(validateJsonSchemaSubset(schema, manifest), []);
});

test("hashes every published file and item content", () => {
  const seen = new Set<string>();
  for (const item of manifest.items) {
    assert.ok(item.files.some((file) => file.path === item.entry && file.role === "entry"));
    for (const file of item.files) {
      assert.ok(!seen.has(file.path), `${file.path} is owned by exactly one item`);
      seen.add(file.path);
      assert.ok(!isTestSourcePath(file.path), `${file.path} must not ship`);
      const bytes = readFileSync(path.join(sourceRoot, file.path));
      assert.equal(file.sha256, sha256Hex(bytes));
      assert.equal(file.size, bytes.byteLength);
      assert.deepEqual(bundle.files.get(file.path), bytes);
    }
    assert.equal(item.contentHash, computeContentHash(item.files));
  }
  assert.equal(bundle.files.size, seen.size);
});

test("resolves every relative import inside the item's registry closure", () => {
  for (const item of manifest.items) {
    const reachable = closureFiles(item);
    for (const file of item.files) {
      const source = readFileSync(path.join(sourceRoot, file.path), "utf8");
      const specifiers = file.path.endsWith(".css") ? [] : scanModuleSpecifiers(file.path, source);
      for (const specifier of specifiers.filter(isRelativeSpecifier)) {
        const target = path.posix.normalize(
          path.posix.join(path.posix.dirname(file.path), specifier),
        );
        assert.ok(reachable.has(target), `${item.name}: ${file.path} -> ${specifier}`);
      }
    }
  }
});

test("registry dependencies are transitively closed and acyclic in naming", () => {
  for (const item of manifest.items) {
    assert.ok(!item.registryDependencies.includes(item.name));
    for (const dependency of item.registryDependencies) {
      const target = itemsByName.get(dependency);
      assert.ok(target, `${item.name} depends on unknown ${dependency}`);
      for (const nested of target.registryDependencies) {
        assert.ok(
          nested === item.name || item.registryDependencies.includes(nested),
          `${item.name} misses ${nested} via ${dependency}`,
        );
      }
    }
  }
});

test("pulls shared foundations as their own items", () => {
  assert.deepEqual(itemsByName.get("rating")?.registryDependencies, ["controllable-state", "id"]);
  assert.ok(itemsByName.get("icon-button")?.registryDependencies.includes("icon"));
  assert.ok(
    itemsByName
      .get("press")
      ?.files.some((file) => file.path === "families/interaction/press/press-notify.ts"),
    "uncatalogued support modules are claimed by the importing item",
  );
  assert.equal(
    itemsByName.get("motion")?.files.find((file) => file.path.endsWith(".css"))?.role,
    "style",
  );
});

test("declares vue as the only npm dependency, as a peer", () => {
  for (const item of manifest.items) {
    for (const dependency of item.dependencies) {
      assert.deepEqual(dependency, { name: "vue", range: "^3.5.0", kind: "peer" });
    }
  }
  assert.ok(manifest.items.some((item) => item.dependencies.length > 0));
});

test("ships the registry directory in the npm tarball", () => {
  const files = Reflect.get(Object(readJson(path.join(packageRoot, "package.json"))), "files");
  assert.ok(Array.isArray(files) && files.includes("registry"));
});

test("writes registry.json and raw files, replacing stale output", () => {
  const root = mkdtempSync(path.join(tmpdir(), "vize-ui-registry-"));
  try {
    mkdirSync(path.join(root, "registry/files"), { recursive: true });
    writeFileSync(path.join(root, "registry/files/stale.ts"), "stale");
    const manifestPath = writeLibRegistry(root, bundle);
    assert.equal(readFileSync(manifestPath, "utf8"), serializeLibRegistryManifest(bundle));
    assert.ok(!readdirSync(path.join(root, "registry/files")).includes("stale.ts"));
    const rating = itemsByName.get("rating");
    for (const file of rating?.files ?? []) {
      const written = readFileSync(path.join(root, "registry/files", file.path));
      assert.equal(sha256Hex(written), file.sha256);
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

function fixtureInput(root: string, files: Readonly<Record<string, string>>) {
  for (const [file, content] of Object.entries(files)) {
    mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    writeFileSync(path.join(root, file), content);
  }
  return {
    packageName: "@fixture/lib",
    packageVersion: "1.0.0",
    kind: "ui" as const,
    defaultTargetDirectory: "src/components/fixture",
    sourceRoot: root,
    items: [
      {
        name: "alpha",
        title: "Alpha",
        description: "Alpha fixture.",
        aliases: [],
        packageSubpath: "./alpha",
        entry: "alpha/alpha.ts",
        files: ["alpha/alpha.ts"],
      },
    ],
    peerDependencies: { vue: "^3.5.0" },
    dependencies: {},
  };
}

test("rejects unresolvable relative imports and undeclared packages", () => {
  const root = mkdtempSync(path.join(tmpdir(), "vize-ui-registry-fixture-"));
  try {
    assert.throws(
      () => buildLibRegistry(fixtureInput(root, { "alpha/alpha.ts": 'import "../../escape.ts";' })),
      /cannot resolve relative import/,
    );
    assert.throws(
      () => buildLibRegistry(fixtureInput(root, { "alpha/alpha.ts": 'import "left-pad";' })),
      /does not declare/,
    );
    assert.throws(
      () =>
        buildLibRegistry({
          ...fixtureInput(root, { "alpha/alpha.ts": "export {};" }),
          items: [{ ...fixtureInput(root, {}).items[0]!, files: ["alpha/missing.ts"] }],
        }),
      /does not exist/,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
