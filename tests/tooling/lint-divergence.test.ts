import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { compareLintFindings } from "../../tools/support/compat/fixtures/lint-divergence.ts";
import { compareLintFindings as compareSuccessor } from "../../tools/support/fixtures/lint_divergence/compare.ts";
import { get, list, type Json } from "../../tools/support/fixtures/lint_divergence/value.ts";
import { compare, cwd, eslintResult, ruleMap, span } from "./_helpers/lint-divergence-fixture.ts";

test("the divergence ledger keys on the full location tuple, not on counts", () => {
  // Both linters report exactly two findings for the same rule in the same
  // file. Only one of them is at the same range, so "2 == 2" must not read as
  // parity.
  const result = compare(
    [
      {
        file: "src/App.vue",
        ruleId: "vue/no-v-html",
        severity: 2,
        ...span(3, 8, 3, 20),
        message: "v-html",
      },
      {
        file: "src/App.vue",
        ruleId: "vue/no-v-html",
        severity: 2,
        ...span(9, 4, 9, 16),
        message: "v-html",
      },
    ],
    [
      eslintResult("src/App.vue", [
        {
          ruleId: "vue/no-v-html",
          severity: 2,
          ...span(3, 8, 3, 20),
          message: "`v-html` directive can lead to XSS attack.",
        },
        {
          ruleId: "vue/no-v-html",
          severity: 2,
          ...span(11, 4, 11, 16),
          message: "`v-html` directive can lead to XSS attack.",
        },
      ]),
    ],
  );

  assert.equal(result.schema, "vize.fixtureLintDivergence");
  assert.equal(result.version, 1);
  assert.equal(result.project, "fixture");
  assert.deepEqual(result.upstream, { package: "eslint-plugin-vue", version: "10.9.2" });
  assert.deepEqual(result.shared, [
    {
      file: "src/App.vue",
      ruleId: "vue/no-v-html",
      upstreamRuleId: "vue/no-v-html",
      severity: "error",
      line: 3,
      column: 8,
      endLine: 3,
      endColumn: 20,
      patinaMessage: "v-html",
      baselineMessage: "`v-html` directive can lead to XSS attack.",
    },
  ]);
  assert.deepEqual(result.messageDifferences, result.shared);
  assert.deepEqual(result.falsePositives, [
    {
      file: "src/App.vue",
      ruleId: "vue/no-v-html",
      severity: "error",
      line: 9,
      column: 4,
      endLine: 9,
      endColumn: 16,
      message: "v-html",
    },
  ]);
  assert.deepEqual(result.falseNegatives, [
    {
      file: "src/App.vue",
      ruleId: "vue/no-v-html",
      upstreamRuleId: "vue/no-v-html",
      severity: "error",
      line: 11,
      column: 4,
      endLine: 11,
      endColumn: 16,
      message: "`v-html` directive can lead to XSS attack.",
    },
  ]);
  assert.deepEqual(result.summary, {
    patinaFindingCount: 2,
    baselineFindingCount: 2,
    comparableBaselineCount: 2,
    sharedCount: 1,
    messageDifferenceCount: 1,
    documentedDivergenceCount: 0,
    ruleLocationDivergenceCount: 0,
    falsePositiveCount: 1,
    falseNegativeCount: 1,
    unimplementedCount: 0,
    intentionalDivergenceCount: 0,
    patinaOnlyRuleFindingCount: 0,
    baselineParseErrorCount: 0,
    baselineExcludedNonVueCount: 0,
    baselineInvalidRangeCount: 0,
    falsePositiveRatio: 0.5,
    falseNegativeRatio: 0.5,
  });
});

test("an aliased rule is compared under its patina name", () => {
  const result = compare(
    [
      {
        file: "src/App.vue",
        ruleId: "vue/attribute-order",
        severity: 1,
        ...span(2, 3, 2, 9),
        message: "attribute order",
      },
    ],
    [
      eslintResult("src/App.vue", [
        {
          ruleId: "vue/attributes-order",
          severity: 1,
          ...span(2, 3, 2, 9),
          message: 'Attribute "id" should go before "class".',
        },
      ]),
    ],
  );

  assert.deepEqual(result.falsePositives, []);
  assert.deepEqual(result.falseNegatives, []);
  assert.equal(result.summary.sharedCount, 1);
  assert.equal(result.shared[0].ruleId, "vue/attribute-order");
  assert.equal(result.shared[0].upstreamRuleId, "vue/attributes-order");
});

