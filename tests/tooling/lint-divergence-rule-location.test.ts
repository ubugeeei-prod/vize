import assert from "node:assert/strict";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { test } from "node:test";

import { compareLintFindings as compareSuccessor } from "../../tools/support/fixtures/lint_divergence/compare.ts";
import { get, list, type Json } from "../../tools/support/fixtures/lint_divergence/value.ts";
import {
  compare,
  cwd,
  currentContext,
  eslintResult,
  ruleMap,
  span,
} from "./_helpers/lint-divergence-fixture.ts";

test("known rule-owned location anchors are classified without hiding both ranges", () => {
  const result = compare(
    [
      {
        file: "src/List.vue",
        ruleId: "vue/require-v-for-key",
        severity: 2,
        ...span(10, 7, 10, 41),
        message:
          "[vize:vue/require-v-for-key] Elements in iteration expect to have 'v-bind:key' directives. Element: <li>",
      },
    ],
    [
      eslintResult("src/List.vue", [
        {
          ruleId: "vue/require-v-for-key",
          severity: 2,
          ...span(10, 4, 10, 76),
          message: "Elements in iteration expect to have 'v-bind:key' directives.",
        },
      ]),
    ],
  );

  assert.deepEqual(result.falsePositives, []);
  assert.deepEqual(result.falseNegatives, []);
  assert.deepEqual(result.ruleLocationDivergences, [
    {
      file: "src/List.vue",
      ruleId: "vue/require-v-for-key",
      upstreamRuleId: "vue/require-v-for-key",
      severity: "error",
      subject: "missing-v-for-key",
      reason:
        "patina reports the v-for directive range while eslint-plugin-vue reports the owning element range.",
      patina: {
        line: 10,
        column: 7,
        endLine: 10,
        endColumn: 41,
        message:
          "[vize:vue/require-v-for-key] Elements in iteration expect to have 'v-bind:key' directives. Element: <li>",
      },
      baseline: {
        line: 10,
        column: 4,
        endLine: 10,
        endColumn: 76,
        message: "Elements in iteration expect to have 'v-bind:key' directives.",
      },
    },
  ]);
  assert.equal(result.summary.ruleLocationDivergenceCount, 1);
  assert.equal(result.summary.falsePositiveRatio, 0);
  assert.equal(result.summary.falseNegativeRatio, 0);
});

test("real-project rule anchors retain complete packets without creating false parity gaps", () => {
  const result = compare(
    [
      {
        file: "src/Card.vue",
        ruleId: "vue/require-component-registration",
        severity: 1,
        ...span(20, 6, 20, 17),
        message:
          "[vize:vue/require-component-registration] Component is used but not explicitly imported",
      },
      {
        file: "src/Card.vue",
        ruleId: "vue/require-prop-comment",
        severity: 1,
        ...span(3, 3, 3, 8),
        message:
          '[vize:vue/require-prop-comment] The "title" property should have a JSDoc comment.',
      },
      {
        file: "src/Card.vue",
        ruleId: "vue/quote-props",
        severity: 1,
        ...span(12, 18, 12, 25),
        message: "[vize:vue/quote-props] Unnecessarily quoted object property",
      },
    ],
    [
      eslintResult("src/Card.vue", [
        {
          ruleId: "vue/no-undef-components",
          severity: 1,
          ...span(20, 5, 20, 31),
          message: "The '<MissingCard>' component has been used, but not defined.",
        },
        {
          ruleId: "vue/require-prop-comment",
          severity: 1,
          ...span(3, 3, 3, 22),
          message: 'The "title" property should have a JSDoc comment.',
        },
        {
          ruleId: "vue/quote-props",
          severity: 1,
          ...span(12, 18, 12, 34),
          message: "Unnecessarily quoted property 'label' found.",
        },
      ]),
    ],
  );

  assert.deepEqual(result.falsePositives, []);
  assert.deepEqual(result.falseNegatives, []);
  assert.deepEqual(
    result.ruleLocationDivergences
      .map(({ ruleId, subject, patina, baseline }) => ({
        ruleId,
        subject,
        patina,
        baseline,
      }))
      .sort((left, right) => left.patina.line - right.patina.line),
    [
      {
        ruleId: "vue/require-prop-comment",
        subject: "prop:title",
        patina: {
          ...span(3, 3, 3, 8),
          message:
            '[vize:vue/require-prop-comment] The "title" property should have a JSDoc comment.',
        },
        baseline: {
          ...span(3, 3, 3, 22),
          message: 'The "title" property should have a JSDoc comment.',
        },
      },
      {
        ruleId: "vue/quote-props",
        subject: "property:12:18",
        patina: {
          ...span(12, 18, 12, 25),
          message: "[vize:vue/quote-props] Unnecessarily quoted object property",
        },
        baseline: {
          ...span(12, 18, 12, 34),
          message: "Unnecessarily quoted property 'label' found.",
        },
      },
      {
        ruleId: "vue/require-component-registration",
        subject: "component-tag:20:6",
        patina: {
          ...span(20, 6, 20, 17),
          message:
            "[vize:vue/require-component-registration] Component is used but not explicitly imported",
        },
        baseline: {
          ...span(20, 5, 20, 31),
          message: "The '<MissingCard>' component has been used, but not defined.",
        },
      },
    ],
  );
  assert.equal(result.summary.ruleLocationDivergenceCount, 3);
});

