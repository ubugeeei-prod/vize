import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { after, before, test } from "node:test";
import {
  assertDigest,
  assertParity,
  command,
  contractDirectory,
  evidenceDirectory,
  fixture,
  materializeFrozenReporters,
  reportFile,
  indexFile,
  object,
  root,
  runReporter,
  type Fixture,
  type Oracle,
} from "../_helpers/lint-report-rust-parity.ts";

// Dedicated source qualification: explicitly run this file in Actions with the
// two immutable refs fetched and rust-script installed. No skip or MJS oracle.
let frozen: ReturnType<typeof materializeFrozenReporters>;
before(() => {
  frozen = materializeFrozenReporters();
  fs.writeFileSync(
    path.join(evidenceDirectory(), "finite-successor-seams.json"),
    JSON.stringify(
      {
        runtime:
          "Exact rustc --version versus exact Node process.version; machine and source identity unchanged",
        usage: "Only exact Rust/Node canonical first header; all remaining help bytes unchanged",
        durationMs:
          "Each baseline duration is a safe integer in [0, 240000]; only this monotonic measurement is removed",
        originalAdditions:
          "Only independently asserted #8849 runtimeVersions/raw/reference expectations and exact Markdown additions; both digest preimages checked",
        parserDiagnostics: [
          {
            vector: "1e309, including duplicate raw:0 shadow",
            rust: "number out of range at exact source line/column",
            node: "JSON number is out of range",
          },
          {
            vector: "unpaired high Unicode surrogate, including duplicate raw:0 shadow",
            rust: "unexpected end of hex escape at exact source line/column",
            node: "JSON strings must contain valid Unicode scalar values",
          },
        ],
        membershipFix:
          "Updated frozen Rust wrongly exempts a known but unmeasured foreign finding; TS must score it and breach unchanged zero budget",
      },
      null,
      2,
    ) + "\n",
  );
});
after(() => frozen?.clean());
const nodeCommand = path.join(root, `${command}.ts`);

function compare(input: Fixture, args = input.args, oracles: Oracle[] = ["original", "updated"]) {
  const successor = runReporter(input, nodeCommand, args);
  for (const oracle of oracles) {
    const measured = runReporter(input, frozen.paths[oracle], args, true);
    assertParity(measured, successor, oracle);
  }
  return successor;
}

test("complete CLI help and argument refusals match both frozen actual Rust commands", () => {
  const input = fixture();
  try {
    const cases = [
      ["--help"],
      ["-h"],
      ["--unknown"],
      ["--registry"],
      ["--budget-mode", "silently-ignore"],
      ["--shard-count", "0"],
      ["--shard-index", "-1"],
      ["--timeout-ms", "0"],
      ["--shard-count", "2", "--shard-index", "2"],
    ];
    for (const args of cases) {
      const result = compare(input, args);
      assert.equal(result.status, args[0] === "--help" || args[0] === "-h" ? 0 : 1);
      assert.deepEqual(result.files, {});
      assert.deepEqual(result.processes, []);
    }
  } finally {
    input.clean();
  }
});

