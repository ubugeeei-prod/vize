import assert from "node:assert/strict";
import { Buffer } from "node:buffer";
import path from "node:path";
import { test } from "node:test";

import { compareTypecheckDiagnostics } from "../../legacy-tools/fixtures/typecheck-divergence.mjs";
import { compare, cwd } from "./_helpers/typecheck-divergence-fixture.ts";

test("typecheck divergence classifies exact diagnostics deterministically", () => {
  const files = [
    {
      file: "src/a.vue",
      diagnostics: ["warning:9:4 [TS6133] unused   value"],
    },
    {
      file: "src/B.vue",
      diagnostics: ["error:2:3 [TS2322] Type  string  is not assignable"],
    },
  ];
  const baseline = [
    `./src/a.vue(9,4): warning TS6133: unused value`,
    `${path.join(cwd, "src/B.vue")}(2,3): error TS2322: Type string is not assignable`,
    "",
  ].join("\r\n");
  const result = compare(files, baseline);

  assert.deepEqual(Object.keys(result), [
    "schema",
    "version",
    "project",
    "summary",
    "shared",
    "messageMismatches",
    "falsePositives",
    "falseNegatives",
    "documentedDifferences",
    "sha256",
  ]);
  assert.equal(result.schema, "vize.fixtureTypecheckDivergence");
  assert.equal(result.version, 4);
  assert.deepEqual(result.summary, {
    vizeDiagnosticCount: 2,
    baselineDiagnosticCount: 2,
    sharedCount: 2,
    messageMismatchCount: 0,
    documentedDifferenceCount: 0,
    falsePositiveCount: 0,
    falseNegativeCount: 0,
    falsePositiveRatio: 0,
    falseNegativeRatio: 0,
    vizeExcludedNonVueCount: 0,
    baselineExcludedNonVueCount: 0,
    baselineExcludedProjectCount: 0,
    baselineExcludedExternalCount: 0,
    baselineExcludedSupportVueCount: 0,
  });
  assert.deepEqual(
    result.shared.map((diagnostic: { file: string }) => diagnostic.file),
    ["src/B.vue", "src/a.vue"],
  );
  assert.match(result.sha256, /^[0-9a-f]{64}$/);

  const reversed = compare([...files].reverse(), baseline.split("\r\n").reverse().join("\n"));
  assert.deepEqual(reversed, result);
});

test("typecheck divergence separates false positives and false negatives", () => {
  const result = compare(
    [
      {
        file: "src/App.vue",
        diagnostics: [
          "error:3:5 [TS2322] shared",
          "error:4:7 [TS2339] vize only",
          "warning:8:2 [TS6133] severity differs",
        ],
      },
    ],
    [
      "src/App.vue(3,5): error TS2322: shared",
      "src/App.vue(6,9): error TS2345: baseline only",
      "src/App.vue(8,2): error TS6133: severity differs",
    ].join("\n"),
  );

  assert.deepEqual(result.summary, {
    vizeDiagnosticCount: 3,
    baselineDiagnosticCount: 3,
    sharedCount: 1,
    messageMismatchCount: 0,
    documentedDifferenceCount: 0,
    falsePositiveCount: 2,
    falseNegativeCount: 2,
    falsePositiveRatio: 2 / 3,
    falseNegativeRatio: 2 / 3,
    vizeExcludedNonVueCount: 0,
    baselineExcludedNonVueCount: 0,
    baselineExcludedProjectCount: 0,
    baselineExcludedExternalCount: 0,
    baselineExcludedSupportVueCount: 0,
  });
  assert.deepEqual(
    result.falsePositives.map((entry: { code: number }) => entry.code),
    [2339, 6133],
  );
  assert.deepEqual(
    result.falseNegatives.map((entry: { code: number }) => entry.code),
    [2345, 6133],
  );
});

test("typecheck divergence retains duplicate diagnostics as a multiset", () => {
  const result = compare(
    [
      {
        file: "src/App.vue",
        diagnostics: ["error:1:1 [TS2307] z message", "error:1:1 [TS2307] a message"],
      },
    ],
    "src/App.vue(1,1): error TS2307: baseline message\n",
  );

  assert.equal(result.summary.sharedCount, 0);
  assert.equal(result.summary.messageMismatchCount, 1);
  assert.equal(result.summary.falsePositiveCount, 1);
  assert.equal(result.messageMismatches[0].vizeMessage, "a message");
  assert.equal(result.falsePositives[0].message, "z message");
});

