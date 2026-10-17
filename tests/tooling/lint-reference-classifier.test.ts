import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  classifyReferenceCoverage as classify,
  eligibleReferenceCoverage as eligible,
  referenceCoverageCases,
  type Json,
} from "../../tools/support/fixtures/lint_reference_coverage.ts";

function fixture(index: number) {
  const entry = referenceCoverageCases()[index];
  assert.ok(entry);
  assert.equal(typeof entry.finding.ruleId, "string");
  const rule = String(entry.finding.ruleId);
  const context: Json = {
    revision: entry.revision,
    preset: entry.preset,
    runtimeVersions: entry.runtimeVersions,
    files: [entry.finding.file],
    rules: { [rule]: entry.finding.severity === "warning" ? "warn" : "error" },
  };
  const source = readFileSync(
    new URL(`../_fixtures/differential/linter/reference-coverage-8849/${entry.source}`, import.meta.url),
    "utf8",
  );
  return { entry, context, source, rule, finding: entry.finding };
}
const noScan = () => assert.fail("unrelated or unmeasured findings must not scan reference packets");

for (let index = 0; index < 2; index++) {
  const { entry, context, source, rule, finding } = fixture(index);
  void test(`${entry.id}: exact original source and full finding remain required`, () => {
    const answer = classify(entry.project, context, finding, source, () => false);
    assert.ok(answer);
    assert.equal(answer.category, "reference-coverage-gap");
    assert.equal(answer.sourceSha256, entry.sourceSha256);
    assert.deepEqual(answer.options, []);
    assert.deepEqual(eligible(entry.project, context, () => source), [entry]);
    assert.deepEqual(eligible(entry.project, context, () => source + "\n"), []);
    assert.equal(classify("other-project", context, finding, source, noScan), null);
    assert.equal(classify(entry.project, context, finding, source + "\n", noScan), null);
    for (const field of ["file", "ruleId", "severity", "line", "column", "endLine", "endColumn", "message"]) {
      const original = finding[field];
      const changed = { ...finding, [field]: typeof original === "number" ? original + 1 : String(original) + "x" };
      assert.equal(classify(entry.project, context, changed, source, noScan), null, field);
    }
    assert.equal(classify(entry.project, context, { ...finding, foreign: true }, source, noScan), null);
  });

  void test(`${entry.id}: changed configuration and newly visible reference findings stay scored`, () => {
    const changed: Json[] = [
      { ...context, revision: "new revision" },
      { ...context, preset: "opinionated" },
      ...Object.keys(entry.runtimeVersions).map((runtime) => ({
        ...context,
        runtimeVersions: { ...entry.runtimeVersions, [runtime]: "next" },
      })),
      { ...context, rules: { [rule]: ["warn", { atComponent: "shorthand" }] } },
      { ...context, rules: { [rule]: "off" } },
    ];
    for (const settings of changed) {
      assert.deepEqual(eligible(entry.project, settings, noScan), []);
      assert.equal(classify(entry.project, settings, finding, source, noScan), null);
    }
    assert.equal(classify(entry.project, context, finding, source, () => true), null);
  });

  void test(`${entry.id}: missing, empty, foreign and malformed measured-file domains refuse exemption AND expectations`, () => {
    const { files: _files, ...missing } = context;
    for (const settings of [missing, ...[[], ["foreign.vue"], null, entry.finding.file, {}].map((files) => ({ ...context, files }))]) {
      assert.deepEqual(eligible(entry.project, settings, noScan), [], JSON.stringify(settings.files));
      assert.equal(classify(entry.project, settings, finding, source, noScan), null, JSON.stringify(settings.files));
    }
    const measured = { ...context, files: ["foreign.vue", entry.finding.file] };
    assert.equal(classify(entry.project, measured, finding, source, () => false)?.case, entry.id);
    assert.deepEqual(eligible(entry.project, measured, () => source), [entry]);
  });
}
