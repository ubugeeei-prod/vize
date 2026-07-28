import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

// @ts-expect-error plain-JS bench module without type declarations
import { diffExpectations, scoreCase, UPSTREAM } from "../../bench/vue-benchmarks-replay.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("replay scoring follows the upstream meta contract", () => {
  const report = (errorCount: number) => ({ errorCount, warningCount: 0, files: [] });

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

  const problems = diffExpectations(results, { a: "pass", b: "pass", c: "fail", e: "pass" });
  assert.deepEqual(problems, [
    "b: expected pass, got fail (expected >=1 error(s), got 0)",
    'c: stale suppression — the case now passes; remove the "fail" expectation',
    'd: not in the expectation table (add "skip")',
    "e: in the expectation table but missing from the corpus",
  ]);
});

test("replay expectation tables stay pinned to the upstream corpus", () => {
  assert.equal(UPSTREAM.repository, "pikax/vue-benchmarks");
  assert.match(UPSTREAM.commit, /^[0-9a-f]{40}$/);

  const tables = ["main", "release"].map((lane) => {
    const tablePath = path.join(root, `bench/vue-benchmarks-replay-expect-${lane}.json`);
    return {
      lane,
      table: JSON.parse(fs.readFileSync(tablePath, "utf8")) as Record<string, string>,
    };
  });
  const [main, release] = tables;
  assert.deepEqual(Object.keys(main.table), Object.keys(release.table));
  for (const { lane, table } of tables) {
    assert.equal(Object.keys(table).length, 29, `${lane} table must cover the pinned corpus`);
    for (const [caseId, outcome] of Object.entries(table)) {
      assert.match(caseId, /^[a-z0-9][a-z0-9-]*$/);
      assert.ok(["pass", "fail", "skip"].includes(outcome), `${lane}:${caseId}=${outcome}`);
    }
  }

  // The 2026-07-27 upstream report's literal-union finding is the canonical
  // fixed-external-failure: released 0.291.0 misses it, current main must
  // catch it. If a new release fixes it, the release replay lane reports a
  // stale suppression until this table records the pass.
  assert.equal(main.table["literal-union-prop-bad"], "pass");
  assert.equal(release.table["literal-union-prop-bad"], "fail");
  assert.equal(main.table["script-type-error"], "pass");
  assert.equal(release.table["script-type-error"], "pass");
});
