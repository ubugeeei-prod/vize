// Independent whole-packet oracle for the bounded source-CLI routing cohort.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { root, projection } from "./n8n-cli-config-inputs.mjs";
import { captureAuthoredOracle, assertAuthoredOracle } from "./n8n-cli-config-oracle.mjs";
import { sha256, writeJson } from "./n8n-cli-config-workspace.mjs";

export const semanticCasesPath = path.join(
  root,
  "tests/_fixtures/differential/lint/n8n-cli-config/semantic-routing.json",
);
export const loadSemanticCases = () => JSON.parse(fs.readFileSync(semanticCasesPath, "utf8"));

function oneCaseFixture(sourceCase) {
  return { cases: [sourceCase], optionControls: [] };
}

export function validateSemanticCases(fixture = loadSemanticCases()) {
  assert.equal(fixture.schema, "vize.n8n.cli-semantic-routing");
  assert.equal(fixture.cases.length, 28);
  assert.equal(new Set(fixture.cases.map(({ id }) => id)).size, 28);
  assert.deepEqual(
    [...new Set(fixture.cases.map(({ family }) => family))].sort((a, b) => a.localeCompare(b)),
    ["casing", "content", "key", "ref"],
  );
  for (const sourceCase of fixture.cases) {
    assert.match(sourceCase.id, /^[a-z0-9-]+$/u);
    assert.equal(sha256(sourceCase.source), sourceCase.sourceSha256, sourceCase.id);
    assert.equal(sourceCase.scriptless, !sourceCase.source.includes("<script"), sourceCase.id);
    assert.ok(Array.isArray(sourceCase.expectedOracleRuleIds), sourceCase.id);
    assert.equal(sourceCase.cliExpectations.length, 1, sourceCase.id);
    assert.equal(sourceCase.cliExpectations[0].file, sourceCase.id + ".vue", sourceCase.id);
  }
}

export async function captureSemanticOracles({
  benchmarkManifest,
  onRecorded = async () => {},
} = {}) {
  const fixture = loadSemanticCases();
  validateSemanticCases(fixture);
  const cases = [];
  for (const sourceCase of fixture.cases) {
    const observations = [];
    for (let iteration = 0; iteration < 2; iteration++) {
      const capture = await captureAuthoredOracle({
        benchmarkManifest,
        fixture: oneCaseFixture(sourceCase),
        baseRuleOptions: { ...projection.linter.ruleOptions, ...sourceCase.ruleOptions },
        filePrefix: "tests/_fixtures/differential/lint/n8n-cli-config/semantic-routing",
        officialVueBase: true,
        async onRecorded(packet) {
          await onRecorded({ caseId: sourceCase.id, iteration, packet });
        },
      });
      observations.push(capture);
    }
    cases.push({ caseId: sourceCase.id, observations });
  }
  return cases;
}

export function assertSemanticOracles(captures, expected = loadSemanticCases().oracleCapture) {
  const fixture = loadSemanticCases();
  validateSemanticCases(fixture);
  assert.equal(captures.length, fixture.cases.length);
  for (const sourceCase of fixture.cases) {
    const actual = captures.find(({ caseId }) => caseId === sourceCase.id);
    const golden = expected?.find(({ caseId }) => caseId === sourceCase.id);
    assert.ok(actual && golden, sourceCase.id + ": complete provider custody is required");
    assert.equal(actual.observations.length, 2, sourceCase.id);
    for (let iteration = 0; iteration < 2; iteration++) {
      assert.deepEqual(
        actual.observations[iteration].recorded.officialVueBase,
        {
          export: "flat/base",
          rules: { "vue/comment-directive": "error", "vue/jsx-uses-vars": "error" },
          processors: ["vue/vue"],
        },
        sourceCase.id + ": actual official Vue infrastructure",
      );
      assertAuthoredOracle(
        actual.observations[iteration],
        golden.observations[iteration],
        oneCaseFixture(sourceCase),
      );
    }
    assert.deepEqual(
      actual.observations[0].recorded,
      actual.observations[1].recorded,
      sourceCase.id + ": repeated whole provider packets",
    );
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const valueAfter = (flag) => (args.includes(flag) ? args[args.indexOf(flag) + 1] : undefined);
  const output =
    valueAfter("--receipt-dir") ??
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-n8n-semantic-oracle-"));
  const captures = await captureSemanticOracles({
    benchmarkManifest: valueAfter("--benchmark-manifest"),
    onRecorded(packet) {
      writeJson(path.join(output, `${packet.caseId}-${packet.iteration}.json`), packet);
    },
  });
  writeJson(path.join(output, "complete.json"), captures);
  if (args.includes("--record")) {
    // This dev-only provider recorder never authenticates or executes a CLI.
    // Whole raw failures already exist; pins, all foreign findings and repeats
    // must satisfy the authored expectations before any golden is replaced.
    const recorded = captures.map(({ caseId, observations }) => ({
      caseId,
      observations: observations.map(({ recorded }) => recorded),
    }));
    assertSemanticOracles(captures, recorded);
    const fixture = loadSemanticCases();
    fixture.oracleCapture = recorded;
    writeJson(semanticCasesPath, fixture);
  }
  assertSemanticOracles(captures);
  console.log(`Whole semantic provider cohort passed; raw receipts: ${output}`);
}