test("complete reports preserve deterministic ignore, rule, raw process and shard behavior", () => {
  const input = fixture();
  try {
    input.write(".gitignore", "generated/\n");
    input.write("generated/Ignored.vue", "<template><div v-html='x' /></template>\n");
    input.configure([
      input.project("zeta", { vueGlobs: ["**/*.vue"] }),
      input.project("alpha", { vueGlobs: ["**/*.vue"] }),
      input.project("middle", { coverage: ["formatter"] }),
    ]);
    const args = [
      ...input.args,
      "--preset",
      "no-rules",
      "--shard-count",
      "2",
      "--shard-index",
      "0",
    ];
    const result = compare(input, args);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.processes.length, 1);
    assert.deepEqual(result.processes[0].argv, [
      "lint",
      "**/*.vue",
      "--format",
      "json",
      "--no-config",
      "--preset",
      "no-rules",
    ]);
    const reports = Object.entries(result.files).filter(([file]) =>
      file.endsWith("-lint-divergence.json"),
    );
    assert.equal(reports.length, 1);
    assert.deepEqual(reportFile(result, reports[0][0]).files, {
      comparedCount: 1,
      expectedCount: null,
    });
    assert.equal(indexFile(result).budget.passed, true);
    // .ignore overrides .gitignore; the nested .ignore re-includes one input.
    input.write(".ignore", "!generated/\ngenerated/*\n");
    input.write("generated/.ignore", "!Keep.vue\n");
    input.write("generated/Keep.vue", "<template><span /></template>\n");
    input.configure([input.project("nested", { vueGlobs: ["**/*.vue"] })], {
      results: [
        { file: "App.vue", messages: [] },
        { file: "generated/Keep.vue", messages: [] },
      ],
    });
    const nested = compare(input, [...input.args, "--preset", "no-rules"]);
    assert.equal(nested.status, 0, nested.stderr);
    assert.deepEqual(reportFile(nested, "nested-lint-divergence.json").files, {
      comparedCount: 2,
      expectedCount: null,
    });
    assert.deepEqual(nested.processes[0].argv, result.processes[0].argv);
  } finally {
    input.clean();
  }
});

test("actual pinned ESLint mapped diagnostics preserve full reports, breached budgets and warnings", () => {
  const input = fixture();
  try {
    input.write("App.vue", '<template><div v-html="html" /></template>\n');
    for (const mode of ["enforce", "record-only"]) {
      const result = compare(input, [...input.args, "--budget-mode", mode]);
      assert.equal(result.status, mode === "enforce" ? 1 : 0);
      const report = reportFile(result, "parity-lint-divergence.json");
      assert.equal(report.divergence.summary.falseNegativeCount, 1);
      assert.equal(report.divergence.falseNegatives[0].upstreamRuleId, "vue/no-v-html");
      assert.equal(report.budget.passed, false);
      assert.equal(report.budget.maxFalseNegativeCount, 0);
      assert.equal(report.raw.eslintResults.length, 1);
      assert.equal(report.raw.eslintResults[0].filePath, path.join(input.directory, "App.vue"));
    }
  } finally {
    input.clean();
  }
});

test("raw fatal ESLint diagnostics including source remain complete and unusable", () => {
  const input = fixture();
  try {
    const source = '<script setup lang="ts">const broken = ;</script>\n';
    input.write("App.vue", source);
    const result = compare(input);
    assert.equal(result.status, 1);
    const report = reportFile(result, "parity-lint-divergence.json");
    assert.equal(report.budget.verdict, "unusable");
    assert.equal(report.divergence.summary.baselineParseErrorCount, 1);
    assert.deepEqual(report.baseline.parseErrorResults, report.raw.eslintResults);
    assert.equal(report.raw.eslintResults[0].source, source);
    assert.equal(report.raw.eslintResults[0].fatalErrorCount, 1);
  } finally {
    input.clean();
  }
});

test("u64 diagnostic ranges and arbitrary raw i64/u64 keys retain every source digit", () => {
  const input = fixture();
  try {
    const stdout =
      '[{"file":"App.vue","unknown":{"min":-9223372036854775808,"max":18446744073709551615,"fraction":1.0,"negativeZero":-0.0,"small":1e-6,"large":1e16,"𐀀":2,"\ue000":3},' +
      '"messages":[{"ruleId":"vue/no-v-html","severity":1,"line":9007199254740993,"column":1,' +
      '"endLine":9007199254740993,"endColumn":2,"message":"precision control"}]}]';
    input.configure([input.project()], { stdout });
    const result = compare(input);
    assert.equal(result.status, 1);
    const report = reportFile(result, "parity-lint-divergence.json");
    assert.equal(object(report.raw.patinaResults[0].unknown).min, -9223372036854775808n);
    assert.equal(object(report.raw.patinaResults[0].unknown).max, 18446744073709551615n);
    assert.equal(report.raw.patinaFindings[0].line, 9007199254740993n);
    const observed = result.rawUnknownTokens["parity-lint-divergence.json"];
    assert.equal(observed.tokens.fraction, "1.0");
    assert.equal(observed.tokens.negativeZero, "-0.0");
    assert.deepEqual(observed.keys.slice(-2), ["\ue000", "𐀀"]);
    assert.equal(report.divergence.falsePositives[0].line, 9007199254740993n);
    assert.equal(Buffer.from(result.processes[0].stdout, "base64").toString(), stdout);
  } finally {
    input.clean();
  }
});

