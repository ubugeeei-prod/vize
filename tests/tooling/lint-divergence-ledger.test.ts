import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import { compareLintFindings as compareCompatibleLintFindings } from "../../tools/support/compat/fixtures/lint-divergence.ts";
import { compareLintFindings as compareCurrentLintFindingsRaw } from "../../tools/support/fixtures/lint_divergence/compare.ts";
import {
  compare,
  currentContext,
  cwd,
  eslintResult,
  span,
} from "./_helpers/lint-divergence-fixture.ts";

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

test("a ledger entry cancels exactly one false positive against one false negative", () => {
  const patina = [
    {
      file: "src/App.vue",
      ruleId: "vue/no-v-html",
      severity: 2,
      ...span(4, 6, 4, 18),
      message: "v-html",
    },
  ];
  const baseline = [
    eslintResult("src/App.vue", [
      {
        ruleId: "vue/no-v-html",
        severity: 2,
        ...span(4, 6, 4, 30),
        message: "`v-html` directive can lead to XSS attack.",
      },
    ]),
  ];
  const reason =
    "patina highlights the directive itself while eslint-plugin-vue highlights the whole attribute.";
  const ledger = [
    {
      project: "fixture",
      file: "src/App.vue",
      ruleId: "vue/no-v-html",
      patina: { severity: "error", ...span(4, 6, 4, 18) },
      baseline: { severity: "error", ...span(4, 6, 4, 30) },
      issue: 3223,
      reason,
    },
  ];

  const undocumented = compare(patina, baseline);
  assert.equal(undocumented.summary.falsePositiveCount, 1);
  assert.equal(undocumented.summary.falseNegativeCount, 1);

  const documented = compare(patina, baseline, ledger);
  assert.deepEqual(documented.falsePositives, []);
  assert.deepEqual(documented.falseNegatives, []);
  assert.deepEqual(documented.documentedDivergences, [
    {
      file: "src/App.vue",
      ruleId: "vue/no-v-html",
      patina: { severity: "error", line: 4, column: 6, endLine: 4, endColumn: 18 },
      baseline: { severity: "error", line: 4, column: 6, endLine: 4, endColumn: 30 },
      issue: 3223,
      reason,
    },
  ]);
  assert.equal(documented.summary.documentedDivergenceCount, 1);
});

test("the stronger direct defineProps policy is intentional on both reporters", () => {
  const directory = `${cwd}/src`;
  fs.mkdirSync(directory, { recursive: true });
  const direct = '<script setup lang="ts">\ndefineProps<{ label: string }>()\n</script>\n';
  const wrapped =
    '<script setup lang="ts">\nwithDefaults(defineProps<{ label?: string }>(), { label: "" })\n</script>\n';
  fs.writeFileSync(`${directory}/BareProps.vue`, direct);
  fs.writeFileSync(`${directory}/WrappedProps.vue`, wrapped);
  fs.writeFileSync(
    `${directory}/ModuleProps.vue`,
    `<script lang="ts">\nexport interface Props { label?: string }\n</script>\n<script setup lang="ts">\nconst props = defineProps<Props>()\n</script>\n`,
  );
  const map = {
    upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
    entries: {
      "vue/define-props-destructuring": {
        status: "mapped",
        patinaRule: "script/define-props-destructuring",
      },
    },
  };
  const findings = [
    {
      file: "src/BareProps.vue",
      ruleId: "script/define-props-destructuring",
      severity: 1,
      ...span(2, 1, 2, 35),
      message: "Prefer destructuring the return value of defineProps().",
    },
    {
      file: "src/WrappedProps.vue",
      ruleId: "script/define-props-destructuring",
      severity: 1,
      ...span(2, 14, 2, 49),
      message: "Prefer destructuring the return value of defineProps().",
    },
    {
      file: "src/ModuleProps.vue",
      ruleId: "script/define-props-destructuring",
      severity: 1,
      ...span(5, 7, 5, 12),
      message: "Prefer destructuring the return value of defineProps().",
    },
  ];
  const current = compareCurrent(
    "fixture",
    cwd,
    map,
    findings,
    [],
    currentContext(map, ["src/BareProps.vue", "src/WrappedProps.vue", "src/ModuleProps.vue"]),
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
    assert.equal(result.summary.intentionalDivergenceCount, 3);
    assert.deepEqual(result.intentionalDivergences.map((finding) => finding.file).sort(), [
      "src/BareProps.vue",
      "src/ModuleProps.vue",
      "src/WrappedProps.vue",
    ]);
  }
});

