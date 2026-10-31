import assert from "node:assert/strict";
import { test } from "node:test";
import {
  ariaComponentProjects,
  ariaPropReference,
  ariaPropReferenceStdout,
  reportedAriaChild,
  reportedAriaConfig,
  reportedAriaParent,
} from "../../tools/support/compat/content-mapper/aria-component-projects.ts";

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
