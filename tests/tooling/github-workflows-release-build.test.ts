import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import {
  cliReleasePlatforms,
  nativeReleasePlatforms,
} from "../../tools/support/github/release-platforms.ts";
import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  uses?: string;
  run?: string;
  if?: string;
  with?: Record<string, unknown>;
  shell?: string;
};
const release = parse(readRepoFile(".github", "workflows", "release.yml"));
const seed = parse(readRepoFile(".github", "workflows", "release-build-cache.yml"));
const recipe = (name: string): { runs: { steps: Step[] } } =>
  parse(readRepoFile(".github", "actions", name, "action.yml"));

test("release and cache warming share every exact-checkout binary recipe", () => {
  for (const [releaseJob, seedJob, action] of [
    ["build-cli", "cli", "build-release-cli"],
    ["build-native-all", "native", "build-release-native"],
  ]) {
    const released = release.jobs[releaseJob];
    assert.equal(released.needs, "plan-release-platforms");
    const releaseStep = released.steps.find(
      (step: Step) => step.uses === `./.github/actions/${action}`,
    );
    const seedStep = seed.jobs[seedJob].steps.find((step: Step) => step.uses === releaseStep.uses);
    assert.equal(releaseStep.with["authorized-release"], "true");
    assert.equal(seedStep.with["authorized-release"], undefined);
    assert.equal(seedStep.with["upload-artifacts"], "false");
    assert.equal(releaseStep.with.target, seedStep.with.target);
    for (const step of recipe(action).runs.steps) if (step.run) assert.ok(step.shell);
  }
  assert.deepEqual(seed.permissions, { contents: "read" });
  assert.equal(seed.concurrency["cancel-in-progress"], true);
  assert.deepEqual(seed.on.push.branches, ["main"]);
  assert.doesNotMatch(
    readRepoFile(".github", "workflows", "release-build-cache.yml"),
    /id-token:|environment:|secrets\./,
  );
});

test("all shipping platforms retain target and ABI guards on accelerated runners", () => {
  const expected = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-pc-windows-msvc",
    "x86_64-pc-windows-msvc",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-musl",
    "x86_64-unknown-linux-musl",
  ].sort();
  assert.deepEqual(cliReleasePlatforms.map(({ target }) => target).sort(), expected);
  assert.deepEqual(nativeReleasePlatforms.map(({ target }) => target).sort(), expected);
  for (const platform of nativeReleasePlatforms.filter(({ target }) => target.endsWith("-gnu"))) {
    assert.match(platform.host, /^blacksmith-32vcpu-ubuntu-2204(?:-arm)?$/);
  }
  for (const name of ["build-release-cli", "build-release-native"]) {
    const steps = recipe(name).runs.steps;
    const toolchain = steps.find(({ uses }) => uses?.startsWith("dtolnay/rust-toolchain@"));
    assert.equal(toolchain?.with?.targets, "${{ inputs.target }}");
    assert.equal(toolchain?.with?.toolchain, "${{ steps.rust-toolchain.outputs.channel }}");
    const sdk = steps.find(({ uses }) => uses?.startsWith("ilammy/msvc-dev-cmd@"));
    assert.match(String(sdk?.with?.arch), /amd64_arm64/);
    assert.match(
      steps.find(({ name }) => name === "Tune Windows release profile")?.run ?? "",
      /CARGO_PROFILE_RELEASE_LTO=thin/,
    );
  }
  const native = readRepoFile(".github", "actions", "build-release-native", "action.yml");
  assert.equal((native.match(/verify-glibc-symbols\.rs --max 2\.36/g) ?? []).length, 2);
  const cli = readRepoFile(".github", "actions", "build-release-cli", "action.yml");
  assert.match(cli, /verify-musl-cli-binary\.rs/);
  assert.match(cli, /configure-zig-musl-linkers\.rs/);
});

test("every produced native artifact is guarded against missing output", () => {
  for (const name of ["build-release-cli", "build-release-native"]) {
    const uploads = recipe(name).runs.steps.filter(({ uses }) =>
      uses?.startsWith("actions/upload-artifact@"),
    );
    assert.equal(uploads.length, name === "build-release-cli" ? 1 : 2);
    for (const step of uploads) {
      assert.equal(step.if, "inputs.upload-artifacts == 'true'");
      assert.equal(step.with?.["if-no-files-found"], "error");
      assert.equal(step.with?.["compression-level"], 0);
    }
  }
});

test("publication keeps installed-package smoke and exact promotion barriers", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const smoke = workflowJobBody(workflow, "smoke-release-packages");
  assert.match(smoke, /smoke-release-install\.rs --prepare-manifests --runtime-checks/);
  assert.deepEqual(release.jobs["smoke-release-packages"].needs, [
    "plan-release-platforms",
    "build-release-packages",
    "build-native-all",
  ]);
  for (const job of Object.values(release.jobs) as Array<{
    name?: string;
    needs?: string | string[];
    steps?: Step[];
  }>) {
    if (
      !job.steps?.some(
        ({ run, uses }) =>
          run?.includes("publish_npm_package/main.ts") ||
          uses === "./.github/actions/publish-release-package",
      )
    )
      continue;
    assert.ok(job.needs?.includes("release-preflight"), job.name);
  }
  for (const jobName of ["release-npm-native", "release-npm-fresco-native", "release-npm-wasm"]) {
    const steps = release.jobs[jobName].steps as Step[];
    const smokeIndex = steps.findIndex(({ run }) => run?.includes("smoke-release-install.rs"));
    const publishIndex = steps.findIndex(({ run }) => run?.includes("publish_npm_package"));
    assert.ok(smokeIndex >= 0 && publishIndex > smokeIndex, jobName);
  }
  const fresco = workflowJobBody(workflow, "release-npm-fresco-native");
  assert.match(fresco, /clean-node-binaries\.ts npm\/fresco-native/);
  assert.match(fresco, /collect-native-artifacts\.ts/);
  assert.doesNotMatch(fresco, /napi create-npm-dirs/);
});
