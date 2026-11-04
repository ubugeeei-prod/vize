import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  ariaComponentProjects,
  ariaPropReference,
  ariaPropReferenceStdout,
  reportedAriaChild,
  reportedAriaConfig,
  reportedAriaParent,
} from "../../tools/support/compat/content-mapper/aria-component-projects.ts";
import {
  CHECKER_SDK_ENTRY,
  PUBLISHED_SDK_ENTRY,
  assertShippedDeclarationModule,
  sharedDeclarationEntry,
} from "../../tools/support/compat/content-mapper/aria-sdk-entry.ts";

test("the package regression retains the original reporter inputs", () => {
  const original = ariaComponentProjects().find((project) => project.id === "original");
  assert.ok(original);
  assert.equal(original.child, reportedAriaChild);
  assert.equal(original.phases[0].parent, reportedAriaParent);
  assert.equal(
    reportedAriaChild,
    '<script setup lang="ts">\ndefineProps<{ size?: string | number }>();\n</script>\n\n<template>\n  <div>{{ size }}</div>\n</template>\n',
  );
  assert.equal(
    reportedAriaParent,
    '<script setup lang="ts">\nimport Child from "./child.vue";\n</script>\n\n<template>\n  <Child aria-label="x" />\n</template>\n',
  );
  assert.deepEqual(reportedAriaConfig, {
    compilerOptions: {
      strict: true,
      module: "esnext",
      moduleResolution: "bundler",
      target: "esnext",
      jsx: "preserve",
      skipLibCheck: true,
      noEmit: true,
    },
    include: ["src/**/*.ts", "src/**/*.vue"],
    contentMappers: [{ package: "vize", extensions: [".vue"] }],
  });
});

test("declared prop errors keep exact authored UTF16 coordinates through repair", () => {
  const projects = ariaComponentProjects();
  assert.equal(projects.length, 12);
  assert.equal(
    projects.reduce((count, project) => count + project.phases.length, 0),
    20,
  );
  for (const [id, column] of [
    ["declared-prop-cycle", 26],
    ["declared-prop-cycle-crlf-unicode", 40],
  ] as const) {
    const project = projects.find((project) => project.id === id)!;
    assert.ok(project);
    assert.deepEqual(
      project.phases.map((phase) => phase.id),
      ["valid", "invalid", "repaired"],
    );
    const [valid, invalid, repaired] = project.phases;
    assert.equal(valid.parent, repaired.parent);
    assert.equal(invalid.parent.replace(':size="false"', ':size="20"'), valid.parent);
    assert.equal(
      invalid.expectedStdout,
      `src/parent.vue(6,${column}): error TS2322: Type 'false' is not assignable to type 'string | number | undefined'.\n`,
    );
    assert.equal(invalid.parent.split(/\r\n|\n/u)[5].slice(column - 1, column + 3), "size");
    assert.deepEqual(
      [valid.expectedStatus, invalid.expectedStatus, repaired.expectedStatus],
      [0, 2, 0],
    );
    assert.equal(valid.expectedStdout, "");
    assert.equal(repaired.expectedStdout, "");
  }
});

test("required camel ARIA props survive authored PropertyKey shadows through repair", () => {
  const projects = ariaComponentProjects(),
    control = projects.find((project) => project.id === "required-camel-aria-cycle")!,
    shadowed = projects.find(
      (project) => project.id === "shadowed-property-key-required-camel-aria-cycle",
    )!;
  assert.ok(control && shadowed);
  for (const project of [control, shadowed]) {
    assert.deepEqual(
      project.phases.map((phase) => phase.id),
      ["valid", "invalid", "repaired"],
    );
    const [valid, invalid, repaired] = project.phases;
    assert.equal(valid.parent, repaired.parent);
    assert.equal(invalid.parent.replace(':size="false"', ':size="20"'), valid.parent);
    assert.ok(valid.parent.includes("ariaLabel: { type: String, required: true }"));
    assert.equal(invalid.parent.split("\n")[5].slice(34, 38), "size");
    assert.deepEqual(
      [valid.expectedStatus, invalid.expectedStatus, repaired.expectedStatus],
      [0, 2, 0],
    );
    assert.equal(
      invalid.expectedStdout,
      "src/parent.vue(6,35): error TS2322: Type 'false' is not assignable to type 'string | number | undefined'.\n",
    );
  }
  for (const [index, phase] of shadowed.phases.entries()) {
    assert.ok(phase.parent.includes("type PropertyKey = 0;"));
    assert.equal(phase.parent.replace("type PropertyKey = 0;", ""), control.phases[index].parent);
  }
});

test("the negative reference is an independent plain TypeScript prop assignment", () => {
  assert.equal(
    ariaPropReference,
    "declare function accept(props: { size?: string | number }): void;\naccept({ size: false });\n",
  );
  assert.equal(
    ariaPropReferenceStdout,
    "reference.ts(2,10): error TS2322: Type 'false' is not assignable to type 'string | number | undefined'.\n",
  );
  assert.equal(ariaPropReference.split("\n")[1].slice(9, 13), "size");
});

