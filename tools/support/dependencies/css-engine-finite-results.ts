// Qualify authored finite successors on the same complete original observations.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import {
  assertOriginalCLIPlan,
  assertWholeFormatterCell,
  assertWholeLinterCell,
  finiteSuccessor,
} from "../../../tests/tooling/support/css-engine-successors.ts";

const directory = process.argv[2];
assert(directory, "the actual observation artifact directory is required");
const output = path.join(directory, "finite-contract.json");
const report = {
  scope:
    "Source-built Rust consumers and CLI; no native-binding, nominal compatibility or publication credit",
  completed: false,
  rustOriginal: 0,
  rustFinite: 0,
  cliOriginal: 0,
  cliFinite: 0,
  atelierOriginal: 0,
  atelierNamedWhole: 0,
  atelierGuard: 0,
  atelierHealthyShape: 0,
  additionalWhole: 0,
  subsequentHealthyWhole: 0,
  historicalFixedNeighbors: 0,
  errors: [] as string[],
};
const persist = () => fs.writeFileSync(output, JSON.stringify(report, null, 2) + "\n");
persist();
const read = (filename: string) => JSON.parse(fs.readFileSync(filename, "utf8"));
const fixture = (name: string) => path.join("tests/_fixtures/differential", name);
const corpus = read(fixture("css-engine-boundary-3295/cases.json"));
const finite = read(fixture("css-engine-vendor-exit/finite-successors.json"));
const baseline = read(fixture("css-engine-boundary-cli-3295/capture.json"));
const observed = read(path.join(directory, "rust-consumers.json"));
const cli = read(path.join(directory, "cli.json"));
const git = spawnSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" });
assert.equal(git.status, 0);
assert.equal(observed.sourceRevision, git.stdout.trim());
assert.equal(cli.sourceRevision, git.stdout.trim());
assert.equal(observed.completed, true);
assert.equal(cli.completed, true);
assert.equal(observed.panicUnwind, true);
assert(["dev", "release"].includes(cli.profile));
assert.equal(observed.debugAssertions, cli.profile === "dev");
assert.equal(observed.expectationsRecaptured, false);
assert.equal(cli.expectationsRecaptured, false);
assert.equal(observed.publicationQualification, false);
assert.equal(cli.publicationQualification, false);
assert.equal(observed.corpusSha256, finite.originalCorpusSHA256);
assert.equal(cli.beforeReferenceSha256, finite.originalCLIExpectedSHA256);
const check = (label: string, body: () => void) => {
  try {
    body();
  } catch (error) {
    report.errors.push(`${label}: ${String(error)}`);
  }
  persist();
};
let index = 0;
for (const [surface, group] of [
  ["formatter", "formatterCases"],
  ["style", "styleCases"],
  ["linter", "linterCases"],
]) {
  for (const entry of corpus[group]) {
    const repaired = finiteSuccessor(surface, entry);
    let source = entry.source;
    for (let pass = 1; pass <= 3; pass += 1) {
      const row = observed.rustConsumers[index++];
      check(`${surface}/${entry.id}/${pass}`, () => {
        assert.equal(row.surface, surface);
        assert.equal(row.case, entry.id);
        assert.equal(row.pass, pass);
        assert.equal(row.source, source);
        if (surface !== "style") assert.deepEqual(row.options, entry.options);
        if (surface === "formatter") {
          const code = repaired?.wholeCode ?? entry.expected;
          assert.equal(row.beforeExpected, entry.expected);
          assert.deepEqual(row.actual, { code, changed: source !== code });
          source = code;
        } else if (surface === "style") {
          assert.deepEqual(row.beforeCase, entry);
          const expected =
            repaired?.wholeExpected ??
            (entry.error
              ? { variant: "StyleFormatError", message: entry.error, display: entry.displayError }
              : { code: entry.expected });
          assert.deepEqual(row.actual, expected);
          if (Object.hasOwn(expected, "code")) source = expected.code;
        } else {
          assert.deepEqual(row.beforeExpected, entry.expected);
          assert.deepEqual(row.actual, repaired?.wholeExpected ?? entry.expected);
          if (repaired)
            assert.equal(row.fixedActual, undefined, "corrected original produces no fix");
          else if (entry.fixedExpected)
            assert.deepEqual(row.fixedActual, {
              source: entry.fixedExpected,
              result: entry.fixedResultExpected,
            });
        }
      });
      report.rustOriginal += 1;
      if (repaired) report.rustFinite += 1;
    }
  }
}
assert.equal(observed.rustConsumers.length, 99);
assert.equal(index, 99);
const originalInputs = read(fixture("css-engine-vendor-exit/atelier-inputs.json"));
for (const [inputGroup, resultGroup] of [
  ["observations", "atelier"],
  ["additionalInputs", "additionalAtelier"],
]) {
  const inputs = originalInputs[inputGroup];
  assert.equal(observed[resultGroup].length, inputs.length);
  for (const [i, input] of inputs.entries()) {
    const row = observed[resultGroup][i];
    check(`${resultGroup}/${i}`, () => {
      assert.deepEqual(row.input, input);
      const expected =
        resultGroup === "atelier"
          ? finite.atelierRoles.find(
              (r: { input: unknown }) =>
                (r.input as { law: string; index: number }).law === input.law &&
                (r.input as { law: string; index: number }).index === input.index,
            )
          : finite.additionalAtelier.find(
              (r: { input: { id: string } }) => r.input.id === input.id,
            );
      if (expected) {
        assert.deepEqual(expected.input, input);
        assert.deepEqual(
          { parsed: row.parsed, compiled: row.compiled, printed: row.printed },
          expected.wholeExpected,
        );
        if (resultGroup === "atelier") report.atelierNamedWhole += 1;
        else report.additionalWhole += 1;
      } else if (row.parsed.ast === undefined) {
        const error =
          "CSS parse error: unsupported math function in a percentage context (upstream lightningcss defect; see vize issue #3295)";
        assert.deepEqual(row.parsed, { errors: [error], warnings: [] });
        assert.deepEqual(row.compiled, {
          code: input.source,
          cssVars: [],
          errors: [error],
          warnings: [],
        });
        assert.equal(row.printed, null);
        report.atelierGuard += 1;
      } else {
        assert.deepEqual(row.parsed.errors, []);
        assert.deepEqual(row.parsed.warnings, []);
        assert(row.parsed.ast && typeof row.parsed.ast === "object");
        report.atelierHealthyShape += 1; // Original27 laws, no invented historical full-AST parity.
      }
      assert.deepEqual(row.subsequentHealthy, finite.subsequentHealthy);
      report.subsequentHealthyWhole += 1;
    });
    if (resultGroup === "atelier") report.atelierOriginal += 1;
  }
}
for (const neighbor of finite.historicalFixedCases) {
  const rows = observed.historicalFixedNeighbors.filter(
    (row: { historicalCase: string }) => row.historicalCase === neighbor.historicalCase,
  );
  assert.equal(rows.length, 3);
  for (const [i, row] of rows.entries())
    check(`fixed/${neighbor.historicalCase}/${i + 1}`, () => {
      assert.equal(row.pass, i + 1);
      assert.equal(row.source, neighbor.source);
      assert.deepEqual(row.options, neighbor.options);
      assert.deepEqual(row.actual, neighbor.wholeExpected);
      report.historicalFixedNeighbors += 1;
    });
}
assert.equal(observed.historicalFixedNeighbors.length, 9);
assertOriginalCLIPlan(
  cli.attempts.map((row: { id: string }) => row.id),
  corpus.formatterCases,
  corpus.linterCases,
);
for (const row of cli.attempts) {
  const [surface, id, pass] = row.id.split("/");
  const entries = surface === "formatter" ? corpus.formatterCases : corpus.linterCases;
  const entry = entries.find((e: { id: string }) => e.id === id);
  check(row.id, () => {
    assert(entry);
    if (surface === "formatter")
      assertWholeFormatterCell(
        row,
        entry,
        Number(pass),
        finiteSuccessor(surface, entry)?.wholeCode ?? entry.expected,
      );
    else
      assertWholeLinterCell(
        row,
        entry,
        Number(pass),
        baseline.runs.find((r: { id: string }) => r.id === id).stdout,
      );
  });
  report.cliOriginal += 1;
  if (finite.cliCells.some((cell: { id: string }) => cell.id === row.id)) report.cliFinite += 1;
}
assert.equal(new Set(cli.attempts.map((r: { id: string }) => r.id)).size, 90);
assert.deepEqual(
  [
    report.rustOriginal,
    report.rustFinite,
    report.cliOriginal,
    report.cliFinite,
    report.atelierOriginal,
    report.atelierNamedWhole,
    report.atelierGuard,
    report.atelierHealthyShape,
    report.additionalWhole,
    report.subsequentHealthyWhole,
    report.historicalFixedNeighbors,
  ],
  [99, 33, 90, 26, 66, 14, 25, 27, 6, 72, 9],
);
report.completed = report.errors.length === 0;
persist();
assert.deepEqual(report.errors, [], "complete finite contract errors remain in the artifact");
console.log(JSON.stringify(report));