test("prop comment anchors follow the named prop across type and destructuring sites", () => {
  const file = "src/CodeInput.vue",
    path = `${cwd}/${file}`;
  mkdirSync(`${cwd}/src`, { recursive: true });
  writeFileSync(
    path,
    "const {\n  modelValue,\n} = defineProps<Props>()\ntype Props = {\n  modelValue: string\n}\n",
  );
  try {
    const patina = [
        {
          file,
          ruleId: "vue/require-prop-comment",
          severity: 1,
          ...span(5, 3, 5, 13),
          message:
            '[vize:vue/require-prop-comment] The "modelValue" property should have a JSDoc comment.',
        },
      ],
      baseline = [
        eslintResult(file, [
          {
            ruleId: "vue/require-prop-comment",
            severity: 1,
            ...span(2, 3, 2, 14),
            message: 'The "modelValue" property should have a JSDoc comment.',
          },
        ]),
      ],
      result = compare(patina, baseline),
      successor = compareSuccessor(
        "fixture",
        cwd,
        ruleMap as Json,
        patina,
        baseline as unknown as Json[],
        currentContext(ruleMap, [file]),
      );

    assert.deepEqual(result.falsePositives, []);
    assert.deepEqual(result.falseNegatives, []);
    assert.equal(result.ruleLocationDivergences[0].subject, "prop:modelValue");
    assert.equal(list(successor.falsePositives).length, 0);
    assert.equal(list(successor.falseNegatives).length, 0);
    assert.equal(
      get(list(successor.ruleLocationDivergences)[0] as Json, "subject"),
      "prop:modelValue",
    );
  } finally {
    rmSync(path);
  }
});

test("defineProps links require a direct type member and the owning destructure", () => {
  const file = "src/CodeInput.vue",
    path = `${cwd}/${file}`,
    message = 'The "title" property should have a JSDoc comment.',
    controls = [
      {
        source:
          "const {\n  title,\n} = defineProps<Props>()\ntype Props = {\n  title: string\n  config: {\n    title: string\n  }\n}\n",
        patina: span(7, 5, 7, 10),
        baseline: span(2, 3, 2, 8),
      },
      {
        source:
          "const { title } = otherObject\nconst { other } = defineProps<Props>()\ntype Props = {\n  title: string\n  other: string\n}\n",
        patina: span(4, 3, 4, 8),
        baseline: span(1, 9, 1, 14),
      },
    ];
  mkdirSync(`${cwd}/src`, { recursive: true });
  try {
    for (const control of controls) {
      writeFileSync(path, control.source);
      const patina = [
          {
            file,
            ruleId: "vue/require-prop-comment",
            severity: 1,
            ...control.patina,
            message: `[vize:vue/require-prop-comment] ${message}`,
          },
        ],
        baseline = [
          eslintResult(file, [
            {
              ruleId: "vue/require-prop-comment",
              severity: 1,
              ...control.baseline,
              message,
            },
          ]),
        ],
        result = compare(patina, baseline),
        successor = compareSuccessor(
          "fixture",
          cwd,
          ruleMap as Json,
          patina,
          baseline as unknown as Json[],
          currentContext(ruleMap, [file]),
        );
      assert.equal(result.ruleLocationDivergences.length, 0);
      assert.equal(result.falsePositives.length, 1);
      assert.equal(result.falseNegatives.length, 1);
      assert.equal(list(successor.ruleLocationDivergences).length, 0);
      assert.equal(list(successor.falsePositives).length, 1);
      assert.equal(list(successor.falseNegatives).length, 1);
    }
  } finally {
    rmSync(path);
  }
});

test("same-named prop comments in distinct contracts stay unpaired", () => {
  const message = 'The "title" property should have a JSDoc comment.',
    result = compare(
      [
        {
          file: "src/Cards.vue",
          ruleId: "vue/require-prop-comment",
          severity: 1,
          ...span(4, 3, 4, 8),
          message: `[vize:vue/require-prop-comment] ${message}`,
        },
        {
          file: "src/Cards.vue",
          ruleId: "vue/require-prop-comment",
          severity: 1,
          ...span(14, 3, 14, 8),
          message: `[vize:vue/require-prop-comment] ${message}`,
        },
      ],
      [
        eslintResult("src/Cards.vue", [
          { ruleId: "vue/require-prop-comment", severity: 1, ...span(24, 3, 24, 22), message },
          { ruleId: "vue/require-prop-comment", severity: 1, ...span(34, 3, 34, 22), message },
        ]),
      ],
    );
  assert.equal(result.ruleLocationDivergences.length, 0);
  assert.equal(result.falsePositives.length, 2);
  assert.equal(result.falseNegatives.length, 2);
});

