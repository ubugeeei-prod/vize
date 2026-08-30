import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import path from "node:path";

import { test } from "vite-plus/test";

import config from "../vite.config.ts";
import { UI_FAMILY_CATALOG_SCHEMA_VERSION, uiFamilyCatalog } from "./family-catalog.ts";

type PackageExport = string | { readonly import: string; readonly types: string };

const stableEntries = uiFamilyCatalog.filter((entry) => entry.maturity === "stable");
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
    assert.ok(
      entry.qualityGates.includes("behavior-contract"),
      `${entry.canonicalName} must require behavior evidence`,
    );
    assert.ok(
      entry.qualityGates.includes("mounted-dom"),
      `${entry.canonicalName} must require mounted-DOM evidence`,
    );
    assert.ok(
      entry.qualityGates.includes("bundle-size"),
      `${entry.canonicalName} must require a bundle budget`,
    );
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
  const familyRoots = new Map([
    ["alert", "src/families/feedback/alert/"],
    ["aspect-ratio", "src/families/layout/aspect-ratio/"],
    ["avatar", "src/families/layout/avatar/"],
    ["banner", "src/families/feedback/banner/"],
    ["badge", "src/families/feedback/badge/"],
    ["breadcrumb", "src/families/navigation/breadcrumb/"],
    ["blockquote", "src/families/typography/blockquote/"],
    ["block-ui", "src/families/feedback/block-ui/"],
    ["button-group", "src/families/actions/button-group/"],
    ["callout", "src/families/feedback/callout/"],
    ["card", "src/families/layout/card/"],
    ["checkbox", "src/families/selection/checkbox/"],
    ["cluster", "src/families/layout/cluster/"],
    ["code", "src/families/typography/code/"],
    ["container", "src/families/layout/container/"],
    ["dialog", "src/families/overlays/dialog/"],
    ["empty-state", "src/families/feedback/empty-state/"],
    ["grid", "src/families/layout/grid/"],
    ["heading", "src/families/typography/heading/"],
    ["icon", "src/families/layout/icon/"],
    ["icon-button", "src/families/layout/icon/"],
    ["kbd", "src/families/typography/kbd/"],
    ["list", "src/families/layout/list/"],
    ["listbox", "src/families/selection/listbox/"],
    ["locale", "src/families/i18n/locale/"],
    ["fullscreen-button", "src/families/actions/fullscreen-button/"],
    ["meter", "src/families/feedback/meter/"],
    ["native-select", "src/families/selection/native-select/"],
    ["pagination", "src/families/navigation/pagination/"],
    ["print-button", "src/families/actions/print-button/"],
    ["progress", "src/families/feedback/progress/"],
    ["progress-bar", "src/families/feedback/progress-bar/"],
    ["radio-group", "src/families/selection/radio-group/"],
    ["rating", "src/families/form/rating/"],
    ["share-button", "src/families/actions/share-button/"],
    ["scroll-area", "src/families/layout/scroll-area/"],
    ["separator", "src/families/layout/separator/"],
    ["skip-link", "src/families/navigation/skip-link/"],
    ["skeleton", "src/families/feedback/skeleton/"],
    ["spacer", "src/families/layout/spacer/"],
    ["stack", "src/families/layout/stack/"],
    ["stepper", "src/families/navigation/stepper/"],
    ["spinner", "src/families/feedback/spinner/"],
    ["status-light", "src/families/feedback/status-light/"],
    ["surface", "src/families/layout/surface/"],
    ["switch", "src/families/selection/switch/"],
    ["table", "src/families/data/table/"],
    ["tabs", "src/families/navigation/tabs/"],
    ["text", "src/families/typography/text/"],
    ["toggle", "src/families/selection/toggle/"],
    ["toggle-group", "src/families/selection/toggle-group/"],
    ["toolbar", "src/families/actions/toolbar/"],
    ["tooltip", "src/families/overlays/tooltip/"],
  ]);

  for (const [canonicalName, familyRoot] of familyRoots) {
    const entry = stableEntries.find((candidate) => candidate.canonicalName === canonicalName);

    assert.ok(entry, `${canonicalName} must stay catalogued`);
    assert.ok(
      entry.entryFile.startsWith(familyRoot),
      `${canonicalName} entry must stay in its family folder`,
    );
    assert.ok(
      entry.behaviorContract.startsWith(familyRoot),
      `${canonicalName} behavior contract must stay in its family folder`,
    );
    assert.ok(
      entry.sourceFiles.every((file) => file.startsWith(familyRoot)),
      `${canonicalName} source files must stay beside the family source`,
    );
    assert.ok(
      entry.tests.every((file) => file.startsWith(familyRoot)),
      `${canonicalName} tests must stay beside the family source`,
    );
    assert.ok(
      entry.typeTests?.every((file) => file.startsWith(familyRoot)) ?? true,
      `${canonicalName} type tests must stay beside the family source`,
    );
  }
});

