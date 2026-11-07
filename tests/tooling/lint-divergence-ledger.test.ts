import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import { compareLintFindings as compareCompatibleLintFindings } from "../../tools/support/compat/fixtures/lint-divergence.ts";
import { compareLintFindings as compareCurrentLintFindings } from "../../tools/support/fixtures/lint_divergence/compare.ts";
import { readRuleMap } from "../../tools/support/compat/fixtures/patina-rule-map.ts";
import {
  compare,
  cwd,
  eslintResult,
  ledgerPath,
  span,
} from "./_helpers/lint-divergence-fixture.ts";

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
  ];
  const current = compareCurrentLintFindings("fixture", cwd, map, findings, [], {});
  const compatible = compareCompatibleLintFindings({
    projectId: "fixture",
    cwd,
    ruleMap: map,
    patinaFindings: findings,
    eslintResults: [],
  });
  for (const result of [current, compatible]) {
    assert.equal(result.summary.falsePositiveCount, 0);
    assert.equal(result.summary.intentionalDivergenceCount, 2);
    assert.deepEqual(
      result.intentionalDivergences.map((finding) => finding.file),
      ["src/BareProps.vue", "src/WrappedProps.vue"],
    );
  }
});

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
  const current = compareCurrentLintFindings("fixture", cwd, map, findings, [], {});
  const compatible = compareCompatibleLintFindings({
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
  const current = compareCurrentLintFindings("fixture", cwd, map, [], baseline, {});
  const compatible = compareCompatibleLintFindings({
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
  const current = compareCurrentLintFindings("fixture", cwd, map, patina, baseline, {});
  const compatible = compareCompatibleLintFindings({
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
  const current = compareCurrentLintFindings("fixture", cwd, map, patina, [], {});
  const compatible = compareCompatibleLintFindings({
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
  const current = compareCurrentLintFindings("fixture", cwd, map, patina, baseline, {});
  const compatible = compareCompatibleLintFindings({
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
  const current = compareCurrentLintFindings("fixture", cwd, map, patina, baseline, {});
  const compatible = compareCompatibleLintFindings({
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

test("recovered templates and conservative legacy surfaces are intentional on both reporters", () => {
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
  const current = compareCurrentLintFindings("fixture", cwd, map, [], baseline, {});
  const compatible = compareCompatibleLintFindings({
    projectId: "fixture",
    cwd,
    ruleMap: map,
    patinaFindings: [],
    eslintResults: baseline,
  });
  for (const result of [current, compatible]) {
    assert.equal(result.summary.falseNegativeCount, 0);
    assert.equal(result.summary.intentionalDivergenceCount, 5);
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
  ];
  const current = compareCurrentLintFindings("fixture", cwd, map, findings, [], {});
  const compatible = compareCompatibleLintFindings({
    projectId: "fixture",
    cwd,
    ruleMap: map,
    patinaFindings: findings,
    eslintResults: [],
  });
  for (const result of [current, compatible]) {
    assert.equal(result.summary.falsePositiveCount, 0);
    assert.equal(result.summary.intentionalDivergenceCount, 4);
  }
});

test("the checked-in divergence ledger validates against the checked-in rule map", () => {
  const ledger = JSON.parse(fs.readFileSync(ledgerPath, "utf8"));
  assert.ok(Array.isArray(ledger), "the ledger must be an array of divergence entries");
  const result = compareCompatibleLintFindings({
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
