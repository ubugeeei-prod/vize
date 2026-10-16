// Independently authored script controls. Recording providers grants no native execution credit.
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

export const scriptRuleCasesPath = path.join(
  root,
  "tests/_fixtures/differential/lint/n8n-cli-config/script-rule-controls.json",
);
export const loadScriptRuleCases = () => JSON.parse(fs.readFileSync(scriptRuleCasesPath, "utf8"));

export const scriptRuleResearchPath = path.join(
  root,
  "tests/_fixtures/differential/lint/n8n-cli-config/script-rule-research.json",
);
export const loadScriptRuleResearch = () =>
  JSON.parse(fs.readFileSync(scriptRuleResearchPath, "utf8"));
export const selectedOracleRules = [
  "vue/no-deprecated-dollar-listeners-api",
  "vue/no-deprecated-dollar-scopedslots-api",
  "vue/no-use-computed-property-like-method",
  "vue/require-valid-default-prop",
];

export function validateScriptRuleCases(fixture = loadScriptRuleCases()) {
  assert.ok(
    ["vize.n8n.cli-script-rule-controls", "vize.n8n.cli-script-rule-research"].includes(
      fixture.schema,
    ),
  );
  const research = fixture.schema.endsWith("research");
  const count = research ? 6 : 16;
  assert.equal(fixture.version, 1);
  assert.equal(fixture.cases.length, count);
  assert.equal(new Set(fixture.cases.map(({ id }) => id)).size, count);
  assert.deepEqual(
    fixture.cases.map(({ id }) => id),
    research
      ? [
          "script-listeners--ordinary-object",
          "script-listeners--template-instance",
          "script-scopedslots--ordinary-object",
          "script-scopedslots--template-instance",
          "script-computed--template-call",
          "script-default--ordinary-object",
        ]
      : ["listeners", "scopedslots", "computed", "default"].flatMap((family) =>
          ["positive", "clean", "suppression", "noncode"].map(
            (control) => `script-${family}--${control}`,
          ),
        ),
  );
  assert.equal(
    fixture.oracleFilePrefix,
    "tests/_fixtures/differential/lint/n8n-cli-config/" +
      (research ? "script-rule-research" : "script-rule-controls"),
  );
  for (const sourceCase of fixture.cases) {
    assert.match(sourceCase.id, /^[a-z0-9-]+$/u);
    assert.equal(sha256(sourceCase.source), sourceCase.sourceSha256, sourceCase.id);
    assert.equal(sourceCase.scriptless, !sourceCase.source.includes("<script"), sourceCase.id);
    assert.equal(sourceCase.compareCoordinates, true, sourceCase.id);
    assert.equal(sourceCase.scriptless, false, sourceCase.id);
    assert.ok(sourceCase.expectedOracleRuleIds.every((id) => selectedOracleRules.includes(id)));
    assert.equal(sourceCase.cliExpectations.length, 1, sourceCase.id);
    assert.equal(sourceCase.cliExpectations[0].file, sourceCase.id + ".vue", sourceCase.id);
    assert.deepEqual(sourceCase.ruleOptions, {});
  }
  assert.ok(fixture.cases.some(({ expectedOracleRuleIds }) => expectedOracleRuleIds.length === 0));
  assert.ok(fixture.cases.some(({ expectedOracleRuleIds }) => expectedOracleRuleIds.length > 0));
  assert.equal(Object.keys(projection.linter.rules).length, 51);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const valueAfter = (flag) => (args.includes(flag) ? args[args.indexOf(flag) + 1] : undefined);
  const output =
    valueAfter("--receipt-dir") ?? fs.mkdtempSync(path.join(os.tmpdir(), "vize-n8n-script-rules-"));
  const fixturePath = args.includes("--research") ? scriptRuleResearchPath : scriptRuleCasesPath;
  const fixture = JSON.parse(fs.readFileSync(fixturePath, "utf8"));
  const captures = await captureSemanticOracles({
    fixture,
    validateFixture: validateScriptRuleCases,
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
    assertSemanticOracles(captures, recorded, fixture, validateScriptRuleCases);
    fixture.oracleCapture = recorded;
    writeJson(fixturePath, fixture);
  }
  assertSemanticOracles(captures, fixture.oracleCapture, fixture, validateScriptRuleCases);
  console.log(`Whole script-rule provider cohort passed; raw receipts: ${output}`);
}
