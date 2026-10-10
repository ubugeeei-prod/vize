import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  expectedScopedPacket,
  projection,
  scopedConfig,
  validateProjection,
} from "./support/n8n-cli-config-inputs.mjs";
import { assertAuthoredOracle, loadAuthoredCases } from "./support/n8n-cli-config-oracle.mjs";

void test("compiler fixture-only and runtime-only changes trigger the full n8n custody workflow", () => {
  const root = new URL("../../", import.meta.url);
  const workflow = fs.readFileSync(new URL(".github/workflows/n8n-adoption.yml", root), "utf8");
  const block = workflow.match(/^  pull_request:\n    paths:\n([\s\S]*?)^  merge_group:/mu);
  assert.notEqual(block, null, "the actual workflow retains pull-request path filters");
  const filters = block[1]
    .split("\n")
    .filter((line) => line.startsWith("      - "))
    .map((line) => line.slice(8));
  const directories = [
    "tests/_fixtures/differential/compiler/legacy-slot-binding-handoff-8142/",
    "tests/_fixtures/differential/compiler/slot-parameter-entities-8142/",
  ];
  const inputs = directories.flatMap((directory) =>
    fs
      .readdirSync(new URL(directory, root), { recursive: true })
      .filter((entry) => fs.statSync(new URL(directory + entry, root)).isFile())
      .map((entry) => directory + entry),
  );
  inputs.push("tests/tooling/support/slot-parameter-entity-runtime.mjs");
  const matches = (file) => filters.some((filter) => path.posix.matchesGlob(file, filter));
  assert.deepEqual(
    inputs.map((file) => [file, matches(file)]),
    inputs.map((file) => [file, true]),
  );
  assert.equal(matches("tests/_fixtures/differential/compiler/unrelated/control.json"), false);
});

void test("latest n8n CLI requirements retain all 51 oracle identities without status filtering", () => {
  assert.equal(validateProjection(), projection);
  const missing = structuredClone(projection);
  delete missing.oracleRules["vue/require-component-registration"];
  assert.throws(() => validateProjection(missing));
  const wrong = structuredClone(projection);
  wrong.oracleRules["vue/require-component-registration"] = "vue/require-component-registration";
  assert.throws(() => validateProjection(wrong));
});

void test("editor warning scopes retain explicit options and do not become file exclusions", () => {
  const editor = "packages/frontend/editor-ui";
  const scoped = scopedConfig(editor, true);
  assert.equal(scoped.linter.rules["vue/attribute-hyphenation"], "warn");
  assert.equal(scoped.linter.ruleOptions["vue/attribute-hyphenation"], "always");
  assert.equal(scoped.entries[0].files.length, 5);
  assert.deepEqual(scopedConfig(editor, false), { linter: projection.linter });
  assert.deepEqual(scopedConfig("packages/modules/mcp/frontend", true), {
    linter: projection.linter,
  });
});

void test("scope packet oracle changes only selected findings and keeps complete foreign fields", () => {
  const editor = "packages/frontend/editor-ui";
  const packet = {
    file: projection.scopes[editor].entries[0].files[0],
    messages: [
      { ruleId: "vue/no-multiple-template-root", severity: 2, unknown: { original: true } },
      { ruleId: "vue/attribute-hyphenation", severity: 2, line: 4, column: 8, fix: "whole" },
      { ruleId: "vue/no-v-html", severity: 2, foreign: ["repeated", "repeated"] },
    ],
    errorCount: 3,
    warningCount: 0,
    unknownEnvelope: { original: [1, 2] },
  };
  assert.deepEqual(expectedScopedPacket(packet, editor), {
    ...packet,
    messages: [{ ...packet.messages[1], severity: 1 }, packet.messages[2]],
    errorCount: 1,
    warningCount: 1,
  });
  const neighbor = { ...packet, file: packet.file + ".neighbor.vue" };
  const expected = expectedScopedPacket(neighbor, editor);
  assert.equal(expected.messages.length, 3);
  assert.deepEqual(expected.messages[0], packet.messages[0]);
  assert.deepEqual(expected.messages[2], packet.messages[2]);
  assert.deepEqual(packet.messages[1].severity, 2);
});

void test("whole independent oracle comparison refuses changed ranges, envelopes and missing findings", () => {
  const expected = loadAuthoredCases().oracleCapture;
  const capture = () => ({ raw: { captures: [] }, recorded: structuredClone(expected) });
  assertAuthoredOracle(capture());
  const mutations = [
    (value) => {
      value.captures[0].packets[0].messages[0].endColumn += 1;
    },
    (value) => {
      value.captures[0].packets[0].foreignEnvelope = true;
    },
    (value) => {
      value.captures[0].packets[0].messages.push(value.captures[0].packets[0].messages[0]);
    },
    (value) => {
      value.captures.pop();
    },
  ];
  for (const mutate of mutations) {
    const changed = capture();
    mutate(changed.recorded);
    assert.throws(() => assertAuthoredOracle(changed));
  }
});

void test("all independent calls require an entire CLI envelope law and retain provider errors", () => {
  const fixture = loadAuthoredCases();
  assert.equal(fixture.cliExpectations.length, 18);
  assert.deepEqual(
    fixture.cliExpectations.map(({ configuration, caseId }) => [configuration, caseId]),
    fixture.oracleCapture.captures.map(({ configuration, caseId }) => [configuration, caseId]),
  );
  for (const { packets } of fixture.cliExpectations) {
    assert.equal(packets.length, 1);
    assert.deepEqual(Object.keys(packets[0]).sort(), [
      "errorCount",
      "file",
      "messages",
      "warningCount",
    ]);
    for (const message of packets[0].messages) {
      assert.deepEqual(Object.keys(message).sort(), [
        "column",
        "endColumn",
        "endLine",
        ...(message.ruleId === "vue/sfc-element-order" ? ["help"] : []),
        "line",
        "message",
        "ruleDocsPath",
        "ruleId",
        "severity",
      ]);
      if (message.ruleId === "vue/sfc-element-order")
        assert.match(message.help, /^Recommended order: </u);
    }
  }
  const failure = {
    raw: { loadError: { name: "ConfigError", message: "foreign provider failure" } },
    recorded: structuredClone(fixture.oracleCapture),
  };
  assert.throws(() => assertAuthoredOracle(failure));
  assert.deepEqual(failure.raw.loadError, {
    name: "ConfigError",
    message: "foreign provider failure",
  });
});
