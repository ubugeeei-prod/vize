import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { repoRoot } from "../_helpers/tool-commands.ts";
import { createReleasePreflightVerifyOnlyFixture } from "./release-preflight-runner-fixture.ts";

/** Runs the actual Rust release preflight over the full 22-archive fixture. */
export function runVerifyOnlyPreflight(
  mutateFirstShard?: (entries: Record<string, string>) => void,
) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-gated-matrix-"));
  try {
    const fixture = createReleasePreflightVerifyOnlyFixture(directory, {
      mutateShardEntries(shard, entries) {
        if (shard === 0) mutateFirstShard?.(entries);
      },
    });
    const result = spawnSync(
      "rust-script",
      ["tools/commands/ci/github/release-preflight.rs", "--verify-only"],
      {
        cwd: repoRoot,
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${fixture.binDir}${path.delimiter}${process.env.PATH ?? ""}`,
          ...fixture.env,
        },
      },
    );
    assert.ifError(result.error);
    return result;
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}
