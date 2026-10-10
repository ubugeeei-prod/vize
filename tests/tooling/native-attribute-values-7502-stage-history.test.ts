// Synthetic historical schema controls confer no source-built execution credit.
import assert from "node:assert/strict";
import { test } from "node:test";
import { baseline7502 } from "./support/native-attribute-values-7502-inputs.ts";
import { historyCommands7502 } from "./support/native-attribute-values-7502-history.ts";
import {
  stageTests7502,
  validateStageHistory7502,
} from "./support/native-attribute-values-7502-stage-history.ts";

test("all three original class stage targets remain mandatory exact-source Cargo commands", () => {
  const commands = historyCommands7502("/baseline", "/history");
  assert.equal(commands.length, 12);
  assert.deepEqual(commands.slice(9), [
    [
      "cargo",
      [
        "test",
        "--locked",
        "--profile",
        "ci",
        "-p",
        "vize_l2",
        "--test",
        "native_selected_owner",
        "--",
        "--nocapture",
      ],
    ],
    [
      "cargo",
      [
        "test",
        "--locked",
        "--profile",
        "ci",
        "-p",
        "vize_l4",
        "--test",
        "native_selected_template",
        "--",
        "--nocapture",
      ],
    ],
    [
      "cargo",
      [
        "test",
        "--locked",
        "--profile",
        "ci",
        "-p",
        "vize_atelier_sfc",
        "--lib",
        "native_selected_setup::tests::original_for::refusal::constant_full_capture_generic_and_zero_occurrence_literal_preserve_distinct_policies",
        "--",
        "--exact",
        "--nocapture",
      ],
    ],
  ]);
});
test("whole historical stage provenance rejects omitted, substituted and reordered sources", () => {
  const original = {
    sourceRevision: baseline7502.revision,
    sourceTree: baseline7502.tree,
    tests: stageTests7502.map((row) => ({
      step: row.step,
      testPath: row.path,
      testSourceSha256: row.sha256,
    })),
  };
  validateStageHistory7502(original);
  for (const mutate of [
    (receipt: any) => (receipt.sourceRevision = "main"),
    (receipt: any) => (receipt.sourceTree = "current"),
    (receipt: any) => receipt.tests.pop(),
    (receipt: any) => receipt.tests.reverse(),
    (receipt: any) => (receipt.tests[0].testSourceSha256 = "current-source"),
    (receipt: any) => (receipt.tests[1].testPath = "replacement.rs"),
    (receipt: any) => (receipt.tests[2].step = "skipped"),
  ]) {
    const changed = structuredClone(original);
    mutate(changed);
    assert.throws(() => validateStageHistory7502(changed));
  }
});