test("layout families keep root compatibility barrels", async () => {
  for (const name of [
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
  ] as const) {
    const source = await readFile(path.resolve(`src/${name}.ts`), "utf8");

    assert.match(
      source,
      new RegExp(`from "\\./families/layout/${name}/${name}\\.ts"`),
      `${name} must keep its historical source entry as a compatibility barrel`,
    );
  }
});

test("typography families keep root compatibility barrels", async () => {
  for (const name of ["blockquote", "code", "heading", "kbd", "text"] as const) {
    const source = await readFile(path.resolve(`src/${name}.ts`), "utf8");

    assert.match(
      source,
      new RegExp(`from "\\./families/typography/${name}/${name}\\.ts"`),
      `${name} must keep its historical source entry as a compatibility barrel`,
    );
  }
});

test("navigation families keep root compatibility barrels", async () => {
  for (const name of ["breadcrumb", "pagination", "stepper", "tabs"] as const) {
    const source = await readFile(path.resolve(`src/${name}.ts`), "utf8");

    assert.match(
      source,
      new RegExp(`from "\\./families/navigation/${name}/${name}\\.ts"`),
      `${name} must keep its historical source entry as a compatibility barrel`,
    );
  }
});

test("selection families keep root compatibility barrels", async () => {
  for (const name of [
    "checkbox",
    "listbox",
    "radio-group",
    "switch",
    "toggle",
    "toggle-group",
  ] as const) {
    const source = await readFile(path.resolve(`src/${name}.ts`), "utf8");

    assert.match(
      source,
      new RegExp(`from "\\./families/selection/${name}/${name}\\.ts"`),
      `${name} must keep its historical source entry as a compatibility barrel`,
    );
  }
});

test("progress family keeps root compatibility barrel", async () => {
  const source = await readFile(path.resolve("src/progress.ts"), "utf8");

  assert.match(
    source,
    /from "\.\/families\/feedback\/progress\/progress-state\.ts"/,
    "progress must keep its historical state export through the root barrel",
  );
  assert.match(
    source,
    /from "\.\/families\/feedback\/progress\/progress-types\.ts"/,
    "progress must keep its historical type export through the root barrel",
  );
  assert.match(
    source,
    /from "\.\/families\/feedback\/progress\/progress\.vue"/,
    "progress must keep its historical component export through the root barrel",
  );
});

test("feedback families keep root compatibility barrels", async () => {
  for (const name of [
    "alert",
    "badge",
    "block-ui",
    "empty-state",
    "meter",
    "skeleton",
    "spinner",
  ] as const) {
    const source = await readFile(path.resolve(`src/${name}.ts`), "utf8");

    assert.match(
      source,
      new RegExp(`from "\\./families/feedback/${name}/${name}\\.ts"`),
      `${name} must keep its historical source entry as a compatibility barrel`,
    );
  }
});