test("zero input requires an explicit integer zero and preserves all refusal side effects", () => {
  const input = fixture();
  try {
    fs.rmSync(path.join(input.directory, "App.vue"));
    input.configure([input.project("parity", { expectedVueFileCount: 0 })], { results: [] });
    const result = compare(input);
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(reportFile(result, "parity-lint-divergence.json").files, {
      comparedCount: 0,
      expectedCount: 0,
    });
    for (const expectedVueFileCount of [undefined, 1, -1, "0", null]) {
      input.configure([input.project("parity", { expectedVueFileCount })], { results: [] });
      const refused = compare(input);
      assert.equal(refused.status, 1);
      assert.match(refused.stderr, /parity matched no Vue files/u);
      assert.deepEqual(refused.files, {});
      assert.deepEqual(refused.processes, []);
    }
    input.write("App.vue", "<template><div /></template>\n");
    input.configure([input.project("parity", { expectedVueFileCount: 0 })]);
    const refused = compare(input);
    assert.equal(refused.status, 1);
    assert.match(refused.stderr, /parity matched 1 Vue files, expected 0/u);
    assert.deepEqual(refused.processes, []);
  } finally {
    input.clean();
  }
});

test("unselected, unhydrated and missing native envelopes keep complete refusal behavior", () => {
  const input = fixture();
  try {
    let result = compare(input, [...input.args, "--project", "absent"]);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /No linter-covered project matched the selection/u);
    assert.deepEqual(result.processes, []);
    input.configure([
      input.project("parity", {
        fixturePath: path.relative(root, path.join(input.directory, "absent")),
      }),
    ]);
    result = compare(input);
    assert.equal(result.status, 1);
    assert.match(result.stdout, /Skipped parity: fixture is not hydrated/u);
    assert.deepEqual(result.processes, []);
    input.configure([input.project()], { results: [] });
    result = compare(input);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /vize lint reported 0 files, expected 1/u);
    assert.deepEqual(result.files, {});
    input.configure([input.project()], { results: {} });
    result = compare(input);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /vize lint envelope must be an array/u);
    for (const value of [[{}], [{ file: "App.vue" }]]) {
      input.configure([input.project()], { results: value });
      result = compare(input);
      assert.equal(result.status, 1);
      assert.match(result.stderr, /vize lint entry must carry (?:file|messages)/u);
      assert.deepEqual(result.files, {});
    }
    input.configure([input.project()], { results: [], status: 17, stderr: "raw native failure\n" });
    result = compare(input);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /unsupported status 17: raw native failure/u);
    assert.equal(
      Buffer.from(result.processes[0].stderr, "base64").toString(),
      "raw native failure\n",
    );
  } finally {
    input.clean();
  }
});

test("named invalid JSON scalar vectors refuse before artifacts, including shadowed duplicate keys", () => {
  const input = fixture();
  try {
    // Only these parser diagnostics are host-specific. Ordinary CLI, validation
    // and budget errors above retain complete byte-for-byte stderr parity.
    const prefix = "vize lint produced invalid JSON: ";
    const vectors = [
      { token: "1e309", reason: "number out of range", js: "JSON number is out of range" },
      {
        token: '"\\ud800"',
        reason: "unexpected end of hex escape",
        js: "JSON strings must contain valid Unicode scalar values",
      },
    ];
    for (const vector of vectors) {
      for (const shadowed of [false, true]) {
        const stdout = `[{"file":"App.vue","messages":[],"raw":${vector.token}${shadowed ? ',"raw":0' : ""}}]`;
        const column = stdout.indexOf(vector.token) + vector.token.length;
        input.configure([input.project()], { stdout });
        const successor = runReporter(input, nodeCommand);
        assert.equal(successor.status, 1);
        assert.deepEqual(successor.files, {});
        assert.equal(successor.stdout, "");
        assert.equal(successor.stderr, `${prefix}${vector.js}\n`);
        assert.equal(Buffer.from(successor.processes[0].stdout, "base64").toString(), stdout);
        for (const oracle of ["original", "updated"] as Oracle[]) {
          const measured = runReporter(input, frozen.paths[oracle], input.args, true);
          assert.equal(measured.status, 1);
          assert.deepEqual(measured.files, {});
          assert.equal(measured.stdout, "");
          assert.equal(measured.stderr, `${prefix}${vector.reason} at line 1 column ${column}\n`);
          assert.deepEqual(measured.processes, successor.processes);
        }
      }
    }
  } finally {
    input.clean();
  }
});

