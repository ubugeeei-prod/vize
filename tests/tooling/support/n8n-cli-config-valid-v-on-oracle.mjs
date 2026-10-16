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

export const vOnModifierCasesPath = path.join(
  root,
  "tests/_fixtures/differential/lint/n8n-cli-config/valid-v-on-modifiers.json",
);
export const loadVOnModifierCases = () => JSON.parse(fs.readFileSync(vOnModifierCasesPath, "utf8"));

export function validateVOnModifierCases(fixture = loadVOnModifierCases()) {
  assert.equal(fixture.schema, "vize.n8n.cli-valid-v-on-modifiers");
  assert.equal(fixture.version, 1);
  assert.equal(fixture.cases.length, 18);
  assert.equal(new Set(fixture.cases.map(({ id }) => id)).size, 18);
  assert.equal(
    fixture.oracleFilePrefix,
    "tests/_fixtures/differential/lint/n8n-cli-config/valid-v-on-modifiers",
  );
  for (const sourceCase of fixture.cases) {
    assert.match(sourceCase.id, /^[a-z0-9-]+$/u);
    assert.equal(sha256(sourceCase.source), sourceCase.sourceSha256, sourceCase.id);
    assert.equal(sourceCase.scriptless, !sourceCase.source.includes("<script"), sourceCase.id);
    assert.equal(sourceCase.compareCoordinates, true, sourceCase.id);
    assert.ok(sourceCase.expectedOracleRuleIds.every((id) => id === "vue/valid-v-on"));
    assert.equal(sourceCase.cliExpectations.length, 1, sourceCase.id);
    assert.equal(sourceCase.cliExpectations[0].file, sourceCase.id + ".vue", sourceCase.id);
    assert.ok(Object.keys(sourceCase.ruleOptions).every((id) => id === "vue/valid-v-on"));
  }
  assert.ok(fixture.cases.some(({ expectedOracleRuleIds }) => expectedOracleRuleIds.length === 0));
  assert.ok(fixture.cases.some(({ expectedOracleRuleIds }) => expectedOracleRuleIds.length > 0));
  assert.equal(Object.keys(projection.linter.rules).length, 51);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const valueAfter = (flag) => (args.includes(flag) ? args[args.indexOf(flag) + 1] : undefined);
  const output =
    valueAfter("--receipt-dir") ?? fs.mkdtempSync(path.join(os.tmpdir(), "vize-n8n-valid-v-on-"));
  const fixture = loadVOnModifierCases();
  const captures = await captureSemanticOracles({
    fixture,
    validateFixture: validateVOnModifierCases,
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
    assertSemanticOracles(captures, recorded, fixture, validateVOnModifierCases);
    fixture.oracleCapture = recorded;
    writeJson(vOnModifierCasesPath, fixture);
  }
  assertSemanticOracles(captures, fixture.oracleCapture, fixture, validateVOnModifierCases);
  console.log(`Whole valid-v-on-modifier provider cohort passed; raw receipts: ${output}`);
}
