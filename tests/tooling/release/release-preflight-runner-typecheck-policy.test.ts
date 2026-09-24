import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { repoRoot } from "../_helpers/moonbit.ts";
import { createReleasePreflightVerifyOnlyFixture } from "../support/release-preflight-runner-fixture.ts";

test("verify-only mode accepts shards without optional typecheck divergence artifacts", () => {
  const tempDir = fs.mkdtempSync(path.join(tmpdir(), "vize-release-optional-typecheck-"));
  try {
    const fixture = createReleasePreflightVerifyOnlyFixture(tempDir, {
      mutateShardEntries(_shard, entries) {
        for (const entryName of Object.keys(entries)) {
          if (entryName.endsWith("-typecheck-divergence.json")) delete entries[entryName];
        }
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
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, new RegExp(`Release preflight passed for ${fixture.tag}`));
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});
