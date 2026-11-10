import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import { compareLintFindings as compareCompatibleLintFindings } from "../../tools/support/compat/fixtures/lint-divergence.ts";
import { compareLintFindings as compareCurrentLintFindingsRaw } from "../../tools/support/fixtures/lint_divergence/compare.ts";
import { currentContext, cwd, eslintResult, span } from "./_helpers/lint-divergence-fixture.ts";

interface Comparison {
  schema: string;
  version: number;
  summary: {
    falsePositiveCount: number;
    falseNegativeCount: number;
    intentionalDivergenceCount: number;
    ruleLocationDivergenceCount: number;
  };
  falsePositives: unknown[];
  falseNegatives: unknown[];
  intentionalDivergences: { file: string }[];
}

const compareCurrent = (
  project: string,
  root: string,
  ruleMap: unknown,
  patinaFindings: unknown[],
  eslintResults: unknown[],
  context: unknown,
): Comparison =>
  compareCurrentLintFindingsRaw(
    project,
    root,
    ruleMap as never,
    patinaFindings as never,
    eslintResults as never,
    context as never,
  ) as unknown as Comparison;
const compareCompatible = (
  input: Parameters<typeof compareCompatibleLintFindings>[0],
): Comparison => compareCompatibleLintFindings(input) as unknown as Comparison;

test("static styles recovered through a TypeScript cast are intentional on both reporters", () => {
  const directory = `${cwd}/src`;
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(
    `${directory}/CastStyle.vue`,
    `<template>\n  <div :style="{ color: 'red', width: size } as StyleValue" />\n</template>\n`,
  );
  const map = {
    upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
    entries: {
      "vue/no-static-inline-styles": {
        status: "mapped",
        patinaRule: "vue/no-static-inline-styles",
      },
    },
  };
  const findings = [
    {
      file: "src/CastStyle.vue",
      ruleId: "vue/no-static-inline-styles",
      severity: 1,
      ...span(2, 18, 2, 30),
      message: "Static inline style attributes are not allowed.",
    },
  ];
  const current = compareCurrent(
    "fixture",
    cwd,
    map,
    findings,
    [],
    currentContext(map, ["src/CastStyle.vue"]),
  );
  const compatible = compareCompatible({
    projectId: "fixture",
    cwd,
    ruleMap: map,
    patinaFindings: findings,
    eslintResults: [],
  });
  for (const result of [current, compatible]) {
    assert.equal(result.summary.falsePositiveCount, 0);
    assert.equal(result.summary.intentionalDivergenceCount, 1);
  }
});

test("native-only dynamic components are intentional on both reporters", () => {
  const directory = `${cwd}/src`;
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(
    `${directory}/NativeDynamic.vue`,
    `<template>\n  <component :is="inline ? 'label' : 'p'" v-html="content" />\n</template>\n`,
  );
  const map = {
    upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
    entries: {
      "vue/no-v-text-v-html-on-component": {
        status: "mapped",
        patinaRule: "vue/no-v-text-v-html-on-component",
      },
    },
  };
  const baseline = [
    eslintResult("src/NativeDynamic.vue", [
      {
        ruleId: "vue/no-v-text-v-html-on-component",
        severity: 2,
        ...span(2, 48, 2, 64),
        message: "Using v-html on component may break component's content.",
      },
    ]),
  ];
  const current = compareCurrent(
    "fixture",
    cwd,
    map,
    [],
    baseline,
    currentContext(map, ["src/NativeDynamic.vue"]),
  );
  const compatible = compareCompatible({
    projectId: "fixture",
    cwd,
    ruleMap: map,
    patinaFindings: [],
    eslintResults: baseline,
  });
  for (const result of [current, compatible]) {
    assert.equal(result.summary.falseNegativeCount, 0);
    assert.equal(result.summary.intentionalDivergenceCount, 1);
  }
});

test("fragment-root foreign elements are intentional on both reporters", () => {
  const directory = `${cwd}/src`;
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(
    `${directory}/ForeignRoot.vue`,
    `<template>\n  <g v-for="item in items" v-html="content" />\n  <text fontSize="16" />\n</template>\n`,
  );
  const map = {
    upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
    entries: {
      "vue/attribute-hyphenation": {
        status: "mapped",
        patinaRule: "vue/attribute-hyphenation",
      },
      "vue/no-v-text-v-html-on-component": {
        status: "mapped",
        patinaRule: "vue/no-v-text-v-html-on-component",
      },
      "vue/require-v-for-key": {
        status: "mapped",
        patinaRule: "vue/require-v-for-key",
      },
      "vue/valid-v-for": { status: "mapped", patinaRule: "vue/valid-v-for" },
    },
  };
  const patina = [
    {
      file: "src/ForeignRoot.vue",
      ruleId: "vue/require-v-for-key",
      severity: 2,
      ...span(2, 3, 2, 51),
      message: "Elements in iteration expect a v-bind:key directive.",
    },
  ];
  const baseline = [
    eslintResult("src/ForeignRoot.vue", [
      {
        ruleId: "vue/valid-v-for",
        severity: 2,
        ...span(2, 6, 2, 27),
        message: "Custom elements in iteration require v-bind:key directives.",
      },
      {
        ruleId: "vue/no-v-text-v-html-on-component",
        severity: 2,
        ...span(2, 29, 2, 45),
        message: "Using v-html on component may break component content.",
      },
      {
        ruleId: "vue/attribute-hyphenation",
        severity: 1,
        ...span(3, 9, 3, 22),
        message: "Attribute 'fontSize' must be hyphenated.",
      },
    ]),
  ];
  const current = compareCurrent(
    "fixture",
    cwd,
    map,
    patina,
    baseline,
    currentContext(map, ["src/ForeignRoot.vue"]),
  );
  const compatible = compareCompatible({
    projectId: "fixture",
    cwd,
    ruleMap: map,
    patinaFindings: patina,
    eslintResults: baseline,
  });
  for (const result of [current, compatible]) {
    assert.equal(result.summary.falsePositiveCount, 0);
    assert.equal(result.summary.falseNegativeCount, 0);
    assert.equal(result.summary.intentionalDivergenceCount, 4);
  }
});

