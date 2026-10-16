import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  loadVOnModifierCases,
  validateVOnModifierCases,
} from "./support/n8n-cli-config-valid-v-on-oracle.mjs";
import { assertSemanticOracles } from "./support/n8n-cli-config-semantic-oracle.mjs";
import { sha256 } from "./support/n8n-cli-config-workspace.mjs";

const fixture = loadVOnModifierCases();
const captures = () =>
  fixture.oracleCapture.map(({ caseId, observations }) => ({
    caseId,
    observations: observations.map((recorded) => ({
      raw: {},
      recorded: structuredClone(recorded),
    })),
  }));
const check = (actual) =>
  assertSemanticOracles(actual, fixture.oracleCapture, fixture, validateVOnModifierCases);

void test("valid-v-on source CLI expectations retain independent identities, severities and coordinates", () => {
  validateVOnModifierCases();
  check(captures());
  const coordinates = ({ ruleId, severity, line, column, endLine, endColumn }) => ({
    ruleId,
    severity,
    line,
    column,
    endLine,
    endColumn,
  });
  for (const sourceCase of fixture.cases) {
    const oracle = fixture.oracleCapture.find(({ caseId }) => caseId === sourceCase.id);
    assert.deepEqual(
      sourceCase.cliExpectations.flatMap(({ messages }) => messages.map(coordinates)),
      oracle.observations[0].captures.flatMap(({ packets }) =>
        packets.flatMap(({ messages }) => messages.map(coordinates)),
      ),
      sourceCase.id,
    );
  }
});

void test("custom modifier controls keep both configured acceptance and reset rejection under all51", () => {
  for (const [id, expected, modifiers] of [
    ["valid-on--custom-allowed", [], ["custom"]],
    ["valid-on--custom-sync", [], ["sync"]],
    ["valid-on--custom-reset", ["vue/valid-v-on"], []],
    ["valid-on--custom-missing-handler", ["vue/valid-v-on"], ["custom"]],
  ]) {
    const sourceCase = fixture.cases.find((entry) => entry.id === id);
    assert.deepEqual(sourceCase.expectedOracleRuleIds, expected);
    const oracle = fixture.oracleCapture.find(({ caseId }) => caseId === id);
    for (const recorded of oracle.observations) {
      assert.equal(Object.keys(recorded.configurations[0].rules).length, 51);
      assert.deepEqual(recorded.configurations[0].rules["vue/valid-v-on"], [
        "error",
        { modifiers },
      ]);
    }
  }
});

void test("new v-on whole packets reject unknown fields, dropped findings, provider failures and omitted repeats", () => {
  for (const mutate of [
    (actual) => {
      actual[0].observations[0].recorded.captures[0].packets[0].foreign = true;
    },
    (actual) => {
      actual[0].observations[0].recorded.captures[0].packets[0].messages.pop();
    },
    (actual) => {
      actual[0].observations[0].raw.loadError = { message: "provider failed" };
    },
    (actual) => {
      actual[0].observations.pop();
    },
  ]) {
    const actual = captures();
    mutate(actual);
    assert.throws(() => check(actual));
  }
});

void test("existing fifteen deprecated-modifier controls remain byte-identical", () => {
  const original = fs.readFileSync(
    new URL(
      "../_fixtures/differential/lint/n8n-cli-config/deprecated-modifiers.json",
      import.meta.url,
    ),
  );
  assert.equal(
    sha256(original),
    "59c8598deac387c37d9589e1c6ba262047e862326dcc494af3d24c442066e859",
  );
});
