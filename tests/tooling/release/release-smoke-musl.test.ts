import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "../support/github-workflows.ts";

interface WorkflowStep {
  readonly env?: Record<string, string>;
  readonly run?: string;
  readonly uses?: string;
  readonly with?: Record<string, string>;
}

interface MuslJob {
  readonly "runs-on"?: string;
  readonly strategy?: {
    readonly matrix?: {
      readonly include?: ReadonlyArray<Record<string, string>>;
    };
  };
  readonly steps: WorkflowStep[];
}

function muslJob(): MuslJob {
  const workflow = parse(readRepoFile(".github", "workflows", "native-smoke.yml")) as {
    jobs: Record<string, MuslJob>;
  };
  return workflow.jobs["fresh-install-musl-smoke"];
}

test("the musl fresh install smoke cross-builds x64 and arm64 in Alpine", () => {
  const job = muslJob();
  const steps = job.steps;
  assert.equal(job["runs-on"], "${{ matrix.runner }}");
  assert.deepEqual(job.strategy?.matrix?.include, [
    {
      runner: "blacksmith-32vcpu-ubuntu-2404",
      target: "x86_64-unknown-linux-musl",
      platform: "linux-x64-musl",
    },
    {
      runner: "ubuntu-22.04-arm",
      target: "aarch64-unknown-linux-musl",
      platform: "linux-arm64-musl",
    },
  ]);
  const build = steps.find((step) => step.uses === "./.github/actions/build-release-native");
  assert.deepEqual(build?.with, {
    target: "${{ matrix.target }}",
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
    step.run?.includes("node tools/support/github/stage-built-native-packages.ts npm/native"),
  );
  assert.ok(prepare, "musl binding must be staged in its packed platform package");
  assert.ok(
    prepare.run?.includes(
      'test -f "npm/native/npm/$NATIVE_PLATFORM/vize-vitrine.$NATIVE_PLATFORM.node"',
    ),
    "the staged musl package must hold the cross-built binding",
  );

  const alpineStep = steps.find((step) => step.run?.includes("node:24-alpine"));
  const alpine = alpineStep?.run ?? "";
  assert.match(alpine, /docker run --rm/u);
  assert.match(alpine, /node:24-alpine@sha256:[0-9a-f]{64}/u);
  assert.match(
    alpine,
    /node tools\/commands\/release\/npm\/smoke-release-install\.ts[\s\\]+--prepare-manifests --runtime-checks/u,
  );
  for (const packageDir of [
    "npm/native",
    '"npm/native/npm/$NATIVE_PLATFORM"',
    "npm/cli",
    "npm/builder/vite",
  ])
    assert.ok(alpine.includes(packageDir), `musl smoke does not pack ${packageDir}`);
  assert.ok(
    alpine.includes('"schema":"vize.loadedNativeBinding"'),
    "musl smoke must prove which binding the fresh project loaded",
  );
  assert.equal(alpineStep?.env?.NATIVE_PLATFORM, "${{ matrix.platform }}");
  assert.equal(
    alpineStep?.env?.NATIVE_PACKAGE,
    "${{ format('@vizejs/native-{0}', matrix.platform) }}",
  );
  assert.match(alpine, /VIZE_SMOKE_EXPECTED_NATIVE_PACKAGE=\$NATIVE_PACKAGE/u);
  assert.match(alpine, /grep -F .*package/u, "receipt package evidence stays independent");
});
