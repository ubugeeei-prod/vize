import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import { test } from "vite-plus/test";

import { foundationFamilyCatalog } from "./family-catalog-foundations.ts";

const sourceRoot = path.resolve("src");
const familySfcPattern =
  /^families\/[a-z0-9]+(?:-[a-z0-9]+)*\/[a-z0-9]+(?:-[a-z0-9]+)*\/[a-z0-9]+(?:-[a-z0-9]+)*\.vue$/u;
const rehomedFoundationUtilities = ["context", "controllable-state"] as const;
const foundationCompatibilityBarrels = [
  ["context.ts", "./families/foundations/context/context.ts"],
  ["controllable-state.ts", "./families/foundations/controllable-state/controllable-state.ts"],
] as const;

const grandfatheredRootSfcFiles = [
  "collapsible-content.vue",
  "collapsible-root.vue",
  "collapsible-trigger.vue",
  "deterministic-id-provider.vue",
  "error-summary.vue",
  "link-anchor.vue",
  "primitive-element.vue",
  "transition.vue",
] as const;

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

test("root foundation utilities stay compatibility-only barrels", () => {
  for (const [filename, target] of foundationCompatibilityBarrels) {
    const source = fs.readFileSync(path.join(sourceRoot, filename), "utf8").trim();

    assert.equal(source, `export * from "${target}";`);
  }
});

function rootSfcFiles(): readonly string[] {
  return fs
    .readdirSync(sourceRoot, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".vue"))
    .map((entry) => entry.name)
    .sort();
}

function collectVueFiles(directory: string): readonly string[] {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) return collectVueFiles(filename);
    return entry.isFile() && entry.name.endsWith(".vue") ? [filename] : [];
  });
}

function toPosixPath(filename: string): string {
  return filename.split(path.sep).join(path.posix.sep);
}
