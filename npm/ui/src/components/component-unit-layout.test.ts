import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import { test } from "vite-plus/test";

import { uiFamilyCatalog } from "../catalog/family-catalog.ts";

const componentsRoot = path.resolve("src/components");
const componentNamePattern = /^[a-z0-9]+(?:-[a-z0-9]+)*$/u;

test("public component units follow the readable source layout", () => {
  const componentNames = fs
    .readdirSync(componentsRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort();

  assert.ok(componentNames.length > 0, "at least one component unit must exist");

  for (const componentName of componentNames) {
    assert.match(componentName, componentNamePattern);
    assertComponentUnit(componentName);
  }
});

function assertComponentUnit(componentName: string): void {
  const relativeRoot = `src/components/${componentName}`;
  const absoluteRoot = path.resolve(relativeRoot);
  const catalogEntry = uiFamilyCatalog.find((entry) => entry.canonicalName === componentName);

  assert.ok(catalogEntry, `${componentName} must have colocated catalogue metadata`);
  assert.equal(catalogEntry.entryFile, `${relativeRoot}/index.ts`);
  assert.equal(catalogEntry.documentation, `${relativeRoot}/${componentName}.guide.md`);

  for (const filename of [
    `${componentName}.vue`,
    `${componentName}.behavior.md`,
    `${componentName}.guide.md`,
    "index.ts",
    "metadata.ts",
    "types.ts",
  ]) {
    assert.ok(
      fs.statSync(path.join(absoluteRoot, filename)).isFile(),
      `${relativeRoot}/${filename}`,
    );
  }

  for (const directory of ["examples", "stories", "tests"]) {
    assert.ok(
      fs.statSync(path.join(absoluteRoot, directory)).isDirectory(),
      `${relativeRoot}/${directory}/`,
    );
  }

  const stylesheet = path.join(absoluteRoot, "styles.css");
  if (fs.existsSync(stylesheet)) assertComponentStyles(fs.readFileSync(stylesheet, "utf8"));

  const ownedPaths = [
    catalogEntry.entryFile,
    catalogEntry.behaviorContract,
    ...(catalogEntry.documentation == null ? [] : [catalogEntry.documentation]),
    ...catalogEntry.sourceFiles,
    ...catalogEntry.tests,
    ...(catalogEntry.typeTests ?? []),
  ];
  assert.ok(
    ownedPaths.every((filename) => filename.startsWith(`${relativeRoot}/`)),
    `${componentName} metadata must not point outside its component unit`,
  );
}

function assertComponentStyles(source: string): void {
  // source-contract: semantic selector ownership is an authored CSS API invariant.
  assert.match(
    source,
    /:where\(\[data-vize-ui=/u,
    "styles must start from a semantic component hook",
  );
  // source-contract: native nesting is required in source before Lightning CSS lowering.
  assert.match(source, /&(?::|\[)/u, "component states must use native CSS nesting");
  // source-contract: public source must never establish a BEM selector contract.
  assert.doesNotMatch(source, /\.[a-z0-9-]+(?:__|--)[a-z0-9-]+/iu, "BEM is not a public API");
  // source-contract: component states stay nested beneath the semantic root in authored CSS.
  assert.doesNotMatch(
    source,
    /\[data-vize-ui=[^\]]+\][^{\n]*(?::(?:active|focus|hover)|\[data-(?:size|state|tone|variant)=)/u,
    "state selectors must nest beneath the semantic component hook",
  );

  const selectorLines = source
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => /^(?::|&|from\b|to\b).*\{$/u.test(line));
  for (const selector of selectorLines) {
    assert.ok(
      selector.length <= 100,
      `split long selector across semantic nested rules: ${selector}`,
    );
  }
}