test("references before local ref declarations are intentional on both reporters", () => {
  const directory = `${cwd}/src`;
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(
    `${directory}/EarlyRef.vue`,
    `<script setup lang="ts">\nconst chosen = palette || []\nconst palette = computed(() => [])\n</script>\n`,
  );
  const map = {
    upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
    entries: {
      "vue/no-ref-as-operand": {
        status: "mapped",
        patinaRule: "script/no-ref-as-operand",
      },
    },
  };
  const patina = [
    {
      file: "src/EarlyRef.vue",
      ruleId: "script/no-ref-as-operand",
      severity: 1,
      ...span(2, 16, 2, 23),
      message: "Ref 'palette' must be accessed through .value.",
    },
  ];
  const current = compareCurrent(
    "fixture",
    cwd,
    map,
    patina,
    [],
    currentContext(map, ["src/EarlyRef.vue"]),
  );
  const compatible = compareCompatible({
    projectId: "fixture",
    cwd,
    ruleMap: map,
    patinaFindings: patina,
    eslintResults: [],
  });
  for (const result of [current, compatible]) {
    assert.equal(result.summary.falsePositiveCount, 0);
    assert.equal(result.summary.intentionalDivergenceCount, 1);
  }
});

test("whitespace mustaches and imported Slot anchors are classified on both reporters", () => {
  const directory = `${cwd}/src`;
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(
    `${directory}/Slots.vue`,
    `<template>\n  {{ ' ' }}\n  <Slot>\n    <slot />\n  </Slot>\n</template>\n`,
  );
  const map = {
    upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
    entries: {
      "vue/no-useless-mustaches": {
        status: "mapped",
        patinaRule: "vue/no-useless-mustaches",
      },
      "vue/require-explicit-slots": {
        status: "mapped",
        patinaRule: "script/require-explicit-slots",
      },
    },
  };
  const patina = [
    {
      file: "src/Slots.vue",
      ruleId: "script/require-explicit-slots",
      severity: 1,
      ...span(4, 5, 4, 13),
      message: "Slot 'default' is rendered but not declared.",
    },
  ];
  const baseline = [
    eslintResult("src/Slots.vue", [
      {
        ruleId: "vue/no-useless-mustaches",
        severity: 1,
        ...span(2, 3, 2, 12),
        message: "Unexpected mustache interpolation with a string literal value.",
      },
      {
        ruleId: "vue/require-explicit-slots",
        severity: 1,
        ...span(3, 3, 5, 10),
        message: "Slots must be explicitly defined.",
      },
      {
        ruleId: "vue/require-explicit-slots",
        severity: 1,
        ...span(4, 5, 4, 13),
        message: "Slots must be explicitly defined.",
      },
    ]),
  ];
  const current = compareCurrent(
    "fixture",
    cwd,
    map,
    patina,
    baseline,
    currentContext(map, ["src/Slots.vue"]),
  );
  const compatible = compareCompatible({
    projectId: "fixture",
    cwd,
    ruleMap: map,
    patinaFindings: patina,
    eslintResults: baseline,
  });
  for (const result of [current, compatible]) {
    assert.equal(result.summary.falsePositiveCount, 0);
    assert.equal(result.summary.falseNegativeCount, 0);
    assert.equal(result.summary.intentionalDivergenceCount, 2);
    assert.equal(result.summary.ruleLocationDivergenceCount, 0);
  }
});

test("formatter-safe quotes and resolved props objects are intentional on both reporters", () => {
  const directory = `${cwd}/src`;
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(
    `${directory}/Calendar.vue`,
    `<template><x-input :title='\`<span style="\${style}">hello</span>\`' /></template>\n<script>\nconst Props = {\n  placeholder: String\n}\nexport default { props: Props }\n</script>\n`,
  );
  const map = {
    upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
    entries: {
      "vue/html-quotes": { status: "mapped", patinaRule: "vue/html-quotes" },
      "vue/require-default-prop": {
        status: "mapped",
        patinaRule: "script/require-default-prop",
      },
    },
  };
  const patina = [
    {
      file: "src/Calendar.vue",
      ruleId: "script/require-default-prop",
      severity: 1,
      ...span(4, 3, 4, 22),
      message: "Prop 'placeholder' requires a default value.",
    },
  ];
  const baseline = [
    eslintResult("src/Calendar.vue", [
      {
        ruleId: "vue/html-quotes",
        severity: 1,
        ...span(1, 27, 1, 66),
        message: "Expected to be enclosed by double quotes.",
      },
    ]),
  ];
  const current = compareCurrent(
    "fixture",
    cwd,
    map,
    patina,
    baseline,
    currentContext(map, ["src/Calendar.vue"]),
  );
  const compatible = compareCompatible({
    projectId: "fixture",
    cwd,
    ruleMap: map,
    patinaFindings: patina,
    eslintResults: baseline,
  });
  for (const result of [current, compatible]) {
    assert.equal(result.summary.falsePositiveCount, 0);
    assert.equal(result.summary.falseNegativeCount, 0);
    assert.equal(result.summary.intentionalDivergenceCount, 2);
  }
});
