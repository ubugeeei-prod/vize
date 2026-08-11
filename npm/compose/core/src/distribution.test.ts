import assert from "node:assert/strict";
import { access, readFile } from "node:fs/promises";
import { test } from "node:test";

interface ExportConditions {
  readonly types: string;
  readonly import: string;
  readonly default: string;
}

interface PackageManifest {
  readonly sideEffects: boolean;
  readonly exports: Readonly<Record<string, ExportConditions>>;
}

const packageRoot = new URL("..", import.meta.url);

void test("publishes every utility through a typed side-effect-free subpath", async () => {
  const manifest = JSON.parse(
    await readFile(new URL("package.json", packageRoot), "utf8"),
  ) as PackageManifest;
  const expectedSubpaths = [
    ".",
    "./abort-signal",
    "./async-resource",
    "./capability",
    "./disposal-scope",
    "./event-listener",
    "./locale",
    "./media-query",
    "./scope",
    "./temporal",
    "./timeout-scheduler",
    "./use-counter",
    "./use-debounced",
    "./use-history",
    "./use-previous",
    "./use-throttled",
    "./use-toggle",
  ];

  assert.equal(manifest.sideEffects, false);
  assert.deepEqual(Object.keys(manifest.exports), expectedSubpaths);

  for (const [subpath, conditions] of Object.entries(manifest.exports)) {
    assert.equal(
      conditions.default,
      conditions.import,
      `${subpath} must expose one deterministic ESM implementation`,
    );
    assert.match(conditions.import, /^\.\/dist\/[a-z-]+\.mjs$/);
    assert.match(conditions.types, /^\.\/dist\/[a-z-]+\.d\.mts$/);
    await access(new URL(conditions.import, packageRoot));
    await access(new URL(conditions.types, packageRoot));
    await assert.doesNotReject(
      import(new URL(conditions.import, packageRoot).href),
      `${subpath} runtime entry must be importable`,
    );
  }
});