/**
 * Regression for #3447. Both of these agree with vue-tsc on file, severity,
 * line, column and code and differ only in the message, so the identity the
 * comparator groups on cannot tell them apart from a match. The first is the
 * `.vue.ts` specifier leak fixed in #3397, which the ledger scored `shared`
 * while it sat on main; the second is vize naming its own synthetic wrapper
 * type in user-facing text. Before the fix both landed in `shared` and
 * `messageMismatches` did not exist.
 */
test("typecheck divergence separates a message-only divergence from a match", () => {
  const result = compare(
    [
      {
        file: "src/App.vue",
        diagnostics: [
          "error:2:20 [TS2307] Cannot find module './Absent.vue.ts' or its corresponding type declarations.",
        ],
      },
      {
        file: "src/Parent.vue",
        diagnostics: [
          "error:6:36 [TS2339] Property 'nope' does not exist on type '__VizeComponentInstance'.",
        ],
      },
    ],
    [
      "src/App.vue(2,20): error TS2307: Cannot find module './Absent.vue' or its corresponding type declarations.",
      "src/Parent.vue(6,36): error TS2339: Property 'nope' does not exist on type 'CreateComponentPublicInstanceWithMixins<ToResolvedProps<{}, {}>, { reload: (times: number) => number; }, {}, {}, {}, ComponentOptionsMixin, ComponentOptionsMixin, ... 18 more ..., {}>'.",
    ].join("\n"),
  );

  assert.equal(result.summary.sharedCount, 0);
  assert.equal(result.summary.messageMismatchCount, 2);
  assert.equal(result.summary.falsePositiveCount, 0);
  assert.equal(result.summary.falseNegativeCount, 0);
  assert.deepEqual(result.messageMismatches, [
    {
      file: "src/App.vue",
      severity: "error",
      line: 2,
      column: 20,
      code: 2307,
      vizeMessage: "Cannot find module './Absent.vue.ts' or its corresponding type declarations.",
      baselineMessage: "Cannot find module './Absent.vue' or its corresponding type declarations.",
    },
    {
      file: "src/Parent.vue",
      severity: "error",
      line: 6,
      column: 36,
      code: 2339,
      vizeMessage: "Property 'nope' does not exist on type '__VizeComponentInstance'.",
      baselineMessage:
        "Property 'nope' does not exist on type 'CreateComponentPublicInstanceWithMixins<ToResolvedProps<{}, {}>, { reload: (times: number) => number; }, {}, {}, {}, ComponentOptionsMixin, ComponentOptionsMixin, ... 18 more ..., {}>'.",
    },
  ]);
});

test("typecheck divergence accepts a diagnostic-free baseline without NaN ratios", () => {
  const result = compare([{ file: "src/App.vue", diagnostics: [] }], "");
  assert.equal(result.summary.falsePositiveRatio, 0);
  assert.equal(result.summary.falseNegativeRatio, 0);
  assert.deepEqual(result.shared, []);
  assert.deepEqual(result.falsePositives, []);
  assert.deepEqual(result.falseNegatives, []);
});

test("typecheck divergence records non-Vue diagnostics outside the comparison surface", () => {
  const result = compare(
    [
      { file: "src/App.vue", diagnostics: [] },
      { file: "src/helper.ts", diagnostics: ["error:1:1 [TS2322] Vize TypeScript"] },
    ],
    [
      "src/helper.ts(1,1): error TS2322: baseline TypeScript",
      "src/other.ts(2,3): error TS2339: baseline only TypeScript",
    ].join("\n"),
  );
  assert.equal(result.summary.vizeDiagnosticCount, 0);
  assert.equal(result.summary.baselineDiagnosticCount, 0);
  assert.equal(result.summary.vizeExcludedNonVueCount, 1);
  assert.equal(result.summary.baselineExcludedNonVueCount, 2);
  assert.equal(result.summary.baselineExcludedProjectCount, 0);
  assert.equal(result.summary.baselineExcludedExternalCount, 0);
  assert.equal(result.summary.baselineExcludedSupportVueCount, 0);
  assert.deepEqual(result.falsePositives, []);
  assert.deepEqual(result.falseNegatives, []);
});

