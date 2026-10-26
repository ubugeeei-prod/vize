import assert from "node:assert/strict";
import { test } from "node:test";

import {
  backfillTag,
  compareStableVersions,
  higherStableVersions,
  releaseRunSource,
  validateResumePolicy,
  type ResumePolicyInput,
} from "../../../tools/support/release/npm_resume/policy.ts";

const valid = (): ResumePolicyInput => ({
  repository: "ubugeeei-prod/vize",
  run: {
    repository: "ubugeeei-prod/vize",
    workflowPath: ".github/workflows/release.yml",
    event: "workflow_dispatch",
    branch: "main",
    status: "completed",
    conclusion: "failure",
    headSha: "a".repeat(40),
    sourceSha: null,
  },
  tagParent: "a".repeat(40),
  artifactNames: ["npm-packages", "npm-package-wasm", "bindings-test"],
  requiredArtifacts: ["npm-packages", "npm-package-wasm", "bindings-test"],
  targetVersion: "0.506.0",
  higherVersions: [],
  incompleteHigherPackages: [],
});

test("npm resume compares only stable releases and derives a non-latest backfill tag", () => {
  assert.equal(compareStableVersions("0.506.0", "0.505.0"), 1);
  assert.equal(compareStableVersions("1.0.0", "0.999.0"), 1);
  assert.equal(compareStableVersions("0.505.0", "0.505.0"), 0);
  assert.equal(compareStableVersions("0.506.0-rc.1", "0.505.0"), null);
  assert.deepEqual(
    higherStableVersions("0.505.0", ["0.504.0", "0.507.0", "0.506.0", "0.506.0-rc.1"]),
    ["0.506.0", "0.507.0"],
  );
  assert.equal(backfillTag("0.505.0"), "backfill-0-505-0");
});

test("npm resume accepts only a finished release run with its complete immutable artifact set", () => {
  assert.deepEqual(validateResumePolicy(valid()), []);
  // Verification runs after the Release run, so a successful run may still lack a package.
  for (const conclusion of ["success", "cancelled", "timed_out"]) {
    const finished = valid();
    finished.run.conclusion = conclusion;
    assert.deepEqual(validateResumePolicy(finished), [], conclusion);
  }
  for (const conclusion of [null, "skipped", "startup_failure"]) {
    const unfinished = valid();
    unfinished.run.status = conclusion === null ? "in_progress" : "completed";
    unfinished.run.conclusion = conclusion;
    assert.deepEqual(
      validateResumePolicy(unfinished),
      conclusion === null
        ? ["run is not complete", "only finished release runs can be resumed"]
        : ["only finished release runs can be resumed"],
      String(conclusion),
    );
  }

  const wrong = valid();
  wrong.run.workflowPath = ".github/workflows/check.yml";
  wrong.run.headSha = "b".repeat(40);
  wrong.artifactNames = ["npm-packages"];
  wrong.incompleteHigherPackages = ["@vizejs/native-win32-arm64-msvc@0.506.0"];
  assert.deepEqual(validateResumePolicy(wrong), [
    "run did not use release.yml",
    "tag does not belong to the selected run",
    "missing release artifact: npm-package-wasm",
    "missing release artifact: bindings-test",
    "newer releases are incomplete: @vizejs/native-win32-arm64-msvc@0.506.0",
  ]);
});

test("npm resume matches the tag against the dispatched source, not the moved main head", () => {
  const source = "c".repeat(40);
  assert.equal(releaseRunSource(`Release next minor @ ${source}`), source);
  assert.equal(releaseRunSource("Release 0.578.0"), null);
  assert.equal(releaseRunSource(`Release npm resume 0.578.0 from run 1 @ ${source}x`), null);

  const cadence = valid();
  cadence.run.headSha = "d".repeat(40);
  cadence.run.sourceSha = source;
  cadence.tagParent = source;
  assert.deepEqual(validateResumePolicy(cadence), []);

  cadence.tagParent = cadence.run.headSha;
  assert.deepEqual(validateResumePolicy(cadence), ["tag does not belong to the selected run"]);
});