test("identical duplicate baseline findings do not become false negatives", () => {
  const baseline = {
    ruleId: "vue/no-v-html",
    severity: 2,
    ...span(3, 8, 3, 20),
    message: "`v-html` directive can lead to XSS attack.",
  };
  const patina = [
      {
        file: "src/App.vue",
        ruleId: "vue/no-v-html",
        severity: 2,
        ...span(3, 8, 3, 20),
        message: "v-html",
      },
    ],
    eslint = [eslintResult("src/App.vue", [baseline, { ...baseline }])],
    result = compare(patina, eslint),
    successor = compareSuccessor(
      "fixture",
      cwd,
      ruleMap,
      patina,
      eslint as Parameters<typeof compareSuccessor>[4],
      { files: ["src/App.vue"] },
    ),
    successorSummary = successor.summary as Record<string, unknown>;

  assert.equal(result.summary.baselineFindingCount, 1);
  assert.equal(successorSummary.baselineFindingCount, 1);
  assert.equal(result.summary.comparableBaselineCount, 1);
  assert.equal(result.summary.sharedCount, 1);
  assert.equal(result.summary.falseNegativeCount, 0);
  assert.deepEqual(result.falseNegatives, []);
});

test("intentional findings do not dilute the false-positive ratio", () => {
  const file = "src/Intentional.vue",
    filename = path.join(cwd, file),
    map = {
      upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
      entries: {
        "vue/no-setup-props-reactivity-loss": {
          status: "mapped",
          patinaRule: "vue/no-setup-props-reactivity-loss",
        },
        "vue/no-v-html": { status: "mapped", patinaRule: "vue/no-v-html" },
      },
    },
    findings = [
      {
        file,
        ruleId: "vue/no-setup-props-reactivity-loss",
        severity: 1,
        ...span(2, 1, 2, 6),
        message: "reactivity loss",
      },
      {
        file,
        ruleId: "vue/no-v-html",
        severity: 1,
        ...span(3, 1, 3, 6),
        message: "v-html",
      },
    ];
  fs.mkdirSync(path.dirname(filename), { recursive: true });
  fs.writeFileSync(filename, "<script setup>\nconst props = defineProps()\n</script>\n");
  try {
    const compatible = compareLintFindings({
        projectId: "fixture",
        cwd,
        ruleMap: map,
        patinaFindings: findings,
        eslintResults: [],
      }),
      successor = compareSuccessor("fixture", cwd, map, findings, [], { files: [file] });
    for (const result of [compatible, successor]) {
      const summary = JSON.parse(JSON.stringify(result.summary)) as Record<string, unknown>;
      assert.equal(summary.falsePositiveCount, 1);
      assert.equal(summary.intentionalDivergenceCount, 1);
      assert.equal(summary.falsePositiveRatio, 1);
    }
  } finally {
    fs.rmSync(filename, { force: true });
  }
});

test("opaque component factories retain template binding uncertainty", () => {
  const file = "src/Factory.vue",
    filename = path.join(cwd, file),
    map = {
      upstream: { package: "eslint-plugin-vue", version: "10.9.2" },
      entries: {
        "vue/no-deprecated-dollar-listeners-api": {
          status: "mapped",
          patinaRule: "script/no-deprecated-dollar-listeners-api",
        },
      },
    },
    baseline = [
      eslintResult(file, [
        {
          ruleId: "vue/no-deprecated-dollar-listeners-api",
          severity: 2,
          ...span(1, 70, 1, 80),
          message: "The `$listeners` is deprecated.",
        },
      ]),
    ],
    source =
      '<script>export default new ComponentFactory({})</script><template><Child v-on="$listeners" /></template>';
  fs.mkdirSync(path.dirname(filename), { recursive: true });
  fs.writeFileSync(filename, source);
  try {
    const compatible = compareLintFindings({
        projectId: "fixture",
        cwd,
        ruleMap: map,
        patinaFindings: [],
        eslintResults: baseline,
      }),
      successor = compareSuccessor("fixture", cwd, map as Json, [], baseline as unknown as Json[], {
        files: [file],
      });
    for (const result of [
      {
        falseNegatives: compatible.falseNegatives,
        intentionalDivergences: compatible.intentionalDivergences,
      },
      {
        falseNegatives: list(successor.falseNegatives),
        intentionalDivergences: list(successor.intentionalDivergences),
      },
    ]) {
      assert.equal(result.falseNegatives.length, 0, JSON.stringify(result, null, 2));
      assert.equal(result.intentionalDivergences.length, 1);
      const reason = get(result.intentionalDivergences[0] as Json, "reason");
      assert.equal(typeof reason, "string");
      assert.match(reason, /opaque user factory/u);
    }
  } finally {
    fs.rmSync(filename, { force: true });
  }
});