const checkerExport = {
  "./virtual-ts-checker": { types: "./virtual-ts-checker.d.ts" },
  "./virtual-ts": { types: "./virtual-ts.d.ts" },
};
const publishedExport = { "./virtual-ts": { types: "./virtual-ts.d.ts" } };
const checkerText = 'import type {} from "vize/virtual-ts-checker";\nexport type Props = {};\n';
const publishedText = 'import type {} from "vize/virtual-ts";\nexport type Props = {};\n';
const shippedModule = /exercise the actual shipped shared declaration module/u;

test("source and published packages bind their own declaration entry", () => {
  const packageRoot = fileURLToPath(new URL("../../npm/cli/", import.meta.url));
  const manifest = JSON.parse(readFileSync(path.join(packageRoot, "package.json"), "utf8"));
  const files = ["virtual-ts-checker.d.ts", "virtual-ts.d.ts"].filter((file) =>
    existsSync(path.join(packageRoot, file)),
  );
  assert.deepEqual(files, ["virtual-ts-checker.d.ts", "virtual-ts.d.ts"]);
  assert.equal(
    sharedDeclarationEntry({ mode: "source", exports: manifest.exports, files }),
    CHECKER_SDK_ENTRY,
  );
  assert.equal(
    sharedDeclarationEntry({ mode: "published", exports: manifest.exports, files }),
    CHECKER_SDK_ENTRY,
  );
  assert.equal(
    sharedDeclarationEntry({
      mode: "published",
      exports: publishedExport,
      files: ["virtual-ts.d.ts"],
    }),
    PUBLISHED_SDK_ENTRY,
  );
  assert.equal(
    sharedDeclarationEntry({
      mode: "source",
      exports: checkerExport,
      files: ["virtual-ts-checker.d.ts", "virtual-ts.d.ts"],
    }),
    CHECKER_SDK_ENTRY,
  );
  for (const contract of [
    { mode: "source" as const, exports: publishedExport, files: ["virtual-ts.d.ts"] },
    { mode: "published" as const, exports: {}, files: [] },
    {
      mode: "published" as const,
      exports: { "./virtual-ts-checker": { types: "./other.d.ts" } },
      files: ["virtual-ts-checker.d.ts"],
    },
    {
      mode: "published" as const,
      exports: { "./virtual-ts": { types: "./not-virtual-ts.d.ts" } },
      files: ["virtual-ts.d.ts"],
    },
    {
      mode: "source" as const,
      exports: { "./not-an-sdk": { types: "./virtual-ts-checker.d.ts" } },
      files: ["virtual-ts-checker.d.ts"],
    },
  ])
    assert.throws(() => sharedDeclarationEntry(contract), shippedModule);
});

test("a missing or incorrect SDK entry cannot satisfy the shipped module", () => {
  assertShippedDeclarationModule(checkerText, CHECKER_SDK_ENTRY);
  assertShippedDeclarationModule(publishedText, PUBLISHED_SDK_ENTRY);
  for (const [text, entry] of [
    ["export type Props = {};\n", CHECKER_SDK_ENTRY],
    [publishedText, CHECKER_SDK_ENTRY],
    [checkerText, PUBLISHED_SDK_ENTRY],
    ['import type {} from "vize/not-the-sdk";\n', CHECKER_SDK_ENTRY],
    [`${checkerText}import type {} from "vize/not-the-sdk";\n`, CHECKER_SDK_ENTRY],
    [`${checkerText}${checkerText}`, CHECKER_SDK_ENTRY],
    ['import type {} from "vize:virtual-ts";\n', PUBLISHED_SDK_ENTRY],
    [`${checkerText}import type {} from "vize:virtual-ts";\n`, CHECKER_SDK_ENTRY],
    [`${checkerText}// leftover vize:virtual-ts\n`, CHECKER_SDK_ENTRY],
  ] as const)
    assert.throws(() => assertShippedDeclarationModule(text, entry), shippedModule);
  for (const [text, entry] of [
    [`${checkerText}type $PropValue<P> = P;\n`, CHECKER_SDK_ENTRY],
    [`${publishedText}type $PropValue<P> = P;\n`, PUBLISHED_SDK_ENTRY],
  ] as const)
    assert.throws(
      () => assertShippedDeclarationModule(text, entry),
      /embedded helpers would hide the published regression/u,
    );
  assert.throws(
    () => assertShippedDeclarationModule(checkerText, "vize/not-the-sdk"),
    shippedModule,
  );
});

test("package qualification asks each packet for that SDK entry", () => {
  const runner = readFileSync(
    new URL("../../tools/support/compat/content-mapper/aria-component-attrs.ts", import.meta.url),
    "utf8",
  );
  assert.match(runner, /sharedDeclarationEntry\(/u);
  assert.match(runner, /assertShippedDeclarationModule\(text, declarationEntry\)/u);
  assert.match(runner, /virtual-ts-checker\.d\.ts/u);
  assert.doesNotMatch(runner, /from "vize\/virtual-ts(?:-checker)?";/u);
});
