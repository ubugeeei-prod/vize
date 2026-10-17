import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

type ReleaseJob = {
  steps?: Array<{
    run?: string;
    uses?: string;
    with?: Record<string, unknown>;
  }>;
};

test("smoke job downloads every packed artifact directory it smoke-installs", () => {
  const source = readRepoFile(".github", "workflows", "release.yml");
  const workflow = parse(source) as { jobs?: Record<string, ReleaseJob> };
  const jobs = workflow.jobs ?? {};

  // Directory -> artifact name, derived from every release-package-* upload.
  // Packages assembled in-job from bindings artifacts (native, fresco-native)
  // never appear here, so they are exempt by construction rather than by list.
  const uploadedPathToName = new Map<string, string>();
  const packageBuilder = jobs["build-release-packages"];
  assert.ok(
    packageBuilder.steps?.some((step) => step.uses === "./.github/actions/release-build-packages"),
  );
  const composite = parse(
    readRepoFile(".github", "actions", "release-build-packages", "action.yml"),
  ) as { runs: ReleaseJob };
  for (const job of [...Object.values(jobs), composite.runs]) {
    for (const step of job.steps ?? []) {
      if (!step.uses?.includes("actions/upload-artifact@")) continue;
      const name = step.with?.name;
      const path = step.with?.path;
      if (typeof name !== "string" || !name.startsWith("release-package-")) continue;
      assert.ok(typeof path === "string", `artifact ${name} must upload a single directory path`);
      uploadedPathToName.set(path, name);
    }
  }
  assert.ok(uploadedPathToName.size > 0, "expected release-package-* uploads in release.yml");

  const smokeJob = jobs["smoke-release-packages"];
  assert.ok(smokeJob, "missing smoke-release-packages job");
  const smokeRun = (smokeJob.steps ?? []).find((step) =>
    step.run?.includes("smoke-release-install.ts"),
  )?.run;
  assert.ok(smokeRun, "smoke-release-packages must run smoke-release-install.ts");
  const smokedDirectories = smokeRun
    .replace(/\\\n/g, " ")
    .split(/\s+/)
    .filter((token) => token.startsWith("npm/"));
  assert.ok(smokedDirectories.length > 0, "smoke install list must not be empty");

  const smokeDownloads = new Map<string, string>();
  for (const step of smokeJob.steps ?? []) {
    if (!step.uses?.includes("actions/download-artifact@")) continue;
    const name = step.with?.name;
    const path = step.with?.path;
    if (typeof name !== "string" || !name.startsWith("release-package-")) continue;
    assert.ok(typeof path === "string", `smoke download ${name} must target a directory path`);
    smokeDownloads.set(path, name);
    assert.equal(
      uploadedPathToName.get(path),
      name,
      `smoke download ${name} -> ${path} must mirror the build upload wiring`,
    );
  }

  for (const directory of smokedDirectories) {
    const artifactName = uploadedPathToName.get(directory);
    if (artifactName == null) continue;
    assert.equal(
      smokeDownloads.get(directory),
      artifactName,
      `smoke-release-packages must download ${artifactName} into ${directory} before installing it`,
    );
  }
});

test("wasm release job installs workspace dependencies before root smoke scripts", () => {
  const source = readRepoFile(".github", "workflows", "release.yml");
  const workflow = parse(source) as { jobs?: Record<string, ReleaseJob> };
  const wasmJob = workflow.jobs?.["release-npm-wasm"];
  assert.ok(wasmJob, "missing release-npm-wasm job");

  const steps = wasmJob.steps ?? [];
  const setupIndex = steps.findIndex(
    (step) => step.uses?.includes("voidzero-dev/setup-vp@") && step.with?.["run-install"] === true,
  );
  const smokeIndex = steps.findIndex((step) =>
    step.run?.includes("node tools/commands/release/npm/smoke-release-install.ts npm/wasm"),
  );

  assert.notEqual(setupIndex, -1, "release-npm-wasm setup-vp must install root dependencies");
  assert.notEqual(smokeIndex, -1, "release-npm-wasm must smoke install the WASM tarball");
  assert.ok(
    setupIndex < smokeIndex,
    "release-npm-wasm must install root dependencies before smoke-release-install.ts imports semver",
  );
});

test("all five release installers use TypeScript and retain unrelated Rust WASM checks", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "release.yml")) as {
    jobs: Record<string, ReleaseJob>;
  };
  const expected = [
    "build-wasm-package",
    "smoke-release-packages",
    "release-npm-native",
    "release-npm-fresco-native",
    "release-npm-wasm",
  ];
  const invocations = Object.entries(workflow.jobs).flatMap(([job, body]) =>
    (body.steps ?? [])
      .filter((step) => step.run?.includes("smoke-release-install."))
      .map((step) => ({ job, args: step.run!.replace(/\\\n/gu, " ").trim().split(/\s+/u) })),
  );
  assert.deepEqual(
    invocations.map(({ job }) => job),
    expected,
  );
  for (const { args } of invocations)
    assert.deepEqual(args.slice(0, 2), [
      "node",
      "tools/commands/release/npm/smoke-release-install.ts",
    ]);
  for (const job of ["smoke-release-packages", "release-npm-native", "release-npm-fresco-native"])
    for (const step of workflow.jobs[job].steps ?? []) {
      assert.ok(!step.uses?.includes("rust-toolchain"), `${job} needs no Rust toolchain`);
      assert.notEqual(step.uses, "./.github/actions/setup-rust-script");
      assert.ok(!step.run?.includes("rust-script"), `${job} must not retain a Rust script caller`);
    }
  for (const job of ["build-wasm-package", "release-npm-wasm"]) {
    const steps = workflow.jobs[job].steps ?? [];
    assert.ok(steps.some((step) => step.uses?.includes("rust-toolchain")));
    assert.ok(steps.some((step) => step.uses === "./.github/actions/setup-rust-script"));
    assert.ok(
      steps.some(
        (step) =>
          step.run === "rust-script tools/commands/release/npm/smoke-wasm-package.rs npm/wasm",
      ),
    );
  }
});