test("unimplemented and intentionally divergent rules never count as parity failures", () => {
  const result = compare(
    [],
    [
      eslintResult("src/App.vue", [
        {
          ruleId: "vue/no-undef-properties",
          severity: 1,
          ...span(4, 1, 4, 6),
          message: "'x' is not defined.",
        },
        {
          ruleId: "vue/max-attributes-per-line",
          severity: 1,
          ...span(7, 3, 7, 9),
          message: "too many attributes",
        },
      ]),
    ],
  );

  assert.deepEqual(result.falseNegatives, []);
  assert.deepEqual(result.unimplemented, [
    {
      file: "src/App.vue",
      ruleId: "vue/no-undef-properties",
      severity: "warning",
      line: 4,
      column: 1,
      endLine: 4,
      endColumn: 6,
      message: "'x' is not defined.",
      issue: 3223,
    },
  ]);
  assert.deepEqual(result.intentionalDivergences, [
    {
      file: "src/App.vue",
      ruleId: "vue/max-attributes-per-line",
      severity: "warning",
      line: 7,
      column: 3,
      endLine: 7,
      endColumn: 9,
      message: "too many attributes",
      reason: "formatting-category rule owned by glyph",
    },
  ]);
  assert.equal(result.summary.comparableBaselineCount, 0);
  assert.equal(result.summary.falseNegativeRatio, 0);
});

test("patina-only rules are extra coverage, not false positives", () => {
  const result = compare(
    [
      {
        file: "src/App.vue",
        ruleId: "vue/no-array-index-key",
        severity: 1,
        ...span(5, 10, 5, 20),
        message: "array index key",
      },
    ],
    [eslintResult("src/App.vue", [])],
  );

  assert.deepEqual(result.falsePositives, []);
  assert.equal(result.summary.patinaOnlyRuleFindingCount, 1);
  assert.equal(result.patinaOnlyRuleFindings[0].ruleId, "vue/no-array-index-key");
  assert.equal(result.summary.falsePositiveRatio, 0);
});

test("parse errors are counted out rather than read as a parity gap", () => {
  const result = compare(
    [],
    [
      eslintResult("src/App.vue", [
        { ruleId: null, severity: 2, ...span(1, 1, 1, 2), message: "Parsing error" },
      ]),
      eslintResult("src/main.ts", [
        { ruleId: "vue/no-v-html", severity: 2, ...span(1, 1, 1, 2), message: "not a vue file" },
      ]),
    ],
  );

  assert.deepEqual(result.falseNegatives, []);
  assert.equal(result.summary.baselineParseErrorCount, 1);
  assert.equal(result.summary.baselineExcludedNonVueCount, 1);
  assert.equal(result.summary.baselineFindingCount, 0);
});

test("baseline findings with invalid ranges are counted out as unusable evidence", () => {
  const result = compare(
    [],
    [
      eslintResult("src/App.vue", [
        {
          ruleId: "vue/no-v-html",
          severity: 2,
          ...span(7, 1, 0, 0),
          message: "bad location",
        },
      ]),
    ],
  );

  assert.deepEqual(result.falseNegatives, []);
  assert.equal(result.summary.baselineInvalidRangeCount, 1);
  assert.equal(result.summary.baselineFindingCount, 0);
});

test("patina findings with invalid ranges remain fail-closed", () => {
  assert.throws(
    () =>
      compare(
        [
          {
            file: "src/App.vue",
            ruleId: "vue/no-v-html",
            severity: 2,
            ...span(7, 1, 0, 0),
            message: "bad location",
          },
        ],
        [eslintResult("src/App.vue", [])],
      ),
    /finding range must be positive safe integers: src\/App\.vue vue\/no-v-html/u,
  );
});

test("a baseline rule missing from the pinned map is a hard error", () => {
  assert.throws(
    () =>
      compare(
        [],
        [
          eslintResult("src/App.vue", [
            {
              ruleId: "vue/not-in-the-pinned-map",
              severity: 1,
              ...span(1, 1, 1, 2),
              message: "drift",
            },
          ]),
        ],
      ),
    /vue\/not-in-the-pinned-map is absent from the pinned rule map/,
  );
});

test("the classification is deterministic and hashed", () => {
  const findings = [
    {
      file: "src/App.vue",
      ruleId: "vue/no-v-html",
      severity: 2,
      ...span(3, 8, 3, 20),
      message: "v-html",
    },
  ];
  const results = [
    eslintResult("src/App.vue", [
      { ruleId: "vue/no-v-html", severity: 2, ...span(3, 8, 3, 20), message: "v-html" },
    ]),
  ];

  assert.equal(compare(findings, results).sha256, compare(findings, results).sha256);
});