test("stable defineProps and self-closing anchor differences are not false findings", () => {
  const directory = `${cwd}/src`;
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(
    `${directory}/LocationAnchors.vue`,
    `<script setup lang="ts">\nconst props =\n  defineProps<{\n    label: string\n  }>()\n</script>\n<template>\n  <p>label</p>\n</template>\n`,
  );
  const map = {
    upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
    entries: {
      "vue/define-props-destructuring": {
        status: "mapped",
        patinaRule: "script/define-props-destructuring",
      },
      "vue/html-self-closing": {
        status: "mapped",
        patinaRule: "vue/html-self-closing",
      },
    },
  };
  const findings = [
    {
      file: "src/LocationAnchors.vue",
      ruleId: "script/define-props-destructuring",
      severity: 1,
      ...span(2, 7, 2, 12),
      message: "Prefer destructuring the return value of defineProps().",
    },
    {
      file: "src/LocationAnchors.vue",
      ruleId: "vue/html-self-closing",
      severity: 1,
      ...span(7, 3, 7, 15),
      message: "Empty element should be self-closing.",
    },
  ];
  const baseline = [
    eslintResult("src/LocationAnchors.vue", [
      {
        ruleId: "vue/define-props-destructuring",
        severity: 1,
        ...span(3, 3, 5, 6),
        message: "Prefer destructuring from defineProps directly.",
      },
      {
        ruleId: "vue/html-self-closing",
        severity: 1,
        ...span(7, 16, 7, 20),
        message: "Require self-closing on HTML elements.",
      },
    ]),
  ];
  const current = compareCurrent(
    "fixture",
    cwd,
    map,
    findings,
    baseline,
    currentContext(map, ["src/LocationAnchors.vue"]),
  );
  const compatible = compareCompatible({
    projectId: "fixture",
    cwd,
    ruleMap: map,
    patinaFindings: findings,
    eslintResults: baseline,
  });
  for (const result of [current, compatible]) {
    assert.equal(result.summary.falsePositiveCount, 0, JSON.stringify(result.falsePositives));
    assert.equal(result.summary.falseNegativeCount, 0, JSON.stringify(result.falseNegatives));
    assert.equal(result.summary.ruleLocationDivergenceCount, 2);
  }
});

