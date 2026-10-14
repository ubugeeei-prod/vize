import assert from "node:assert/strict";
import { test } from "node:test";

import { isStaleLocalBuild, resolveVizeLaunchCommand } from "./support/lsp/launch.ts";

test("LSP tests ignore a globally installed vize binary", () => {
  const probed: string[] = [];
  const command = resolveVizeLaunchCommand(
    (candidate) => {
      probed.push(candidate);
      return candidate === "vize";
    },
    "",
    { required: false, isStale: () => false },
  );

  assert.ok(!probed.includes("vize"));
  assert.deepEqual(command, ["cargo", "run", "-q", "-p", "vize", "--", "lsp"]);
});

test("a checkout binary older than the Rust sources is never launched", () => {
  const probed: string[] = [];
  const command = resolveVizeLaunchCommand(
    (candidate) => {
      probed.push(candidate);
      return true;
    },
    "",
    { required: false, isStale: (binary) => binary.endsWith("target/debug/vize") },
  );
  assert.ok(!probed.some((candidate) => candidate.endsWith("target/debug/vize")));
  assert.ok(command[0].endsWith("target/ci/vize"), command.join(" "));
});

test("an explicit VIZE_LSP_BIN is used even when local builds are stale", () => {
  const command = resolveVizeLaunchCommand(() => true, "/custom/vize", {
    required: false,
    isStale: () => true,
  });
  assert.deepEqual(command, ["/custom/vize", "lsp"]);
});

test("every stale checkout build falls back to building the current source", () => {
  const command = resolveVizeLaunchCommand(() => true, "", {
    required: false,
    isStale: () => true,
  });
  assert.deepEqual(command, ["cargo", "run", "-q", "-p", "vize", "--", "lsp"]);
});

test("staleness compares the binary with the newest source", () => {
  assert.equal(isStaleLocalBuild(100, 200), true);
  assert.equal(isStaleLocalBuild(300, 200), false);
  assert.equal(isStaleLocalBuild(undefined, 200), false);
});
