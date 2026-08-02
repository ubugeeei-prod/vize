import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

type WorkflowStep = {
  env?: Record<string, string>;
  id?: string;
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
};

type CheckWorkflow = {
  jobs?: Record<
    string,
    {
      permissions?: Record<string, string>;
      steps?: WorkflowStep[];
    }
  >;
};

test("the SemVer job resolves its release type before running cargo-semver-checks", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "check.yml")) as CheckWorkflow;
  const job = workflow.jobs?.["semver-checks"];
  assert.ok(job);
  assert.deepEqual(job.permissions, { contents: "read", "pull-requests": "read" });

  const steps = job.steps ?? [];
  assert.deepEqual(
    steps.find((step) => step.id === "semver-release-type"),
    {
      env: { GITHUB_TOKEN: "${{ github.token }}" },
      id: "semver-release-type",
      name: "Resolve SemVer release type",
      run: "node tools/github/semver-change-marker.mjs",
    },
  );

  assert.deepEqual(
    steps.find((step) => step.name === "Check public API SemVer compatibility"),
    {
      env: {
        BASELINE_REV:
          "${{ github.event_name == 'pull_request' && github.event.pull_request.base.sha || (github.event_name == 'push' && github.event.before || '') }}",
        SEMVER_RELEASE_TYPE: "${{ steps.semver-release-type.outputs.release-type }}",
      },
      name: "Check public API SemVer compatibility",
      run: [
        'case "$BASELINE_REV" in 0000000000000000000000000000000000000000) BASELINE_REV="";; esac',
        "SEMVER_ARGS=()",
        'case "$SEMVER_RELEASE_TYPE" in major) SEMVER_ARGS+=(--release-type major);; esac',
        'if [ -n "$BASELINE_REV" ]; then',
        '  cargo semver-checks check-release --package ${{ matrix.crate }} --baseline-rev "$BASELINE_REV" "${SEMVER_ARGS[@]}"',
        "else",
        '  cargo semver-checks check-release --package ${{ matrix.crate }} "${SEMVER_ARGS[@]}"',
        "fi",
        "",
      ].join("\n"),
    },
  );

  assert.deepEqual(
    steps.map((step) => step.name ?? step.uses?.split("@")[0]),
    [
      "actions/checkout",
      "Resolve SemVer release type",
      "dtolnay/rust-toolchain",
      "wild-linker/action",
      "./.github/actions/setup-rust-sticky-cache",
      "Install cargo-semver-checks",
      "Check public API SemVer compatibility",
    ],
  );
});
