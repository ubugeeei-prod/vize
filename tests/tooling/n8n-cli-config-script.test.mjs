import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  loadScriptRuleCases,
  loadScriptRuleResearch,
  selectedOracleRules,
  validateScriptRuleCases,
} from "./support/n8n-cli-config-script-oracle.mjs";
import { assertSemanticOracles } from "./support/n8n-cli-config-semantic-oracle.mjs";
import { projection } from "./support/n8n-cli-config-inputs.mjs";
import { sha256 } from "./support/n8n-cli-config-workspace.mjs";
import { runScriptRuleAcceptance } from "./support/n8n-cli-config-script-rules-acceptance.mjs";

const fixtures = [loadScriptRuleCases(), loadScriptRuleResearch()];
const captures = (fixture) =>
  fixture.oracleCapture.map(({ caseId, observations }) => ({
    caseId,
    observations: observations.map((recorded) => ({
      raw: {},
      recorded: structuredClone(recorded),
    })),
  }));
const check = (fixture, actual = captures(fixture)) =>
  assertSemanticOracles(actual, fixture.oracleCapture, fixture, validateScriptRuleCases);

void test("script replay refuses an existing receipt directory before touching retained evidence", async () => {
  const output = fs.mkdtempSync(path.join(os.tmpdir(), "vize-script-custody-"));
  const sentinel = path.join(output, "failed-packet.json");
  const evidence = '{"failure":"retain exactly"}\n';
  fs.writeFileSync(sentinel, evidence);
  try {
    await assert.rejects(runScriptRuleAcceptance({ output }), /refuse to overwrite retained/);
    assert.deepEqual(fs.readdirSync(output), ["failed-packet.json"]);
    assert.equal(fs.readFileSync(sentinel, "utf8"), evidence);
  } finally {
    fs.rmSync(output, { recursive: true });
  }
});

void test("declared sixteen script controls and six counterexamples retain complete repeated full51 packets", () => {
  const inverse = Object.fromEntries(
    Object.entries(projection.oracleRules).map(([rule, oracle]) => [oracle, rule]),
  );
  const coordinates = ({ ruleId, severity, line, column, endLine, endColumn }) => ({
    ruleId: inverse[ruleId] ?? ruleId,
    severity,
    line,
    column,
    endLine,
    endColumn,
  });
  for (const fixture of fixtures) {
    check(fixture);
    for (const sourceCase of fixture.cases) {
      const oracle = fixture.oracleCapture.find(({ caseId }) => caseId === sourceCase.id);
      assert.deepEqual(
        sourceCase.cliExpectations.flatMap(({ messages }) => messages.map(coordinates)),
        oracle.observations[0].captures.flatMap(({ packets }) =>
          packets.flatMap(({ messages }) => messages.map(coordinates)),
        ),
        sourceCase.id + ": authored target coordinates, not native execution evidence",
      );
      for (const recorded of oracle.observations) {
        assert.equal(Object.keys(recorded.configurations[0].rules).length, 51);
        for (const rule of selectedOracleRules)
          assert.equal(recorded.configurations[0].rules[rule], "error");
        assert.deepEqual(recorded.configurations[0].ruleOptions, projection.linter.ruleOptions);
      }
    }
  }
});

void test("each suppression retains the whole independently reported suppressed finding", () => {
  const fixture = fixtures[0];
  for (const sourceCase of fixture.cases.filter(({ control }) => control === "suppression")) {
    const oracle = fixture.oracleCapture.find(({ caseId }) => caseId === sourceCase.id);
    for (const recorded of oracle.observations) {
      const packet = recorded.captures[0].packets[0];
      assert.deepEqual(packet.messages, []);
      assert.equal(packet.suppressedMessages.length, 1);
      assert.ok(selectedOracleRules.includes(packet.suppressedMessages[0].ruleId));
      assert.equal(packet.suppressedMessages[0].suppressions[0].kind, "directive");
    }
  }
});

void test("whole script packets refuse foreign fields, lost suppression, missing findings, provider failure and repeat omission", () => {
  const fixture = fixtures[0];
  for (const mutate of [
    (actual) => {
      actual[0].observations[0].recorded.captures[0].packets[0].foreign = true;
    },
    (actual) => {
      actual[0].observations[0].recorded.captures[0].packets[0].messages.pop();
    },
    (actual) => {
      actual[2].observations[0].recorded.captures[0].packets[0].suppressedMessages.pop();
    },
    (actual) => {
      actual[0].observations[0].raw.loadError = { message: "provider failed" };
    },
    (actual) => {
      actual[0].observations.pop();
    },
  ]) {
    const actual = captures(fixture);
    mutate(actual);
    assert.throws(() => check(fixture, actual));
  }
});

void test("all previous full51 authored cohorts remain byte-identical", () => {
  for (const [name, expected] of [
    ["cases", "6d208d7fbee622d41364ba0364a1d6190f22d39c9dcec956681d618ec7cb903a"],
    ["semantic-routing", "0391fe3875f7b6fea962b8fc5b16a23ad553b526d9192eb9f4a4f31b25f5543c"],
    ["deprecated-modifiers", "59c8598deac387c37d9589e1c6ba262047e862326dcc494af3d24c442066e859"],
    ["valid-v-on-modifiers", "d11401747a9efcccee2f4d6490da0d2fe86c269efba0d006f76a01f8c9b09550"],
  ]) {
    assert.equal(
      sha256(
        fs.readFileSync(
          new URL(`../_fixtures/differential/lint/n8n-cli-config/${name}.json`, import.meta.url),
        ),
      ),
      expected,
      name,
    );
  }
});
