import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import path from "node:path";

import { test } from "vite-plus/test";

import config from "../vite.config.ts";
import { UI_FAMILY_CATALOG_SCHEMA_VERSION, uiFamilyCatalog } from "./family-catalog.ts";
import { assertFamilyPaths, uiFamilyRoots } from "./family-layout-test-utils.ts";

type PackageExport = string | { readonly import: string; readonly types: string };

const stableEntries = uiFamilyCatalog.filter((entry) => entry.maturity === "stable");
const requiredQualityGates = ["behavior-contract", "mounted-dom", "bundle-size"] as const;
const packageManifest = JSON.parse(await readFile(path.resolve("package.json"), "utf8")) as {
  readonly exports: Readonly<Record<string, PackageExport>>;
};
const packEntries = (
  config as { readonly pack?: { readonly entry?: Readonly<Record<string, string>> } }
).pack?.entry;
const rendererGate = [
  await readFile(path.resolve("scripts/check-renderers.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures-avatar.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures-commands.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures-data.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures-dialog.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures-feedback.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures-icon.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures-layout.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures-navigation.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures-overlays.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures-primitives.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures-selection.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures-spinner.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures-status-light.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures-text.ts"), "utf8"),
  await readFile(path.resolve("scripts/renderer-fixtures-toggle-group.ts"), "utf8"),
].join("\n");

test("publishes a versioned stable source-owned family catalog", () => {
  assert.equal(UI_FAMILY_CATALOG_SCHEMA_VERSION, 1);
  assert.equal(stableEntries.length, uiFamilyCatalog.length);

  const names = stableEntries.map((entry) => entry.canonicalName);
  assert.deepEqual(names, [...names].sort(), "catalog entries must stay in canonical order");
  assert.equal(new Set(names).size, names.length, "canonical names must be unique");

  for (const entry of stableEntries) {
    assert.ok(entry.owner.length > 0, `${entry.canonicalName} must declare an owner`);
    assert.ok(entry.aliases.length > 0, `${entry.canonicalName} must declare aliases`);
    assert.ok(
      entry.upstreamCoverage.length > 0,
      `${entry.canonicalName} must declare upstream coverage`,
    );
    for (const gate of requiredQualityGates) {
      assert.ok(entry.qualityGates.includes(gate), `${entry.canonicalName} must require ${gate}`);
    }
    assert.ok(entry.bundleBudget, `${entry.canonicalName} must publish a bundle budget`);

    for (const dependency of entry.dependencies) {
      assert.ok(
        names.includes(dependency),
        `${entry.canonicalName} has unknown dependency ${dependency}`,
      );
    }
  }
});

test("catalogued families match package exports and build entries", () => {
  assert.ok(packEntries, "vite-plus pack entries must be readable");

  for (const entry of stableEntries) {
    const exportTarget = packageManifest.exports[entry.packageSubpath];
    assert.equal(typeof exportTarget, "object", `${entry.packageSubpath} must be exported`);
    if (typeof exportTarget !== "object") continue;

    assert.equal(
      exportTarget.import,
      `./dist/${entry.canonicalName}.mjs`,
      `${entry.canonicalName} import output must follow the catalog`,
    );
    assert.equal(
      exportTarget.types,
      `./dist/${entry.canonicalName}.d.mts`,
      `${entry.canonicalName} types output must follow the catalog`,
    );
    assert.equal(
      packEntries?.[entry.canonicalName],
      entry.entryFile,
      `${entry.canonicalName} pack entry must follow the catalog`,
    );
  }
});

test("stable catalog entries have every required artifact", async () => {
  for (const entry of stableEntries) {
    const files = [entry.entryFile, entry.behaviorContract, ...entry.sourceFiles, ...entry.tests];
    for (const file of entry.typeTests ?? []) files.push(file);

    await Promise.all(files.map((file) => stat(path.resolve(file))));

    const behavior = await readFile(path.resolve(entry.behaviorContract), "utf8");
    assert.match(
      behavior,
      /^\|.+\|$/m,
      `${entry.canonicalName} behavior contract must include a normative table`,
    );

    if (entry.qualityGates.includes("vapor-compile")) {
      assert.ok(entry.rendererFixture, `${entry.canonicalName} must name its renderer fixture`);
      const sourceFixture = `src/${entry.rendererFixture}`;
      if (entry.sourceFiles.includes(sourceFixture as (typeof entry.sourceFiles)[number])) {
        await stat(path.resolve(sourceFixture));
      } else {
        assert.match(
          rendererGate,
          new RegExp(`filename:\\s*["']${entry.rendererFixture.replaceAll(".", "\\.")}["']`),
          `${entry.canonicalName} renderer fixture must be compiled by scripts/check-renderers.ts`,
        );
      }
    }

    if (entry.qualityGates.includes("type-inference")) {
      assert.ok(
        (entry.typeTests?.length ?? 0) > 0 ||
          entry.sourceFiles.some((file) => file.endsWith(".vue")),
        `${entry.canonicalName} must provide type tests or a typed SFC contract`,
      );
    }
  }
});

