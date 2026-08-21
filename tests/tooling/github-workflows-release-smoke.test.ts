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
  for (const job of Object.values(jobs)) {
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
    step.run?.includes("smoke-release-install.mjs"),
  )?.run;
  assert.ok(smokeRun, "smoke-release-packages must run smoke-release-install.mjs");
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
    step.run?.includes("node tools/npm/smoke-release-install.mjs npm/wasm"),
  );

  assert.notEqual(setupIndex, -1, "release-npm-wasm setup-vp must install root dependencies");
  assert.notEqual(smokeIndex, -1, "release-npm-wasm must smoke install the WASM tarball");
  assert.ok(
    setupIndex < smokeIndex,
    "release-npm-wasm must install root dependencies before smoke-release-install.mjs imports semver",
  );
});
