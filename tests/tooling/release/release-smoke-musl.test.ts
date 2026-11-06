import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "../support/github-workflows.ts";

interface WorkflowStep {
  readonly run?: string;
  readonly uses?: string;
  readonly with?: Record<string, string>;
}

function muslSteps(): readonly WorkflowStep[] {
  const workflow = parse(readRepoFile(".github", "workflows", "native-smoke.yml")) as {
    jobs: Record<string, { steps: WorkflowStep[] }>;
  };
  return workflow.jobs["fresh-install-musl-smoke"]?.steps ?? [];
}

test("the musl fresh install smoke cross-builds and runs the packed matrix in Alpine", () => {
  const steps = muslSteps();
  const build = steps.find((step) => step.uses === "./.github/actions/build-release-native");
  assert.deepEqual(build?.with, {
    target: "x86_64-unknown-linux-musl",
    "release-version": "${{ steps.version.outputs.version }}",
    "cross-compile": "true",
    "upload-artifacts": "false",
  });
  const buildIndex = steps.indexOf(build ?? {});
  const rustScriptIndex = steps.findIndex(
    (step) => step.uses === "./.github/actions/setup-rust-script",
  );
  assert.ok(
    rustScriptIndex > buildIndex,
    "the musl lane must provision rust-script before building packed CLI dependencies",
  );

  const prepare = steps.find((step) =>
    step.run?.includes("cp vize-vitrine.linux-x64-musl.node npm/linux-x64-musl/"),
  );
  assert.ok(prepare, "musl binding must be staged in its packed platform package");

  const alpine = steps.find((step) => step.run?.includes("node:24-alpine"))?.run ?? "";
  assert.match(alpine, /docker run --rm/u);
  assert.match(
    alpine,
    /node tools\/commands\/release\/npm\/smoke-release-install\.ts[\s\\]+--prepare-manifests --runtime-checks/u,
  );
  for (const packageDir of [
    "npm/native",
    "npm/native/npm/linux-x64-musl",
    "npm/cli",
    "npm/builder/vite",
  ])
    assert.ok(alpine.includes(packageDir), `musl smoke does not pack ${packageDir}`);
  assert.ok(
    alpine.includes('"package":"@vizejs/native-linux-x64-musl"'),
    "musl smoke must prove which binding the fresh project selected",
  );
});
