import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import { test } from "vite-plus/test";

import { basicFamilyCatalog } from "./family-catalog-basics.ts";
import { foundationFamilyCatalog } from "./family-catalog-foundations.ts";
import { focusFamilyCatalog } from "./family-catalog-focus.ts";
import { interactionFamilyCatalog } from "./family-catalog-interactions.ts";
import { overlayFamilyCatalog } from "./family-catalog-overlays.ts";
import {
  assertFamilyPaths,
  rehomedFlatFamilies,
  rehomedFoundationUtilities,
  rootCompatibilityBarrelGroups,
} from "./family-layout-test-utils.ts";

const sourceRoot = path.resolve("src");
const familySfcPattern =
  /^families\/[a-z0-9]+(?:-[a-z0-9]+)*\/[a-z0-9]+(?:-[a-z0-9]+)*\/[a-z0-9]+(?:-[a-z0-9]+)*\.vue$/u;
const rootDeterministicIdImportPattern =
  /from\s+["']\.\.\/\.\.\/\.\.\/deterministic-id(?:-provider)?\.(?:ts|vue)["']/u;

const grandfatheredRootSfcFiles = [] as const;

test("new public SFCs live in family directories", () => {
  assert.deepEqual(rootSfcFiles(), grandfatheredRootSfcFiles);
});

test("family SFCs keep area and primitive directory segments", () => {
  const files = collectVueFiles(path.join(sourceRoot, "families"))
    .map((filename) => toPosixPath(path.relative(sourceRoot, filename)))
    .sort();
  const offenders = files.filter((filename) => !familySfcPattern.test(filename));

  assert.ok(files.length > 0);
  assert.deepEqual(offenders, []);
});

test("rehomed foundation utilities are cataloged from family directories", () => {
  const familiesByName = new Map(
    foundationFamilyCatalog.map((family) => [family.canonicalName, family]),
  );

  for (const familyName of rehomedFoundationUtilities) {
    const family = familiesByName.get(familyName);
    assert.ok(family, `${familyName} must remain in the foundation catalog`);

    const familyRoot = `src/families/foundations/${familyName}/`;
    assert.equal(family.entryFile, `${familyRoot}${familyName}.ts`);
    assert.deepEqual(family.sourceFiles, [`${familyRoot}${familyName}.ts`]);
    assert.equal(family.behaviorContract, `${familyRoot}${familyName}.behavior.md`);
    assert.deepEqual(family.tests, [`${familyRoot}${familyName}.test.ts`]);
    assert.deepEqual(family.typeTests, [`${familyRoot}${familyName}.types.test-d.ts`]);
  }
});

test("rehomed flat families are cataloged from family directories", () => {
  for (const { catalog, familyName, familyRoot } of rehomedFlatFamilies) {
    const family = catalog.find((entry) => entry.canonicalName === familyName);
    assert.ok(family, `${familyName} must remain cataloged`);

    assertFamilyPaths(familyName, familyRoot, "entry", [family.entryFile]);
    assertFamilyPaths(familyName, familyRoot, "behavior contract", [family.behaviorContract]);
    assertFamilyPaths(familyName, familyRoot, "source files", family.sourceFiles);
    assertFamilyPaths(familyName, familyRoot, "tests", family.tests);
    assertFamilyPaths(familyName, familyRoot, "type tests", family.typeTests ?? []);
  }
});

for (const { testName, barrels } of rootCompatibilityBarrelGroups) {
  test(testName, () => {
    for (const [filename, target] of barrels) {
      const source = fs.readFileSync(path.join(sourceRoot, filename), "utf8").trim();

      assert.equal(source, `export * from "${target}";`);
    }
  });
}

test("rehomed measure family is cataloged from a family directory", () => {
  const family = interactionFamilyCatalog.find((entry) => entry.canonicalName === "measure");
  assert.ok(family);

  const familyRoot = "src/families/interaction/measure/";
  assert.equal(family.entryFile, `${familyRoot}measure.ts`);
  assert.deepEqual(family.sourceFiles, [
    `${familyRoot}measure.ts`,
    `${familyRoot}measure-runtime.ts`,
    `${familyRoot}measure-types.ts`,
  ]);
  assert.equal(family.behaviorContract, `${familyRoot}measure.behavior.md`);
  assert.deepEqual(family.tests, [
    `${familyRoot}measure.test.ts`,
    `${familyRoot}measure-ssr.test.ts`,
  ]);
  assert.deepEqual(family.typeTests, [`${familyRoot}measure.types.test-d.ts`]);
  assert.equal(family.rendererFixture, "MeasureConsumer.vue");
});

test("rehomed error-summary family is cataloged from a family directory", () => {
  const family = foundationFamilyCatalog.find((entry) => entry.canonicalName === "error-summary");
  assert.ok(family);

  const familyRoot = "src/families/form/error-summary/";
  assert.equal(family.entryFile, `${familyRoot}error-summary.ts`);
  assert.deepEqual(family.sourceFiles, [
    `${familyRoot}error-summary.vue`,
    `${familyRoot}error-summary.ts`,
    `${familyRoot}error-summary-runtime.ts`,
    `${familyRoot}error-summary-types.ts`,
  ]);
  assert.equal(family.behaviorContract, `${familyRoot}error-summary.behavior.md`);
  assert.deepEqual(family.tests, [
    `${familyRoot}error-summary.test.ts`,
    `${familyRoot}error-summary-ssr.test.ts`,
  ]);
  assert.deepEqual(family.typeTests, [`${familyRoot}error-summary.types.test-d.ts`]);
  assert.equal(family.rendererFixture, "families/form/error-summary/error-summary.vue");
});

test("rehomed command family is cataloged from a family directory", () => {
  const family = foundationFamilyCatalog.find((entry) => entry.canonicalName === "command");
  assert.ok(family);

  const familyRoot = "src/families/foundations/command/";
  assert.equal(family.entryFile, `${familyRoot}command.ts`);
  assert.deepEqual(family.sourceFiles, [
    `${familyRoot}command.ts`,
    `${familyRoot}command-types.ts`,
  ]);
  assert.equal(family.behaviorContract, `${familyRoot}command.behavior.md`);
  assert.deepEqual(family.tests, [
    `${familyRoot}command.test.ts`,
    `${familyRoot}command-ssr.test.ts`,
  ]);
  assert.deepEqual(family.typeTests, [`${familyRoot}command.types.test-d.ts`]);
  assert.equal(family.rendererFixture, "CommandConsumer.vue");
});

test("rehomed disclosure families are cataloged from family directories", () => {
  const family = basicFamilyCatalog.find((entry) => entry.canonicalName === "collapsible");
  assert.ok(family);

  const familyRoot = "src/families/disclosure/collapsible/";
  assert.equal(family.entryFile, `${familyRoot}collapsible.ts`);
  assert.deepEqual(family.sourceFiles, [
    `${familyRoot}collapsible-root.vue`,
    `${familyRoot}collapsible-trigger.vue`,
    `${familyRoot}collapsible-content.vue`,
    `${familyRoot}collapsible.ts`,
    `${familyRoot}collapsible-context.ts`,
    `${familyRoot}collapsible-types.ts`,
  ]);
  assert.equal(family.behaviorContract, `${familyRoot}collapsible.behavior.md`);
  assert.deepEqual(family.tests, [
    `${familyRoot}collapsible.test.ts`,
    `${familyRoot}collapsible-ssr.test.ts`,
  ]);
  assert.deepEqual(family.typeTests, [`${familyRoot}collapsible.types.test-d.ts`]);
  assert.equal(family.rendererFixture, "CollapsibleConsumer.vue");
});

test("root collapsible entry stays a compatibility-only barrel", () => {
  const source = fs.readFileSync(path.join(sourceRoot, "collapsible.ts"), "utf8").trim();

  assert.equal(source, 'export * from "./families/disclosure/collapsible/collapsible.ts";');
});

test("rehomed navigation link family is cataloged from a family directory", () => {
  const family = basicFamilyCatalog.find((entry) => entry.canonicalName === "link");
  assert.ok(family);

  const familyRoot = "src/families/navigation/link/";
  assert.equal(family.entryFile, `${familyRoot}link.ts`);
  assert.deepEqual(family.sourceFiles, [
    `${familyRoot}link-anchor.vue`,
    `${familyRoot}link.ts`,
    `${familyRoot}link-types.ts`,
  ]);
  assert.equal(family.behaviorContract, `${familyRoot}link.behavior.md`);
  assert.deepEqual(family.tests, [`${familyRoot}link.test.ts`]);
  assert.deepEqual(family.typeTests, [`${familyRoot}link.types.test-d.ts`]);
  assert.equal(family.rendererFixture, "families/navigation/link/link-anchor.vue");
});

test("rehomed overlay transition family is cataloged from a family directory", () => {
  const family = overlayFamilyCatalog.find((entry) => entry.canonicalName === "transition");
  assert.ok(family);

  const familyRoot = "src/families/overlays/transition/";
  assert.equal(family.entryFile, `${familyRoot}transition.ts`);
  assert.deepEqual(family.sourceFiles, [
    `${familyRoot}transition.vue`,
    `${familyRoot}transition.ts`,
    `${familyRoot}transition-runtime.ts`,
    `${familyRoot}transition-types.ts`,
  ]);
  assert.equal(family.behaviorContract, `${familyRoot}transition.behavior.md`);
  assert.deepEqual(family.tests, [
    `${familyRoot}transition.test.ts`,
    `${familyRoot}transition-ssr.test.ts`,
  ]);
  assert.deepEqual(family.typeTests, [`${familyRoot}transition.types.test-d.ts`]);
  assert.equal(family.rendererFixture, "families/overlays/transition/transition.vue");
});

test("rehomed id family is cataloged from a family directory", () => {
  const family = focusFamilyCatalog.find((entry) => entry.canonicalName === "id");
  assert.ok(family);

  const familyRoot = "src/families/foundations/id/";
  assert.equal(family.entryFile, `${familyRoot}id.ts`);
  assert.deepEqual(family.sourceFiles, [
    `${familyRoot}deterministic-id-provider.vue`,
    `${familyRoot}id.ts`,
    `${familyRoot}deterministic-id.ts`,
  ]);
  assert.equal(family.behaviorContract, `${familyRoot}id.behavior.md`);
  assert.deepEqual(family.tests, [`${familyRoot}id.test.ts`]);
  assert.deepEqual(family.typeTests, [`${familyRoot}id.types.test-d.ts`]);
  assert.equal(family.rendererFixture, "families/foundations/id/deterministic-id-provider.vue");
});

test("family sources import deterministic IDs from the foundation family", () => {
  const offenders = collectSourceFiles(path.join(sourceRoot, "families"))
    .map((filename) => ({
      filename: toPosixPath(path.relative(sourceRoot, filename)),
      source: fs.readFileSync(filename, "utf8"),
    }))
    .filter(({ source }) => rootDeterministicIdImportPattern.test(source))
    .map(({ filename }) => filename)
    .sort();

  assert.deepEqual(offenders, []);
});

function rootSfcFiles(): readonly string[] {
  return fs
    .readdirSync(sourceRoot, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".vue"))
    .map((entry) => entry.name)
    .sort();
}

function collectVueFiles(directory: string): readonly string[] {
  return collectSourceFiles(directory).filter((filename) => filename.endsWith(".vue"));
}

function collectSourceFiles(directory: string): readonly string[] {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) return collectSourceFiles(filename);
    return entry.isFile() && /\.(?:ts|vue)$/u.test(entry.name) ? [filename] : [];
  });
}

function toPosixPath(filename: string): string {
  return filename.split(path.sep).join(path.posix.sep);
}
