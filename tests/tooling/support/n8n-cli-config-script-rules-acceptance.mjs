// Execute both declared cohorts before refusing any mismatch. No native status filter.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runSemanticAcceptance } from "./n8n-cli-config-semantic-acceptance.mjs";
import { root } from "./n8n-cli-config-inputs.mjs";
import { errorPacket } from "./n8n-cli-config-oracle.mjs";
import { writeJson } from "./n8n-cli-config-workspace.mjs";
import {
  loadScriptRuleCases,
  loadScriptRuleResearch,
  validateScriptRuleCases,
} from "./n8n-cli-config-script-oracle.mjs";

export async function runScriptRuleAcceptance({ output }) {
  assert.equal(fs.existsSync(output), false, "refuse to overwrite retained script-rule receipts");
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.mkdirSync(output);
  const outcomes = [];
  const failures = [];
  for (const [cohort, fixture] of [
    ["controls", loadScriptRuleCases()],
    ["research", loadScriptRuleResearch()],
  ]) {
    try {
      await runSemanticAcceptance({
        output: path.join(output, cohort),
        fixture,
        validateFixture: validateScriptRuleCases,
      });
      outcomes.push({ cohort, status: "passed", cases: fixture.cases.length });
    } catch (error) {
      outcomes.push({
        cohort,
        status: "failed",
        cases: fixture.cases.length,
        error: errorPacket(error),
      });
      failures.push(error);
    }
  }
  writeJson(path.join(output, "outcomes.json"), outcomes);
  if (failures.length) {
    throw new AggregateError(
      failures,
      "Declared script controls/research remain unqualified; complete receipts retained",
    );
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await runScriptRuleAcceptance({
    output: path.resolve(process.argv[2] ?? path.join(root, "target/n8n-cli-config/script-rules")),
  });
}
