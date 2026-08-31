import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

const injectNativeOptionalDeps =
  "rust-script tools/commands/release/npm/inject-native-optional-deps.rs";

function assertReleaseJobRunsRustScript(jobName: string, command: string): void {
  const job = workflowJobBody(readRepoFile(".github", "workflows", "release.yml"), jobName);
  const setupRust = job.indexOf("uses: dtolnay/rust-toolchain@");
  const setupRustScript = job.indexOf("uses: ./.github/actions/setup-rust-script");
  const commandIndex = job.indexOf(command);

  assert.ok(setupRust >= 0, `${jobName} must set up Rust before Rust Script`);
  assert.ok(setupRustScript > setupRust, `${jobName} must install rust-script after Rust`);
  assert.ok(
    commandIndex > setupRustScript,
    `${jobName} must run ${command} after rust-script setup`,
  );
}

test("release workflow injects native optional dependency pins with Rust Script", () => {
  assertReleaseJobRunsRustScript(
    "release-npm-oxlint-plugin",
    `${injectNativeOptionalDeps} npm/oxlint/package.json npm/native/package.json --print`,
  );
  assertReleaseJobRunsRustScript(
    "release-npm-cli",
    `${injectNativeOptionalDeps} npm/cli/package.json --print`,
  );

  const workflow = readRepoFile(".github", "workflows", "release.yml");
  assert.doesNotMatch(workflow, /tools\/moon\/cmd\/inject_native_optional_deps/);
});
