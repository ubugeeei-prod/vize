import assert from "node:assert/strict";
import { Buffer } from "node:buffer";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { compareTypecheckDiagnostics } from "../../tools/fixtures/typecheck-divergence.mjs";

const cwd = path.join(os.tmpdir(), "vize-divergence-workspace");

function compare(
  files: Array<{ file: string; diagnostics: string[] }>,
  vueTscOutput: string,
  documentedDifferences?: unknown[],
) {
  return compareTypecheckDiagnostics({
    projectId: "fixture",
    cwd,
    vizeReport: { files },
    vueTscOutput,
    documentedDifferences,
  });
}

const suggestionDifference = {
  project: "fixture",
  file: "src/App.vue",
  severity: "error",
  line: 5,
  column: 16,
  vize: { code: 2552, message: "Cannot find name 'useRouter'. Did you mean 'router'?" },
  baseline: { code: 2304, message: "Cannot find name 'useRouter'." },
  issue: 3358,
  reason: "vue-tsc exhausts its per-program spelling-suggestion budget before this span.",
};

const suggestionFiles = [
  {
    file: "src/App.vue",
    diagnostics: ["error:5:16 [TS2552] Cannot find name 'useRouter'. Did you mean 'router'?"],
  },
];
const suggestionBaseline = "src/App.vue(5,16): error TS2304: Cannot find name 'useRouter'.\n";

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
    "falsePositives",
    "falseNegatives",
    "documentedDifferences",
    "sha256",
  ]);
  assert.equal(result.schema, "vize.fixtureTypecheckDivergence");
  assert.equal(result.version, 3);
  assert.deepEqual(result.summary, {
    vizeDiagnosticCount: 2,
    baselineDiagnosticCount: 2,
    sharedCount: 2,
    documentedDifferenceCount: 0,
    falsePositiveCount: 0,
    falseNegativeCount: 0,
    falsePositiveRatio: 0,
    falseNegativeRatio: 0,
    vizeExcludedNonVueCount: 0,
    baselineExcludedNonVueCount: 0,
    baselineExcludedProjectCount: 0,
    baselineExcludedExternalCount: 0,
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
    documentedDifferenceCount: 0,
    falsePositiveCount: 2,
    falseNegativeCount: 2,
    falsePositiveRatio: 2 / 3,
    falseNegativeRatio: 2 / 3,
    vizeExcludedNonVueCount: 0,
    baselineExcludedNonVueCount: 0,
    baselineExcludedProjectCount: 0,
    baselineExcludedExternalCount: 0,
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

  assert.equal(result.summary.sharedCount, 1);
  assert.equal(result.summary.falsePositiveCount, 1);
  assert.equal(result.shared[0].vizeMessage, "a message");
  assert.equal(result.falsePositives[0].message, "z message");
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
  assert.deepEqual(result.falseNegatives, []);
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

test("typecheck divergence cancels a documented difference against both buckets", () => {
  const result = compare(suggestionFiles, suggestionBaseline, [suggestionDifference]);

  assert.equal(result.summary.documentedDifferenceCount, 1);
  assert.equal(result.summary.falsePositiveCount, 0);
  assert.equal(result.summary.falseNegativeCount, 0);
  assert.equal(result.summary.falsePositiveRatio, 0);
  assert.equal(result.summary.falseNegativeRatio, 0);
  assert.equal(result.summary.vizeDiagnosticCount, 1);
  assert.equal(result.summary.baselineDiagnosticCount, 1);
  assert.deepEqual(result.documentedDifferences, [
    {
      file: "src/App.vue",
      severity: "error",
      line: 5,
      column: 16,
      vize: suggestionDifference.vize,
      baseline: suggestionDifference.baseline,
      issue: 3358,
      reason: suggestionDifference.reason,
    },
  ]);

  const withoutLedger = compare(suggestionFiles, suggestionBaseline);
  assert.equal(withoutLedger.summary.documentedDifferenceCount, 0);
  assert.equal(withoutLedger.summary.falsePositiveCount, 1);
  assert.equal(withoutLedger.summary.falseNegativeCount, 1);
});

test("typecheck divergence keeps a documented difference that no longer reproduces", () => {
  const reworded = { code: 2552, message: "Cannot find name 'useRouter'." };
  for (const [difference, label] of [
    [{ ...suggestionDifference, project: "other" }, "another project"],
    [{ ...suggestionDifference, column: 15 }, "a shifted column"],
    [{ ...suggestionDifference, vize: reworded }, "a reworded vize message"],
    [{ ...suggestionDifference, baseline: { code: 2551, message: "x y" } }, "a new vue-tsc code"],
  ] as const) {
    const result = compare(suggestionFiles, suggestionBaseline, [difference]);
    assert.equal(result.summary.documentedDifferenceCount, 0, label);
    assert.equal(result.summary.falsePositiveCount, 1, label);
    assert.equal(result.summary.falseNegativeCount, 1, label);
  }
  // Only vize reports at 5:16, so there is nothing to cancel the false positive
  // against and the ledger entry must not hide it.
  const oneSided = compare(suggestionFiles, "", [suggestionDifference]);
  assert.equal(oneSided.summary.documentedDifferenceCount, 0);
  assert.equal(oneSided.summary.falsePositiveCount, 1);
});

test("typecheck divergence rejects an unreviewable documented difference", () => {
  for (const [difference, message] of [
    [{ ...suggestionDifference, reason: "cosmetic" }, /reason must explain/],
    [{ ...suggestionDifference, issue: 0 }, /issue must be the tracking issue/],
    [{ ...suggestionDifference, project: "" }, /must name a project/],
    [{ ...suggestionDifference, severity: "info" }, /severity must be error or warning/],
    [{ ...suggestionDifference, line: 0 }, /line must be a positive safe integer/],
    [{ ...suggestionDifference, file: "src/App.ts" }, /must reference a \.vue file/],
    [{ ...suggestionDifference, file: "../App.vue" }, /stay inside/],
    [{ ...suggestionDifference, baseline: suggestionDifference.vize }, /must record a difference/],
    [{ ...suggestionDifference, vize: { code: 2552 } }, /message must be a string/],
  ] as const) {
    assert.throws(() => compare(suggestionFiles, suggestionBaseline, [difference]), message);
  }
  assert.throws(
    () =>
      compare(suggestionFiles, suggestionBaseline, [suggestionDifference, suggestionDifference]),
    /duplicates an earlier documented difference/,
  );
  assert.throws(
    () => compare(suggestionFiles, suggestionBaseline, "no" as never),
    /must be an array/,
  );
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
