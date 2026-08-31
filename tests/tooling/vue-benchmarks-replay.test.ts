import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

// @ts-expect-error plain-JS bench module without type declarations
import {
  DEFAULT_REPLAY_TOOLS,
  diffExpectations,
  scoreCase,
  UPSTREAM,
} from "../../tools/benchmarks/scripts/vue-benchmarks-replay.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("replay scoring follows the upstream meta contract", () => {
  const report = (errorCount: number) => ({
    errorCount,
    warningCount: 0,
    files: [],
  });

  assert.deepEqual(scoreCase({ expectErrors: false }, 0, report(0), ""), {
    outcome: "pass",
    detail: "clean",
  });
  assert.deepEqual(scoreCase({ expectErrors: false }, 1, report(2), ""), {
    outcome: "fail",
    detail: "expected clean, got 2 error(s)",
  });
  assert.deepEqual(scoreCase({ expectErrors: true, minErrors: 1 }, 0, report(0), ""), {
    outcome: "fail",
    detail: "expected >=1 error(s), got 0",
  });
  assert.deepEqual(
    scoreCase({ expectErrors: true, mustMatch: ["TS2322"] }, 1, report(1), "error:2:7 [TS2322]"),
    { outcome: "pass", detail: "caught 1 error(s)" },
  );
  assert.deepEqual(
    scoreCase({ expectErrors: true, mustMatch: ["TS9999"] }, 1, report(1), "error:2:7 [TS2322]"),
    { outcome: "fail", detail: "diagnostics did not match any of: TS9999" },
  );
  assert.deepEqual(
    scoreCase({ expectErrors: true, mustNotMatch: ["__vize_"] }, 1, report(1), "__vize_ctx leak"),
    { outcome: "fail", detail: "output contained a forbidden pattern" },
  );
  assert.deepEqual(scoreCase({ expectErrors: true }, 1, null, "panic"), {
    outcome: "fail",
    detail: "no JSON report (status=1)",
  });
});

test("replay expectation drift fails closed in both directions", () => {
  const results = [
    { caseId: "a", outcome: "pass", detail: "clean" },
    { caseId: "b", outcome: "fail", detail: "expected >=1 error(s), got 0" },
    { caseId: "c", outcome: "pass", detail: "caught 1 error(s)" },
    { caseId: "d", outcome: "skip", detail: "requires strict-component-attrs" },
  ];

  assert.deepEqual(diffExpectations(results, { a: "pass", b: "fail", c: "pass", d: "skip" }), []);

  const problems = diffExpectations(results, {
    a: "pass",
    b: "pass",
    c: "fail",
    e: "pass",
  });
  assert.deepEqual(problems, [
    "b: expected pass, got fail (expected >=1 error(s), got 0)",
    'c: stale suppression — the case now passes; remove the "fail" expectation',
    'd: not in the expectation table (add "skip")',
    "e: in the expectation table but missing from the corpus",
  ]);
});

test("replay expectation tables stay pinned to the upstream corpus", () => {
  assert.deepEqual(UPSTREAM, {
    repository: "pikax/vue-benchmarks",
    url: "https://github.com/pikax/vue-benchmarks",
    commit: "65c6102504b14cd49c0b03305be8dd0b9d208c59",
    license: "MIT",
  });
  assert.deepEqual(DEFAULT_REPLAY_TOOLS, [
    "vize",
    "verter-tsc",
    "golar-typecheck",
    "golar-default",
  ]);

  const tables = ["main", "release"].map((lane) => {
    const tablePath = path.join(
      root,
      `tools/benchmarks/scripts/vue-benchmarks-replay-expect-${lane}.json`,
    );
    return {
      lane,
      table: JSON.parse(fs.readFileSync(tablePath, "utf8")) as Record<string, string>,
    };
  });
  const [main, release] = tables;
  assert.deepEqual(Object.keys(main.table), Object.keys(release.table));
  for (const { lane, table } of tables) {
    assert.equal(Object.keys(table).length, 150, `${lane} table must cover the pinned corpus`);
    for (const [caseId, outcome] of Object.entries(table)) {
      assert.match(caseId, /^[a-z0-9][a-z0-9-]*$/);
      assert.ok(["pass", "fail"].includes(outcome), `${lane}:${caseId}=${outcome}`);
    }
  }

  // The 2026-07-27 upstream report's literal-union finding is the canonical
  // fixed-external-failure: 0.291.0 missed it, main caught it, and released
  // 0.302.0 shipped the fix. Both lanes now pin it as a required pass, so a
  // regression in either one fails the replay instead of being suppressed.
  assert.equal(main.table["literal-union-prop-bad"], "pass");
  assert.equal(release.table["literal-union-prop-bad"], "pass");
  assert.equal(main.table["script-type-error"], "pass");
  assert.equal(release.table["script-type-error"], "pass");
});
