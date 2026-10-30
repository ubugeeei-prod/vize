import assert from "node:assert/strict";
import { test } from "node:test";
import {
  ariaComponentProjects,
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
  assert.equal(projects.length, 10);
  assert.equal(
    projects.reduce((count, project) => count + project.phases.length, 0),
    14,
  );
  for (const [id, column] of [
    ["declared-prop-cycle", 32],
    ["declared-prop-cycle-crlf-unicode", 46],
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
      `src/parent.vue(6,${column}): error TS2322: Type 'boolean' is not assignable to type 'string | number | undefined'.\n`,
    );
    assert.equal(invalid.parent.split(/\r\n|\n/u)[5].slice(column - 1, column + 4), "false");
    assert.deepEqual(
      [valid.expectedStatus, invalid.expectedStatus, repaired.expectedStatus],
      [0, 2, 0],
    );
    assert.equal(valid.expectedStdout, "");
    assert.equal(repaired.expectedStdout, "");
  }
});