test("typecheck divergence records project-level and external baseline diagnostics", () => {
  const result = compare(
    [{ file: "src/App.vue", diagnostics: [] }],
    [
      "error TS2688: Cannot find type definition file for 'vitest/globals'.",
      `${path.join(cwd, "..", "node_modules", "types.d.ts")}(1,1): error TS2304: external`,
      "../external.vue(2,3): warning TS6133: external Vue file",
    ].join("\n"),
  );
  assert.equal(result.summary.baselineDiagnosticCount, 0);
  assert.equal(result.summary.baselineExcludedProjectCount, 1);
  assert.equal(result.summary.baselineExcludedExternalCount, 2);
  assert.equal(result.summary.baselineExcludedSupportVueCount, 0);
  assert.deepEqual(result.falseNegatives, []);
});

test("typecheck divergence excludes dot-directory support Vue diagnostics", () => {
  const result = compare(
    [{ file: "src/App.vue", diagnostics: ["error:1:1 [TS2322] shared"] }],
    [
      "src/App.vue(1,1): error TS2322: shared",
      "docs/.vitepress/components/Support.vue(2,3): error TS2345: support only",
    ].join("\n"),
  );
  assert.equal(result.summary.baselineDiagnosticCount, 1);
  assert.equal(result.summary.baselineExcludedSupportVueCount, 1);
  assert.equal(result.summary.falseNegativeCount, 0);
  assert.deepEqual(result.falseNegatives, []);
});

test("typecheck divergence keeps a dot-prefixed Vue filename in the comparison", () => {
  const result = compare(
    [{ file: "src/App.vue", diagnostics: [] }],
    "src/.Local.vue(1,1): error TS2345: dot-prefixed filename\n",
  );
  assert.equal(result.summary.baselineExcludedSupportVueCount, 0);
  assert.equal(result.summary.falseNegativeCount, 1);
});

test("typecheck divergence compares dot-directory Vue files Vize itself checked", () => {
  const result = compare(
    [
      {
        file: "docs/.vitepress/components/Support.vue",
        diagnostics: ["error:2:3 [TS2345] shared"],
      },
    ],
    "docs/.vitepress/components/Support.vue(2,3): error TS2345: shared\n",
  );
  assert.equal(result.summary.baselineExcludedSupportVueCount, 0);
  assert.equal(result.summary.sharedCount, 1);
  assert.equal(result.summary.falseNegativeCount, 0);
});

test("typecheck divergence uses UTF-8 byte order for Unicode paths", () => {
  const paths = ["src/😀.vue", "src/é.vue", "src/z.vue", "src/Ω.vue"];
  const result = compare(
    paths.map((file, index) => ({
      file,
      diagnostics: [`error:1:1 [TS${2000 + index}] message`],
    })),
    "",
  );
  const expected = [...paths].sort((left, right) =>
    Buffer.compare(Buffer.from(left), Buffer.from(right)),
  );
  assert.deepEqual(
    result.falsePositives.map((entry: { file: string }) => entry.file),
    expected,
  );
});

test("typecheck divergence rejects ambiguous diagnostics and escaping paths", () => {
  const validFiles = [{ file: "src/App.vue", diagnostics: [] }];
  for (const [files, output, message] of [
    [
      [{ file: "src/App.vue", diagnostics: ["error:1:1 missing-code"] }],
      "",
      /unparseable Vize diagnostic/,
    ],
    [
      [{ file: "src/App.vue", diagnostics: ["error:0:1 [TS2322] invalid range"] }],
      "",
      /positive safe integers/,
    ],
    [[{ file: "../App.vue", diagnostics: [] }], "", /stay inside/],
    [validFiles, "prefix error TS5058: missing config\n", /unparseable vue-tsc diagnostic/],
  ] as const) {
    assert.throws(() => compare(files as never, output), message);
  }
});

test("typecheck divergence rejects invalid envelopes", () => {
  assert.throws(
    () =>
      compareTypecheckDiagnostics({
        projectId: "",
        cwd,
        vizeReport: { files: [] },
        vueTscOutput: "",
      }),
    /project id is required/,
  );
  assert.throws(
    () =>
      compareTypecheckDiagnostics({
        projectId: "fixture",
        cwd: "relative",
        vizeReport: { files: [] },
        vueTscOutput: "",
      }),
    /cwd must be absolute/,
  );
  assert.throws(() =>
    compareTypecheckDiagnostics({
      projectId: "fixture",
      cwd,
      vizeReport: {},
      vueTscOutput: "",
    }),
  );
});
