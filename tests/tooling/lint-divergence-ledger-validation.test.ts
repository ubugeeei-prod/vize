import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import { compareLintFindings as compareCompatibleLintFindings } from "../../tools/support/compat/fixtures/lint-divergence.ts";
import { compareLintFindings as compareCurrentLintFindingsRaw } from "../../tools/support/fixtures/lint_divergence/compare.ts";
import { readRuleMap } from "../../tools/support/compat/fixtures/patina-rule-map.ts";
import {
  compare,
  cwd,
  eslintResult,
  ledgerPath,
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

test("a ledger entry only cancels a divergence that is actually present", () => {
  // The ledger describes a divergence in a file neither linter reported on, so
  // it must stay unpaired rather than silently absorbing something else.
  const documented = compare(
    [],
    [],
    [
      {
        project: "fixture",
        file: "src/Other.vue",
        ruleId: "vue/no-v-html",
        patina: { severity: "error", ...span(1, 1, 1, 4) },
        baseline: { severity: "error", ...span(2, 1, 2, 4) },
        issue: 3223,
        reason:
          "patina reports the directive name while eslint-plugin-vue reports the attribute value node.",
      },
    ],
  );

  assert.deepEqual(documented.documentedDivergences, []);
  assert.equal(documented.summary.documentedDivergenceCount, 0);
});

test("a ledger entry that records no actual difference is rejected", () => {
  assert.throws(
    () =>
      compare(
        [],
        [],
        [
          {
            project: "fixture",
            file: "src/App.vue",
            ruleId: "vue/no-v-html",
            patina: { severity: "error", ...span(1, 1, 1, 2) },
            baseline: { severity: "error", ...span(1, 1, 1, 2) },
            issue: 3223,
            reason:
              "this rationale is long enough to pass the length gate but describes no divergence.",
          },
        ],
      ),
    /must record a difference between the two linters/,
  );
});

test("a ledger entry without a written rationale is rejected", () => {
  assert.throws(
    () =>
      compare(
        [],
        [],
        [
          {
            project: "fixture",
            file: "src/App.vue",
            ruleId: "vue/no-v-html",
            patina: { severity: "error", ...span(1, 1, 1, 2) },
            baseline: { severity: "error", ...span(2, 1, 2, 2) },
            issue: 3223,
            reason: "different",
          },
        ],
      ),
    /reason must explain why the divergence is expected/,
  );
});

test("a ledger entry without a tracking issue is rejected", () => {
  assert.throws(
    () =>
      compare(
        [],
        [],
        [
          {
            project: "fixture",
            file: "src/App.vue",
            ruleId: "vue/no-v-html",
            patina: { severity: "error", ...span(1, 1, 1, 2) },
            baseline: { severity: "error", ...span(2, 1, 2, 2) },
            reason:
              "patina reports the directive name while eslint-plugin-vue reports the attribute value node.",
          },
        ],
      ),
    /issue must be the tracking issue number/,
  );
});

test("recovered templates remain comparable while conservative legacy surfaces are intentional", () => {
  const directory = `${cwd}/src`;
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(
    `${directory}/Emit.vue`,
    `<template><button @click="$emit('save')" /></template>\n<script>\nexport default { emits: ['save'] }\n</script>\n`,
  );
  fs.writeFileSync(
    `${directory}/Table.vue`,
    `<template><table><tbody ref="list"><transition-group><slot name="row" /></transition-group></tbody></table></template>\n`,
  );
  fs.writeFileSync(
    `${directory}/Events.vue`,
    `<script>\nexport default { methods: { show(options) {\nextend(this, options)\nthis.$on('save', () => {})\n} } }\n</script>\n`,
  );
  fs.writeFileSync(
    `${directory}/Legacy.vue`,
    `<script setup>\nconst state = $ref({ count: 0 }); consume($(state))\n</script>\n`,
  );
  const map = {
    upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
    entries: {
      "vue/no-unused-emit-declarations": {
        status: "mapped",
        patinaRule: "script/no-unused-emit-declarations",
      },
      "vue/require-explicit-slots": {
        status: "mapped",
        patinaRule: "script/require-explicit-slots",
      },
      "vue/no-unused-refs": { status: "mapped", patinaRule: "vue/no-unused-refs" },
      "vue/no-deprecated-events-api": {
        status: "mapped",
        patinaRule: "script/no-deprecated-events-api",
      },
      "vue/no-ref-object-reactivity-loss": {
        status: "mapped",
        patinaRule: "script/no-ref-object-reactivity-loss",
      },
    },
  };
  const baseline = [
    eslintResult("src/Emit.vue", [
      {
        ruleId: "vue/no-unused-emit-declarations",
        severity: 1,
        ...span(3, 27, 3, 33),
        message: "`save` is defined as emit but never used.",
      },
    ]),
    eslintResult("src/Table.vue", [
      {
        ruleId: "vue/require-explicit-slots",
        severity: 1,
        ...span(1, 60, 1, 79),
        message: "Slots must be explicitly defined.",
      },
      {
        ruleId: "vue/no-unused-refs",
        severity: 1,
        ...span(1, 29, 1, 39),
        message: "'list' is defined as ref, but never used.",
      },
    ]),
    eslintResult("src/Events.vue", [
      {
        ruleId: "vue/no-deprecated-events-api",
        severity: 2,
        ...span(4, 6, 4, 9),
        message: "The Events API is deprecated.",
      },
    ]),
    eslintResult("src/Legacy.vue", [
      {
        ruleId: "vue/no-ref-object-reactivity-loss",
        severity: 1,
        ...span(2, 48, 2, 53),
        message: "Getting a reactive variable loses reactivity.",
      },
    ]),
  ];
  const recoveredFindings = [
    {
      file: "src/Table.vue",
      ruleId: "script/require-explicit-slots",
      severity: 1,
      ...span(1, 60, 1, 79),
      message: "[vize:script/require-explicit-slots] Slot 'row' is rendered but not declared.",
    },
    {
      file: "src/Table.vue",
      ruleId: "vue/no-unused-refs",
      severity: 1,
      ...span(1, 29, 1, 39),
      message: "[vize:vue/no-unused-refs] Template ref 'list' is never referenced.",
    },
  ];
  const current = compareCurrent("fixture", cwd, map, recoveredFindings, baseline, {});
  const compatible = compareCompatible({
    projectId: "fixture",
    cwd,
    ruleMap: map,
    patinaFindings: recoveredFindings,
    eslintResults: baseline,
  });
  for (const result of [current, compatible]) {
    assert.equal(result.summary.falsePositiveCount, 0);
    assert.equal(result.summary.falseNegativeCount, 0);
    assert.equal(result.summary.sharedCount, 2);
    assert.equal(result.summary.intentionalDivergenceCount, 3);
  }
});

test("sound Patina-only real-project findings are intentional on both reporters", () => {
  const directory = `${cwd}/src`;
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(`${directory}/TemplateOnly.vue`, `<template><main>Settings</main></template>\n`);
  fs.writeFileSync(
    `${directory}/Props.vue`,
    `<script setup lang="ts">\ninterface Props extends Base {\n  label?: string\n}\nwithDefaults(defineProps<Props>(), {})\n</script>\n`,
  );
  fs.writeFileSync(
    `${directory}/Mutation.vue`,
    `<script setup lang="ts">\n(props.item.value as Ref<any>).value = next\n</script>\n`,
  );
  fs.writeFileSync(
    `${directory}/UnusedPug.vue`,
    `<script>\nexport default { components: { IpfsImageViewer } }\n</script>\n<template lang="pug">\nmain\n  token-logo\n</template>\n`,
  );
  const map = {
    upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
    entries: {
      "vue/multi-word-component-names": {
        status: "mapped",
        patinaRule: "vue/multi-word-component-names",
      },
      "vue/no-bare-strings-in-template": {
        status: "mapped",
        patinaRule: "vue/no-bare-strings-in-template",
      },
      "vue/require-default-prop": {
        status: "mapped",
        patinaRule: "script/require-default-prop",
      },
      "vue/no-mutating-props": { status: "mapped", patinaRule: "vue/no-mutating-props" },
      "vue/no-unused-components": {
        status: "mapped",
        patinaRule: "vue/no-unused-components",
      },
    },
  };
  const findings = [
    {
      file: "src/TemplateOnly.vue",
      ruleId: "vue/multi-word-component-names",
      severity: 2,
      ...span(1, 1, 1, 1),
      message: 'Component name "TemplateOnly" should be multi-word.',
    },
    {
      file: "src/TemplateOnly.vue",
      ruleId: "vue/no-bare-strings-in-template",
      severity: 1,
      ...span(1, 17, 1, 25),
      message: "Raw text should be translated.",
    },
    {
      file: "src/Props.vue",
      ruleId: "script/require-default-prop",
      severity: 2,
      ...span(3, 3, 3, 17),
      message: "Prop 'label' requires a default value.",
    },
    {
      file: "src/Mutation.vue",
      ruleId: "vue/no-mutating-props",
      severity: 2,
      ...span(2, 1, 2, 38),
      message: "Unexpected mutation of prop '(props.item.value as Ref<any>).value'.",
    },
    {
      file: "src/UnusedPug.vue",
      ruleId: "vue/no-unused-components",
      severity: 1,
      ...span(2, 32, 2, 47),
      message:
        "[vize:vue/no-unused-components] Component 'IpfsImageViewer' is registered but never used in template",
    },
  ];
  const current = compareCurrent("fixture", cwd, map, findings, [], {
    files: ["src/TemplateOnly.vue", "src/Props.vue", "src/Mutation.vue", "src/UnusedPug.vue"],
  });
  const compatible = compareCompatible({
    projectId: "fixture",
    cwd,
    ruleMap: map,
    patinaFindings: findings,
    eslintResults: [],
  });
  for (const result of [current, compatible]) {
    assert.equal(result.summary.falsePositiveCount, 0);
    assert.equal(result.summary.intentionalDivergenceCount, 5);
  }
});

test("a registered component used by a Pug template cannot be classified as intentional", () => {
  fs.mkdirSync(`${cwd}/src`, { recursive: true });
  fs.writeFileSync(
    `${cwd}/src/UsedPug.vue`,
    `<script>\nexport default { components: { UsedPanel } }\n</script>\n<template lang="pug">\nmain\n  used-panel\n</template>\n`,
  );
  const finding = {
    file: "src/UsedPug.vue",
    ruleId: "vue/no-unused-components",
    severity: 1,
    ...span(2, 32, 2, 41),
    message:
      "[vize:vue/no-unused-components] Component 'UsedPanel' is registered but never used in template",
  };
  const result = compareCompatible({
    projectId: "fixture",
    cwd,
    ruleMap: {
      upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
      entries: {
        "vue/no-unused-components": {
          status: "mapped",
          patinaRule: "vue/no-unused-components",
        },
      },
    },
    patinaFindings: [finding],
    eslintResults: [],
  });
  assert.equal(result.summary.falsePositiveCount, 1);
  assert.equal(result.summary.intentionalDivergenceCount, 0);
});

test("the checked-in divergence ledger validates against the checked-in rule map", () => {
  const ledger = JSON.parse(fs.readFileSync(ledgerPath, "utf8"));
  assert.ok(Array.isArray(ledger), "the ledger must be an array of divergence entries");
  const result = compareCompatible({
    projectId: "unhydrated",
    cwd,
    ruleMap: readRuleMap(),
    patinaFindings: [],
    eslintResults: [],
    documentedDivergences: ledger,
  });

  assert.equal(result.schema, "vize.fixtureLintDivergence");
  assert.equal(result.version, 1);
  assert.equal(result.summary.falsePositiveCount, 0);
  assert.equal(result.summary.falseNegativeCount, 0);
});
