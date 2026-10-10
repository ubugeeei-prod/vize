// The three named coded historical class assumptions run on their own source.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { baseline7502, exactKeys7502, hash7502 } from "./native-attribute-values-7502-inputs.ts";

export const stageTests7502 = [
  {
    step: "selected-owner-build",
    path: "davinci/vize_l2/tests/native_selected_owner_cases/attribute.rs",
    sha256: "43bbe2f6b5b2e2c4ec7638547af99b21e4e186bcd497cdbbd43f7c2b95458ec1",
    argv: [
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
  },
  {
    step: "selected-template-build",
    path: "davinci/vize_l4/tests/native_selected_template_cases/refusal.rs",
    sha256: "1f6f981707350e8258447592559d23b63f874339e9798158f188be12e590b939",
    argv: [
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
  },
  {
    step: "selected-for-class-build",
    path: "crates/vize_atelier_sfc/src/native_selected_setup/tests/original_for/refusal.rs",
    sha256: "ef470424d35663b7222cdab1f521a25bbdbb9f880cf811be9fb5620108f966df",
    argv: [
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
  },
] as const;
interface StageSource7502 {
  step: string;
  testPath: string;
  testSourceSha256: string;
}
export interface StageHistory7502 {
  sourceRevision: string;
  sourceTree: string;
  tests: StageSource7502[];
}
export function stageHistory7502(worktree: string): StageHistory7502 {
  const tests = stageTests7502.map((test) => {
    const bytes = readFileSync(path.join(worktree, test.path));
    assert.equal(hash7502(bytes), test.sha256, "unchanged whole original historical test source");
    return { step: test.step, testPath: test.path, testSourceSha256: test.sha256 };
  });
  const receipt = { sourceRevision: baseline7502.revision, sourceTree: baseline7502.tree, tests };
  validateStageHistory7502(receipt);
  return receipt;
}
export function validateStageHistory7502(receipt: StageHistory7502) {
  exactKeys7502(receipt, ["sourceRevision", "sourceTree", "tests"]);
  assert.equal(receipt.sourceRevision, baseline7502.revision);
  assert.equal(receipt.sourceTree, baseline7502.tree);
  assert.equal(receipt.tests.length, 3);
  for (const [index, test] of receipt.tests.entries()) {
    exactKeys7502(test, ["step", "testPath", "testSourceSha256"]);
    const expected = stageTests7502[index];
    assert.deepEqual(
      test,
      { step: expected.step, testPath: expected.path, testSourceSha256: expected.sha256 },
      "all three entire named original tests stay authenticated",
    );
  }
}