test("a singleton same-named prop outside its declaration stays unpaired", () => {
  const message = 'The "title" property should have a JSDoc comment.',
    result = compare(
      [
        {
          file: "src/Card.vue",
          ruleId: "vue/require-prop-comment",
          severity: 1,
          ...span(4, 3, 4, 8),
          message: `[vize:vue/require-prop-comment] ${message}`,
        },
      ],
      [
        eslintResult("src/Card.vue", [
          { ruleId: "vue/require-prop-comment", severity: 1, ...span(14, 3, 14, 22), message },
        ]),
      ],
    );
  assert.equal(result.ruleLocationDivergences.length, 0);
  assert.equal(result.falsePositives.length, 1);
  assert.equal(result.falseNegatives.length, 1);
});

test("prop comment uniqueness is scoped to each file", () => {
  const message = 'The "title" property should have a JSDoc comment.',
    findings = ["A", "B"].map((name) => ({
      file: `src/${name}.vue`,
      ruleId: "vue/require-prop-comment",
      severity: 1,
      ...span(4, 3, 4, 8),
      message: `[vize:vue/require-prop-comment] ${message}`,
    })),
    result = compare(
      findings,
      ["A", "B"].map((name) =>
        eslintResult(`src/${name}.vue`, [
          { ruleId: "vue/require-prop-comment", severity: 1, ...span(4, 3, 4, 22), message },
        ]),
      ),
    );
  assert.equal(result.ruleLocationDivergences.length, 2);
  assert.deepEqual(result.falsePositives, []);
  assert.deepEqual(result.falseNegatives, []);
});

test("nested quote-props packets require the same property-key anchor", () => {
  const result = compare(
    [
      {
        file: "src/Nested.vue",
        ruleId: "vue/quote-props",
        severity: 1,
        ...span(2, 16, 2, 30),
        message: "[vize:vue/quote-props] Unnecessarily quoted object property",
      },
    ],
    [
      eslintResult("src/Nested.vue", [
        {
          ruleId: "vue/quote-props",
          severity: 1,
          ...span(2, 3, 2, 35),
          message: "Unnecessarily quoted property 'outer' found.",
        },
      ]),
    ],
  );
  assert.equal(result.ruleLocationDivergences.length, 0);
  assert.equal(result.falsePositives.length, 1);
  assert.equal(result.falseNegatives.length, 1);
});

test("unused component location anchors are paired by component identity", () => {
  const result = compare(
    [
      {
        file: "src/Uploader.vue",
        ruleId: "vue/no-unused-components",
        severity: 2,
        ...span(1, 11, 2, 12),
        message:
          "[vize:vue/no-unused-components] Component 'UploaderFile' is registered but never used in template",
      },
      {
        file: "src/Uploader.vue",
        ruleId: "vue/no-unused-components",
        severity: 2,
        ...span(1, 11, 2, 12),
        message:
          "[vize:vue/no-unused-components] Component 'UploaderFiles' is registered but never used in template",
      },
    ],
    [
      eslintResult("src/Uploader.vue", [
        {
          ruleId: "vue/no-unused-components",
          severity: 2,
          ...span(140, 7, 140, 20),
          message: 'The "UploaderFiles" component has been registered but not used.',
        },
        {
          ruleId: "vue/no-unused-components",
          severity: 2,
          ...span(141, 7, 141, 19),
          message: 'The "UploaderFile" component has been registered but not used.',
        },
      ]),
    ],
  );

  assert.deepEqual(result.falsePositives, []);
  assert.deepEqual(result.falseNegatives, []);
  assert.deepEqual(
    result.ruleLocationDivergences.map((entry) => entry.subject),
    ["component:UploaderFile", "component:UploaderFiles"],
  );
  assert.equal(result.summary.ruleLocationDivergenceCount, 2);
});

test("location pairs take precedence over one-sided intentional classification", () => {
  const filename = "src/DefineProps.vue";
  const path = `${cwd}/${filename}`;
  mkdirSync(`${cwd}/src`, { recursive: true });
  writeFileSync(
    path,
    '<script setup lang="ts">\nconst props = defineProps<{ title: string }>()\n</script>\n',
  );
  try {
    const result = compare(
      [
        {
          file: filename,
          ruleId: "script/define-props-destructuring",
          severity: 1,
          ...span(2, 7, 2, 12),
          message:
            "[vize:script/define-props-destructuring] Prefer destructuring the return value of defineProps().",
        },
      ],
      [
        eslintResult(filename, [
          {
            ruleId: "vue/define-props-destructuring",
            severity: 1,
            ...span(2, 15, 2, 47),
            message: "Prefer destructuring from `defineProps` directly.",
          },
        ]),
      ],
    );

    assert.deepEqual(result.falsePositives, []);
    assert.deepEqual(result.falseNegatives, []);
    assert.deepEqual(result.intentionalDivergences, []);
    assert.equal(result.ruleLocationDivergences.length, 1);
    assert.equal(result.ruleLocationDivergences[0].subject, "define-props:2:2");
  } finally {
    rmSync(path);
  }
});
