import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  loadDeprecatedCases,
  validateDeprecatedCases,
} from "./support/n8n-cli-config-deprecated-oracle.mjs";
import { assertSemanticOracles } from "./support/n8n-cli-config-semantic-oracle.mjs";
import { sha256 } from "./support/n8n-cli-config-workspace.mjs";

const fixture = loadDeprecatedCases();
const captures = () =>
  fixture.oracleCapture.map(({ caseId, observations }) => ({
    caseId,
    observations: observations.map((recorded) => ({
      raw: {},
      recorded: structuredClone(recorded),
    })),
  }));
const check = (actual) =>
  assertSemanticOracles(actual, fixture.oracleCapture, fixture, validateDeprecatedCases);

void test("deprecated modifier cohort retains all51, clean controls and whole repeated provider packets", () => {
  validateDeprecatedCases();
  check(captures());
  for (const sourceCase of fixture.cases) {
    const oracle = fixture.oracleCapture.find(({ caseId }) => caseId === sourceCase.id);
    for (const recorded of oracle.observations) {
      assert.equal(Object.keys(recorded.configurations[0].rules).length, 51);
      const packet = recorded.captures[0].packets[0];
      assert.equal(packet.source ?? sourceCase.source, sourceCase.source);
      assert.ok(Array.isArray(packet.suppressedMessages));
    }
  }
  const foreign = fixture.cases.filter(({ expectedOracleRuleIds }) =>
    expectedOracleRuleIds.includes("vue/valid-v-bind"),
  );
  assert.equal(foreign.length, 2, "unsupported bind modifiers remain whole foreign findings");
});

void test("whole deprecated modifier evidence refuses foreign fields, altered fixes and missing repeats", () => {
  const mutations = [
    (actual) => {
      actual[0].observations[0].raw.loadError = { message: "provider failed" };
    },
    (actual) => {
      actual[0].observations[0].recorded.captures[0].packets[0].foreign = true;
    },
    (actual) => {
      actual[0].observations[0].recorded.captures[0].packets[0].messages[0].endColumn++;
    },
    (actual) => {
      const numeric = actual.find(({ caseId }) => caseId === "deprecated-numeric--shorthand");
      numeric.observations[0].recorded.captures[0].packets[0].messages[0].fix.text = "changed";
    },
    (actual) => {
      actual[0].observations.pop();
    },
  ];
  for (const mutate of mutations) {
    const actual = captures();
    mutate(actual);
    assert.throws(() => check(actual));
  }
});

void test("deprecated cases cannot discard their clean pair or independent coordinate checks", () => {
  for (const mutate of [
    (value) => {
      value.cases.pop();
    },
    (value) => {
      value.cases[0].source += " ";
    },
    (value) => {
      value.cases[0].compareCoordinates = false;
    },
  ]) {
    const value = structuredClone(fixture);
    mutate(value);
    assert.throws(() => validateDeprecatedCases(value));
  }
});

void test("original 28-case authority and discovered incompatible packets remain distinct", () => {
  const original = fs.readFileSync(
    new URL("../_fixtures/differential/lint/n8n-cli-config/semantic-routing.json", import.meta.url),
  );
  assert.equal(
    sha256(original),
    "0391fe3875f7b6fea962b8fc5b16a23ad553b526d9192eb9f4a4f31b25f5543c",
  );
  const research = JSON.parse(
    fs.readFileSync(
      new URL(
        "../_fixtures/differential/lint/n8n-cli-config/modifier-unqualified-research.json",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  assert.match(research.qualification, /Unqualified local research/u);
  assert.equal(research.cases.length, 3);
  for (const { input, provider, native } of research.cases) {
    assert.equal(sha256(input.source), input.sourceSha256);
    assert.equal(provider.observations.length, 2);
    assert.equal(native.length, 2);
    assert.equal(
      fixture.cases.some(({ id }) => id === input.id),
      false,
    );
    for (const observation of native) {
      assert.deepEqual(JSON.parse(observation.stdout), observation.packets);
    }
    const ids = (packets) =>
      packets.flatMap(({ messages }) => messages.map(({ ruleId }) => ruleId));
    assert.notDeepEqual(
      ids(native[0].packets),
      ids(provider.observations[0].recorded.captures[0].packets),
    );
  }
});
