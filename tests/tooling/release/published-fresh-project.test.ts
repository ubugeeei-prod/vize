import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { expectedNativePackage } from "../../../tools/support/release/verify_published/fresh_project.ts";
import { readRepoFile } from "../support/github-workflows.ts";

type Step = {
  env?: Record<string, string>;
  if?: string;
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
};
type Job = {
  needs?: string | string[];
  "runs-on"?: string;
  steps?: Step[];
  strategy?: { matrix?: { include?: Array<Record<string, string>> } };
  uses?: string;
  with?: Record<string, string>;
};

const published = parse(readRepoFile(".github", "workflows", "published-fresh-project.yml")) as {
  on: {
    pull_request: { paths: string[] };
    workflow_call: { inputs: Record<string, { required?: boolean }> };
    workflow_dispatch: { inputs: Record<string, unknown> };
  };
  jobs: Record<string, Job>;
};
const releaseVerify = parse(readRepoFile(".github", "workflows", "release-verify.yml")) as {
  jobs: Record<string, Job>;
};

test("public native selection names every supported package family", () => {
  assert.equal(expectedNativePackage("darwin", "x64"), "@vizejs/native-darwin-x64");
  assert.equal(expectedNativePackage("darwin", "arm64"), "@vizejs/native-darwin-arm64");
  assert.equal(expectedNativePackage("win32", "x64"), "@vizejs/native-win32-x64-msvc");
  assert.equal(expectedNativePackage("win32", "arm64"), "@vizejs/native-win32-arm64-msvc");
  assert.equal(expectedNativePackage("linux", "x64", "gnu"), "@vizejs/native-linux-x64-gnu");
  assert.equal(expectedNativePackage("linux", "arm64", "musl"), "@vizejs/native-linux-arm64-musl");
});

test("published fresh projects run from exact public versions on every release host", () => {
  assert.ok(published.on.pull_request.paths.includes(".github/workflows/release-verify.yml"));
  assert.ok(
    published.on.pull_request.paths.includes(
      "tools/support/release/verify_published/fresh_project_fixture.ts",
    ),
  );
  assert.equal(published.on.workflow_call.inputs.version?.required, true);
  assert.equal(published.on.workflow_call.inputs.source_ref?.required, true);
  assert.ok("version" in published.on.workflow_dispatch.inputs);

  const standard = published.jobs["public-hosts"]!;
  assert.equal(standard.needs, "resolve");
  assert.deepEqual(
    standard.strategy?.matrix?.include?.map(({ platform }) => platform),
    [
      "linux-x64-gnu",
      "linux-arm64-gnu",
      "darwin-x64",
      "darwin-arm64",
      "win32-x64-msvc",
      "win32-arm64-msvc",
    ],
  );
  const serialized = JSON.stringify(standard.steps);
  assert.match(serialized, /verify_published\/fresh_project\.ts/);
  assert.match(serialized, /needs\.resolve\.outputs\.version/);
  assert.match(serialized, /published-fresh-project-\$\{\{ matrix\.platform \}\}/);

  const musl = published.jobs["public-musl"]!;
  assert.deepEqual(
    musl.strategy?.matrix?.include?.map(({ platform }) => platform),
    ["linux-x64-musl", "linux-arm64-musl"],
  );
  const alpine = musl.steps?.find((step) => step.name?.includes("Alpine"))?.run ?? "";
  assert.match(alpine, /node:24-alpine@sha256:[0-9a-f]{64}/u);
  assert.match(alpine, /\$GITHUB_WORKSPACE:\/work:ro/u);
  assert.match(alpine, /VIZE_PUBLISHED_FRESH_EVIDENCE=\/evidence\/receipt\.json/u);
});

test("release verification calls the public host matrix after npm propagation", () => {
  const job = releaseVerify.jobs["verify-fresh-projects"]!;
  assert.equal(job.needs, "verify-npm");
  assert.equal(job.uses, "./.github/workflows/published-fresh-project.yml");
  assert.deepEqual(job.with, {
    version: "${{ inputs.version }}",
    source_ref: "${{ inputs.tag }}",
  });
});
