import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import {
  cliReleasePlatforms,
  nativeReleasePlatforms,
} from "../../tools/support/github/release-platforms.ts";
import { readRepoFile } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  uses?: string;
  run?: string;
  if?: string;
  with?: Record<string, unknown>;
  shell?: string;
  env?: Record<string, string>;
};
const release = parse(readRepoFile(".github", "workflows", "release.yml"));
const seed = parse(readRepoFile(".github", "workflows", "release-build-cache.yml"));
const recipe = (name: string): { runs: { steps: Step[] } } =>
  parse(readRepoFile(".github", "actions", name, "action.yml"));

test("cache planning preserves a failed version producer before writing any job output", () => {
  const step = seed.jobs.plan.steps.find((item: { id?: string }) => item.id === "next");
  assert.equal(step.shell, "bash");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-cache-version-producer-"));
  try {
    const bin = path.join(directory, "bin"),
      output = path.join(directory, "output");
    fs.mkdirSync(bin);
    const node = path.join(bin, "node");
    const original = "retained=original\n";
    for (const [status, version] of [
      [37, "0.614.0"],
      [0, "0.614.0-rc.1"],
    ] as const) {
      fs.writeFileSync(node, `#!/bin/sh\nprintf '%s\\n' '${version}'\nexit ${status}\n`, {
        mode: 0o755,
      });
      fs.writeFileSync(output, original);
      const result = spawnSync(
        "bash",
        ["--noprofile", "--norc", "-e", "-o", "pipefail", "-c", step.run],
        {
          cwd: directory,
          encoding: "utf8",
          env: {
            ...process.env,
            PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
            GITHUB_OUTPUT: output,
          },
        },
      );
      assert.equal(result.status, status, result.stderr);
      assert.equal(
        fs.readFileSync(output, "utf8"),
        status === 0 ? `${original}version=${version}\n` : original,
      );
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("release and cache warming share every exact-checkout binary recipe", () => {
  for (const [releaseJob, seedJob, action] of [
    ["build-cli", "cli", "build-release-cli"],
    ["build-native", "native", "build-release-native"],
  ]) {
    const released = release.jobs[releaseJob];
    assert.equal(released.needs, "version");
    const releaseStep = released.steps.find(
      (step: Step) => step.uses === `./.github/actions/${action}`,
    );
    const seedStep = seed.jobs[seedJob].steps.find((step: Step) => step.uses === releaseStep.uses);
    assert.equal(releaseStep.with["authorized-release"], "true");
    assert.equal(seedStep.with["authorized-release"], undefined);
    assert.equal(seedStep.with["upload-artifacts"], "false");
    assert.equal(releaseStep.with.target, seedStep.with.target);
    assert.equal(releaseStep.with["release-version"], "${{ needs.version.outputs.version }}");
    assert.equal(seedStep.with["release-version"], "${{ needs.plan.outputs.version }}");
    const builds = recipe(action).runs.steps.filter((step) => step.name?.startsWith("Build "));
    for (const step of builds) {
      assert.equal(step.env?.VIZE_RELEASE_VERSION, "${{ inputs.release-version }}");
    }
    for (const step of recipe(action).runs.steps) if (step.run) assert.ok(step.shell);
  }
  assert.deepEqual(seed.permissions, { contents: "read" });
  assert.equal(seed.concurrency.group, "release-build-cache-${{ github.ref }}");
  assert.equal(seed.concurrency["cancel-in-progress"], false);
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