test("new family-owned SFC primitives keep implementation and tests together", () => {
  for (const [canonicalName, familyRoot] of uiFamilyRoots) {
    const entry = stableEntries.find((candidate) => candidate.canonicalName === canonicalName);

    assert.ok(entry, `${canonicalName} must stay catalogued`);
    assertFamilyPaths(canonicalName, familyRoot, "entry", [entry.entryFile]);
    assertFamilyPaths(canonicalName, familyRoot, "behavior contract", [entry.behaviorContract]);
    assertFamilyPaths(canonicalName, familyRoot, "source files", entry.sourceFiles);
    assertFamilyPaths(canonicalName, familyRoot, "tests", entry.tests);
    assertFamilyPaths(canonicalName, familyRoot, "type tests", entry.typeTests ?? []);
  }
});

test("accessibility families keep root compatibility barrels", async () => {
  await assertFamilyBarrels("accessibility", [
    "announcer",
    "focus",
    "focus-guards",
    "focus-scope",
    "inert-outside",
    "interaction-modality",
    "live-region",
    "scroll-lock",
    "visually-hidden",
  ]);
});

test("layout families keep root compatibility barrels", async () => {
  await assertFamilyBarrels("layout", [
    "aspect-ratio",
    "avatar",
    "card",
    "cluster",
    "container",
    "grid",
    "list",
    "separator",
    "spacer",
    "stack",
  ]);
});

test("typography families keep root compatibility barrels", async () => {
  await assertFamilyBarrels("typography", ["blockquote", "code", "heading", "kbd", "text"]);
});

test("navigation families keep root compatibility barrels", async () => {
  await assertFamilyBarrels("navigation", ["breadcrumb", "link", "pagination", "stepper", "tabs"]);
});

test("selection families keep root compatibility barrels", async () => {
  await assertFamilyBarrels("selection", [
    "checkbox",
    "listbox",
    "radio-group",
    "switch",
    "toggle",
    "toggle-group",
  ]);
});

test("overlay infrastructure families keep root compatibility barrels", async () => {
  await assertFamilyBarrels("overlays", [
    "alert-dialog",
    "dismissable-layer",
    "motion",
    "portal",
    "positioner",
    "presence",
    "transition",
  ]);
});

test("interaction families keep root compatibility barrels", async () => {
  await assertFamilyBarrels("interaction", [
    "drag-and-drop",
    "history",
    "hover",
    "long-press",
    "measure",
    "move",
    "pointer-grace",
    "press",
    "shortcut",
    "sortable",
    "spatial-navigation",
    "typeahead",
    "virtualizer",
  ]);
});

test("foundation families keep root compatibility barrels", async () => {
  await assertFamilyBarrels("foundations", ["collection", "composite-navigation", "primitive"]);
});

test("form families keep root compatibility barrels", async () => {
  await assertFamilyBarrels("form", ["error-summary"]);
});

test("progress family keeps root compatibility barrel", async () => {
  const source = await readFile(path.resolve("src/progress.ts"), "utf8");

  assert.match(
    source,
    /(?=.*from "\.\/families\/feedback\/progress\/progress-state\.ts")(?=.*from "\.\/families\/feedback\/progress\/progress-types\.ts")(?=.*from "\.\/families\/feedback\/progress\/progress\.vue")/s,
    "progress must keep its historical state, type, and component exports",
  );
});

test("feedback families keep root compatibility barrels", async () => {
  await assertFamilyBarrels("feedback", [
    "alert",
    "badge",
    "block-ui",
    "empty-state",
    "meter",
    "skeleton",
    "spinner",
  ]);
});

async function assertFamilyBarrels(area: string, names: readonly string[]): Promise<void> {
  for (const name of names) {
    const source = await readFile(path.resolve(`src/${name}.ts`), "utf8");

    assert.match(
      source,
      new RegExp(`from "\\./families/${area}/${name}/${name}\\.ts"`),
      `${name} must keep its historical source entry as a compatibility barrel`,
    );
  }
}
