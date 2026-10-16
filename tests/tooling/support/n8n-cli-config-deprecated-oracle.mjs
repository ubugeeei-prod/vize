// Independent extra cohort; the original 28-case/public-install authority is unchanged.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { root, projection } from "./n8n-cli-config-inputs.mjs";
import { sha256, writeJson } from "./n8n-cli-config-workspace.mjs";
import {
  captureSemanticOracles,
  assertSemanticOracles,
} from "./n8n-cli-config-semantic-oracle.mjs";

export const deprecatedCasesPath = path.join(
  root,
  "tests/_fixtures/differential/lint/n8n-cli-config/deprecated-modifiers.json",
);
export const loadDeprecatedCases = () => JSON.parse(fs.readFileSync(deprecatedCasesPath, "utf8"));

export function validateDeprecatedCases(fixture = loadDeprecatedCases()) {
  assert.equal(fixture.schema, "vize.n8n.cli-deprecated-modifiers");
  assert.equal(fixture.version, 1);
  assert.equal(fixture.cases.length, 15);
  assert.equal(new Set(fixture.cases.map(({ id }) => id)).size, 15);
  assert.equal(
    fixture.oracleFilePrefix,
    "tests/_fixtures/differential/lint/n8n-cli-config/deprecated-modifiers",
  );
  for (const family of ["sync", "native", "numeric"]) {
    const cases = fixture.cases.filter((entry) => entry.family === family);
    assert.equal(cases.length, 5, family);
    assert.ok(cases.some(({ expectedOracleRuleIds }) => expectedOracleRuleIds.length === 0));
    assert.ok(cases.some(({ expectedOracleRuleIds }) => expectedOracleRuleIds.length > 0));
  }
  for (const sourceCase of fixture.cases) {
    assert.match(sourceCase.id, /^[a-z0-9-]+$/u);
    assert.equal(sha256(sourceCase.source), sourceCase.sourceSha256, sourceCase.id);
    assert.equal(sourceCase.scriptless, !sourceCase.source.includes("<script"), sourceCase.id);
    assert.equal(sourceCase.compareCoordinates, true, sourceCase.id);
    assert.deepEqual(sourceCase.ruleOptions, {}, sourceCase.id);
    assert.ok(
      sourceCase.expectedOracleRuleIds.every((id) =>
        Object.values(projection.oracleRules).includes(id),
      ),
    );
    assert.equal(sourceCase.cliExpectations.length, 1, sourceCase.id);
    assert.equal(sourceCase.cliExpectations[0].file, sourceCase.id + ".vue", sourceCase.id);
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const valueAfter = (flag) => (args.includes(flag) ? args[args.indexOf(flag) + 1] : undefined);
  const output =
    valueAfter("--receipt-dir") ?? fs.mkdtempSync(path.join(os.tmpdir(), "vize-n8n-deprecated-"));
  const fixture = loadDeprecatedCases();
  const captures = await captureSemanticOracles({
    fixture,
    validateFixture: validateDeprecatedCases,
    benchmarkManifest: valueAfter("--benchmark-manifest"),
    onRecorded(packet) {
      writeJson(path.join(output, `${packet.caseId}-${packet.iteration}.json`), packet);
    },
  });
  writeJson(path.join(output, "complete.json"), captures);
  if (args.includes("--record")) {
    // Record provider-owned complete packets only after both repeated calls and
    // the independently authored finding identities pass, including foreign findings.
    const recorded = captures.map(({ caseId, observations }) => ({
      caseId,
      observations: observations.map(({ recorded }) => recorded),
    }));
    assertSemanticOracles(captures, recorded, fixture, validateDeprecatedCases);
    fixture.oracleCapture = recorded;
    writeJson(deprecatedCasesPath, fixture);
  }
  assertSemanticOracles(captures, fixture.oracleCapture, fixture, validateDeprecatedCases);
  console.log(`Whole deprecated-modifier provider cohort passed; raw receipts: ${output}`);
}