test("runtime-sensitive ordering, UTS, and proven unused registrations stay intentional", () => {
  const directory = `${cwd}/src`;
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(
    `${directory}/MergeOrder.vue`,
    `<template><button @click="save" v-bind="attrs">Save</button></template>\n`,
  );
  fs.writeFileSync(
    `${directory}/Native.vue`,
    `<template><view></view></template>\n<script setup lang="uts">\nconst props = defineProps({ label: String })\n</script>\n`,
  );
  fs.writeFileSync(
    `${directory}/Unused.vue`,
    `<template><div><!-- <SfButton /> --></div></template>\n<script>\nimport { SfButton } from "ui"\nexport default { components: { SfButton } }\n</script>\n`,
  );
  const map = {
    upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
    entries: {
      "vue/attributes-order": {
        status: "mapped",
        patinaRule: "vue/attribute-order",
      },
      "vue/define-props-destructuring": {
        status: "mapped",
        patinaRule: "script/define-props-destructuring",
      },
      "vue/html-self-closing": {
        status: "mapped",
        patinaRule: "vue/html-self-closing",
      },
      "vue/no-unused-components": {
        status: "mapped",
        patinaRule: "vue/no-unused-components",
      },
    },
  };
  const findings = [
    {
      file: "src/Unused.vue",
      ruleId: "vue/no-unused-components",
      severity: 1,
      ...span(4, 32, 4, 40),
      message: "Component 'SfButton' is registered but never used in template",
    },
  ];
  const baseline = [
    eslintResult("src/MergeOrder.vue", [
      {
        ruleId: "vue/attributes-order",
        severity: 1,
        ...span(1, 32, 1, 46),
        message: 'Attribute "v-bind" should go before "@click".',
      },
    ]),
    eslintResult("src/Native.vue", [
      {
        ruleId: "vue/define-props-destructuring",
        severity: 1,
        ...span(3, 15, 3, 53),
        message: "Prefer destructuring from defineProps directly.",
      },
      {
        ruleId: "vue/html-self-closing",
        severity: 1,
        ...span(1, 17, 1, 24),
        message: "Require self-closing on Vue.js custom components.",
      },
    ]),
  ];
  const current = compareCurrent(
    "fixture",
    cwd,
    map,
    findings,
    baseline,
    currentContext(map, ["src/MergeOrder.vue", "src/Native.vue", "src/Unused.vue"]),
  );
  const compatible = compareCompatible({
    projectId: "fixture",
    cwd,
    ruleMap: map,
    patinaFindings: findings,
    eslintResults: baseline,
  });
  for (const result of [current, compatible]) {
    assert.equal(result.summary.falsePositiveCount, 0, JSON.stringify(result.falsePositives));
    assert.equal(result.summary.falseNegativeCount, 0, JSON.stringify(result.falseNegatives));
    assert.equal(result.summary.intentionalDivergenceCount, 4);
  }
});

test("upstream rule ids classify second-script shifts and resolved default spreads", () => {
  const directory = `${cwd}/src`;
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(
    `${directory}/TwoScripts.vue`,
    `<script lang="ts">\nexport const defaults = { label: "" }\nexport interface Props { label?: string }\n</script>\n<script setup lang="ts">\nconst props = withDefaults(defineProps<Props>(), { ...defaults })\nconst element = ref(null)\n</script>\n<template><div ref="element" /></template>\n`,
  );
  const map = {
    upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
    entries: {
      "vue/define-props-destructuring": {
        status: "mapped",
        patinaRule: "script/define-props-destructuring",
      },
      "vue/prefer-use-template-ref": {
        status: "mapped",
        patinaRule: "script/prefer-use-template-ref",
      },
      "vue/require-default-prop": {
        status: "mapped",
        patinaRule: "script/require-default-prop",
      },
    },
  };
  const baseline = [
    eslintResult("src/TwoScripts.vue", [
      {
        ruleId: "vue/define-props-destructuring",
        severity: 1,
        ...span(2, 7, 2, 12),
        message: "Prefer destructuring from defineProps.",
      },
      {
        ruleId: "vue/prefer-use-template-ref",
        severity: 1,
        ...span(3, 7, 3, 14),
        message: "Use useTemplateRef() instead of ref().",
      },
      {
        ruleId: "vue/require-default-prop",
        severity: 2,
        ...span(3, 26, 3, 40),
        message: "Prop 'label' requires default value to be set.",
      },
    ]),
  ];
  const current = compareCurrent(
    "fixture",
    cwd,
    map,
    [],
    baseline,
    currentContext(map, ["src/TwoScripts.vue"]),
  );
  const compatible = compareCompatible({
    projectId: "fixture",
    cwd,
    ruleMap: map,
    patinaFindings: [],
    eslintResults: baseline,
  });
  for (const result of [current, compatible]) {
    assert.equal(result.summary.falseNegativeCount, 0, JSON.stringify(result.falseNegatives));
    assert.equal(result.summary.intentionalDivergenceCount, 3);
  }
});