interface ReviewedCase {
  id: string;
  project: string;
  revision: string;
  source: string;
  finding: { file: string; [key: string]: unknown };
}
const reviewedValue: unknown = JSON.parse(
  fs.readFileSync(path.join(root, contractDirectory, "contract.json"), "utf8"),
);
const reviewedCases = object(reviewedValue).cases;
assert.ok(Array.isArray(reviewedCases));
const reviewed = reviewedCases.map((value: unknown): ReviewedCase => {
  const row = object(value);
  for (const key of ["id", "project", "revision", "source"])
    assert.equal(typeof row[key], "string");
  assert.equal(typeof object(row.finding).file, "string");
  return row as unknown as ReviewedCase;
});
for (const entry of reviewed) {
  test(`updated real Rust and TS preserve full reviewed case ${entry.id} and missing/duplicate findings`, () => {
    const input = fixture();
    try {
      fs.rmSync(path.join(input.directory, "App.vue"));
      input.write(
        entry.finding.file,
        fs.readFileSync(path.join(root, contractDirectory, entry.source), "utf8"),
      );
      const project = input.project(entry.project, {
        revision: entry.revision,
        vueGlobs: [entry.finding.file],
        expectedVueFileCount: 1,
      });
      const { file, ...message } = entry.finding;
      for (const count of [1, 0, 2]) {
        input.configure([project], {
          results: [{ file, messages: Array.from({ length: count }, () => message) }],
        });
        const result = compare(input, input.args, ["updated"]);
        assert.equal(result.status, count === 1 ? 0 : 1, result.stderr);
        const report = reportFile(result, `${entry.project}-lint-divergence.json`);
        assert.equal(report.divergence.referenceCoverageExpectations.length, 1);
        assert.equal(report.divergence.referenceCoverageExpectations[0].observedCount, count);
        assert.equal(
          report.divergence.summary.referenceCoverageIntegrityErrorCount,
          count === 1 ? 0 : 1,
        );
        assert.equal(report.budget.maxFalsePositiveCount, 0);
        assert.equal(report.budget.maxFalseNegativeCount, 0);
      }
    } finally {
      input.clean();
    }
  });
}

test("unmeasured foreign reviewed source is scored: explicit correction to updated Rust membership bug", () => {
  const input = fixture();
  try {
    const entry = reviewed[0];
    input.write(
      entry.finding.file,
      fs.readFileSync(path.join(root, contractDirectory, entry.source), "utf8"),
    );
    const { file, ...message } = entry.finding;
    input.configure([input.project(entry.project, { revision: entry.revision })], {
      results: [{ file, messages: [message] }],
    });
    const old = runReporter(input, frozen.paths.updated, input.args, true);
    const actual = runReporter(input, nodeCommand);
    assert.equal(old.status, 0, old.stderr);
    assert.equal(actual.status, 1, actual.stderr);
    const report = reportFile(actual, `${entry.project}-lint-divergence.json`);
    assertDigest(report.divergence);
    assert.deepEqual(report.divergence.referenceCoverageExpectations, []);
    assert.equal(report.divergence.summary.falsePositiveCount, 1);
    assert.equal(report.divergence.summary.patinaOutsideBaselineSurfaceCount, 0);
    assert.equal(report.budget.verdict, "breached");
    assert.deepEqual(actual.processes, old.processes);
    assert.equal(report.raw.patinaResults[0].file, entry.finding.file);
  } finally {
    input.clean();
  }
});
