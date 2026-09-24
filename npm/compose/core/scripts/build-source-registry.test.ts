import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  buildLibRegistry,
  computeContentHash,
  sha256Hex,
} from "../../../ui/scripts/source-registry-bundle/bundle.ts";
import {
  isRelativeSpecifier,
  scanModuleSpecifiers,
} from "../../../ui/scripts/source-registry-bundle/imports.ts";
import { validateJsonSchemaSubset } from "../../../ui/scripts/source-registry-bundle/json-schema-subset.ts";
import {
  createPackageLibRegistryInput,
  serializeLibRegistryManifest,
} from "../../../ui/scripts/source-registry-bundle/write.ts";
import { COMPOSABLE_CATALOG } from "../src/catalog.ts";
import {
  COMPOSABLE_LIB_EXCLUDED_SUBPATHS,
  composableLibRegistryOptions,
} from "./build-source-registry.ts";

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const bundle = buildLibRegistry(createPackageLibRegistryInput(composableLibRegistryOptions()));
const { manifest } = bundle;
const itemsByName = new Map(manifest.items.map((item) => [item.name, item]));

function readJson(file: string): unknown {
  return JSON.parse(readFileSync(file, "utf8"));
}

void test("publishes one composable item per catalog entry except metadata", () => {
  const packageJson = readJson(path.join(packageRoot, "package.json"));
  assert.equal(manifest.kind, "composable");
  assert.equal(manifest.defaultTargetDirectory, "src/composables/vize");
  assert.equal(manifest.package.name, Reflect.get(Object(packageJson), "name"));
  assert.equal(manifest.package.version, Reflect.get(Object(packageJson), "version"));
  assert.deepEqual(
    manifest.items.map((item) => item.packageSubpath),
    COMPOSABLE_CATALOG.entries
      .map((entry) => entry.subpath)
      .filter((subpath) => !COMPOSABLE_LIB_EXCLUDED_SUBPATHS.has(subpath))
      .sort(),
  );
  for (const utility of COMPOSABLE_CATALOG.utilities) {
    const item = itemsByName.get(utility.entry.slice(2));
    assert.ok(item?.aliases.includes(utility.name), `${utility.name} resolves to ${utility.entry}`);
  }
});

void test("is deterministic and conforms to the published JSON Schema", () => {
  const again = buildLibRegistry(createPackageLibRegistryInput(composableLibRegistryOptions()));
  assert.equal(serializeLibRegistryManifest(again), serializeLibRegistryManifest(bundle));
  const schema = readJson(
    path.join(packageRoot, "../../cli/schemas/vize-lib-registry.schema.json"),
  );
  assert.deepEqual(validateJsonSchemaSubset(schema, manifest), []);
});

void test("hashes files and resolves every relative import inside the closure", () => {
  for (const item of manifest.items) {
    const reachable = new Set(item.files.map((file) => file.path));
    for (const dependency of item.registryDependencies) {
      for (const file of itemsByName.get(dependency)?.files ?? []) reachable.add(file.path);
    }
    for (const file of item.files) {
      assert.ok(!file.path.endsWith(".test.ts"));
      const bytes = readFileSync(path.join(packageRoot, "src", file.path));
      assert.equal(file.sha256, sha256Hex(bytes));
      for (const specifier of scanModuleSpecifiers(file.path, bytes.toString("utf8"))) {
        if (!isRelativeSpecifier(specifier)) continue;
        const target = path.posix.normalize(
          path.posix.join(path.posix.dirname(file.path), specifier),
        );
        assert.ok(reachable.has(target), `${item.name}: ${file.path} -> ${specifier}`);
      }
    }
    assert.equal(item.contentHash, computeContentHash(item.files));
  }
  assert.deepEqual(itemsByName.get("retry-async")?.registryDependencies, [
    "abort-signal",
    "retry-delay",
    "timeout-scheduler",
  ]);
});

void test("declares vue as a peer and runtime packages from dependencies", () => {
  assert.deepEqual(itemsByName.get("use-toggle")?.dependencies, [
    { name: "vue", range: "^3.5.0", kind: "peer" },
  ]);
  assert.deepEqual(
    itemsByName.get("temporal")?.dependencies.map((dependency) => dependency.kind),
    ["runtime", "peer"],
  );
  const files = Reflect.get(Object(readJson(path.join(packageRoot, "package.json"))), "files");
  assert.ok(Array.isArray(files) && files.includes("registry"));
});
