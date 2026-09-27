import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import {
  currentCommand,
  extendHashKeyWorkflow,
} from "../../tools/support/levels/extend-hash-key-workflow.mjs";

const originalCommand =
  "cargo test -p vize_davinci --test artifact_keys --test key_manifests -- --nocapture";
const source = `name: Current Incremental
on:
  push: { branches: [main], paths: ["current/**"] }
  workflow_dispatch:
concurrency: { group: current-source, cancel-in-progress: true }
jobs:
  key-stability:
    permissions: { contents: read }
    strategy:
      matrix:
        platform:
          - { runner: current-linux, target: linux-x64-gnu }
          - { runner: current-macos, target: darwin-arm64 }
    steps:
      - { uses: current-cache, with: { key: preserved-current-key } }
      - name: Print and compare the TS-43 golden keys
        run: ${originalCommand}
  unrelated:
    runs-on: current-unrelated
    steps: [{run: echo preserve-me}]
`;

test("hash replay preserves current events, runners, caches and unrelated jobs", () => {
  const expected = parse(source);
  expected.jobs["key-stability"].steps[1].run = currentCommand;
  const after = extendHashKeyWorkflow(source);
  assert.deepEqual(parse(after), expected);
  assert.equal(extendHashKeyWorkflow(after), after);
});

test("hash replay refuses a stale PR-trigger workflow", () => {
  assert.throws(
    () =>
      extendHashKeyWorkflow(
        source.replace("  workflow_dispatch:", "  pull_request:\n  workflow_dispatch:"),
      ),
    /AssertionError/,
  );
});

test("hash replay refuses an unknown command rather than replacing current source", () => {
  assert.throws(
    () => extendHashKeyWorkflow(source.replace(originalCommand, "cargo test current-custom")),
    /unrecognized current key-stability command/,
  );
});
