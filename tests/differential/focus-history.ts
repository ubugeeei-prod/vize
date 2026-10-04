import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./harness.mjs";
import type { ObserverSpec } from "./observer-build.ts";

export const FOCUS_CASES_PATH = "crates/vize_patina/tests/fixtures/focus-history/cases.json";
export const FOCUS_CASES_SHA256 =
  "5ab754f0a882fc9c7f8eb843edf454af7909793debe985c0096daa6b132708ba";
export const FOCUS_OBSERVER: ObserverSpec = {
  product: "linter",
  packageName: "vize_patina",
  exampleName: "focus_history_observer",
  sourcePath: "crates/vize_patina/examples/focus_history_observer/main.rs",
  probes: [["--contract"]],
};
export const FOCUS_CONTRACT = {
  schema: "vize.focus-history-observer",
  version: 1,
  apis: ["--legacy", "--native"],
  entry: "lint_template",
  filename: "test.vue",
  registry: "one-concrete-original-rule",
  options: "original-unspecified",
  output: "Case+RuleIdentity+Observation",
  acceptance: "capture-only",
  fallback: false,
};

export function rawBytes(encoded: string) {
  assert.equal(typeof encoded, "string");
  const bytes = Buffer.from(encoded, "base64");
  assert.equal(bytes.toString("base64"), encoded, "canonical actual stream encoding required");
  return bytes;
}
export function utf8(bytes: Buffer) {
  return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
}
export function validateFocusProbe(receipt: any) {
  assert.equal(receipt.probes.length, 1);
  const [probe] = receipt.probes;
  assert.deepEqual(probe.argv, ["--contract"]);
  assert.equal(probe.exitStatus, 0);
  const bytes = rawBytes(probe.stdoutBase64);
  assert.equal(probe.sha256, sha256(bytes));
  assert.deepEqual(bytes, Buffer.from(`${JSON.stringify(FOCUS_CONTRACT)}\n`));
}

export function loadFocusCases(
  root: string,
  bytes = fs.readFileSync(path.join(root, FOCUS_CASES_PATH)),
) {
  assert.equal(sha256(bytes), FOCUS_CASES_SHA256, "reviewed original witness input drift");
  const fixtures = JSON.parse(utf8(bytes));
  assert.equal(fixtures.length, 8);
  assert.equal(new Set(fixtures.map((fixture: any) => fixture.id)).size, 8);
  return fixtures.map((fixture: any) => {
    assert.equal(fixture.source_sha256, sha256(Buffer.from(fixture.source)));
    assert.equal(fixture.filename, "test.vue");
    assert.equal(fixture.entry, "template");
    for (const option of ["vue_version", "vapor", "locale", "help_level", "severity"])
      assert.equal(fixture[option], null, "original unspecified options must remain absent");
    for (const object of ["fix", "parent", "witness_revision", "witness_blob"])
      assert.match(fixture[object], /^[a-f0-9]{40}$/);
    assert(["a11y/no-autofocus", "a11y/no-access-key"].includes(fixture.rule));
    assert(["introduced-by-fix", "parent-control"].includes(fixture.witness_kind));
    assert([0, 1].includes(fixture.original_warning_count));
    // No expected output or inferred full diagnostic body accompanies this input.
    return { id: fixture.id, authored: fixture, input: Buffer.from(JSON.stringify(fixture)) };
  });
}

export function decodeFocusRefusal(bytes: Buffer, fixture: any) {
  const outcome = JSON.parse(utf8(bytes));
  assert.deepEqual(Object.keys(outcome).sort(), ["context", "refusal", "state"]);
  assert.equal(
    outcome.state,
    "refused",
    "unprovided actual concrete rules cannot claim native handling",
  );
  assert.equal(typeof outcome.context, "string");
  assert(outcome.context.startsWith("Case {\n") && outcome.context.endsWith("\n"));
  assert(outcome.context.includes("\nRuleIdentity {\n"));
  assert.deepEqual(outcome.refusal, {
    kind: "UnprovidedRule",
    detail: `UnprovidedRule { rule: "${fixture.authored.rule}" }`,
    unprovided_rule: fixture.authored.rule,
  });
  return outcome;
}
