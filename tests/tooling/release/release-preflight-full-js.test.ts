import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { repoRoot } from "../_helpers/moonbit.ts";
import { createReleasePreflightVerifyOnlyFixture } from "../support/release-preflight-runner-fixture.ts";

test("actual Rust verify-only refuses incomplete full JS qualification", () => {
  for (const jobName of ["test-js-packages", "build-js-packages", "full-js-report"]) {
    for (const variant of [
      "missing",
      "duplicate",
      "skipped",
      "failure",
      "cancelled",
      "unfinished",
    ]) {
      const tempDir = fs.mkdtempSync(path.join(tmpdir(), "vize-release-full-js-"));
      try {
        const fixture = createReleasePreflightVerifyOnlyFixture(tempDir, {
          mutateJobs(jobs) {
            const checkJobs = jobs[101];
            assert.ok(checkJobs);
            const index = checkJobs.findIndex((job) => job.name === jobName);
            assert.notEqual(index, -1);
            const original = checkJobs[index];
            if (variant === "missing") checkJobs.splice(index, 1);
            else if (variant === "duplicate") checkJobs.push({ ...original });
            else if (variant === "unfinished")
              checkJobs[index] = { ...original, status: "in_progress" };
            else checkJobs[index] = { ...original, conclusion: variant };
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
        assert.equal(result.status, 1, `${result.stderr}\n${result.stdout}`.trim());
        assert.ok(result.stderr.includes(jobName), result.stderr);
        if (variant === "missing") assert.match(result.stderr, /found 0/);
        else if (variant === "duplicate") assert.match(result.stderr, /found 2/);
        else if (variant === "unfinished") assert.match(result.stderr, /in_progress/);
        else assert.ok(result.stderr.includes(`completed/${variant}`), result.stderr);
      } finally {
        fs.rmSync(tempDir, { recursive: true, force: true });
      }
    }
  }
});
