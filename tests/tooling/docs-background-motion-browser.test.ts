import assert from "node:assert/strict";
import { mkdir, mkdtemp, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { qualifyBackgroundMotion } from "../../docs/scripts/background-motion-qualifier.ts";

test(
  "actual authored WebGL respects reduced motion and native runtime updates",
  { timeout: 120_000 },
  async () => {
    const hosted = process.env.GITHUB_ACTIONS === "true";
    const directory = hosted
      ? path.resolve("target/differential/docs-background-motion")
      : os.tmpdir();
    if (hosted) await mkdir(directory, { recursive: true });
    const output =
      process.env.VIZE_BACKGROUND_MOTION_EVIDENCE ??
      (await mkdtemp(path.join(directory, "vize-background-motion-")));
    try {
      const receipt = await qualifyBackgroundMotion(output);
      assert.equal(receipt.completed, true);
      assert.equal(receipt.failure, null);
      assert.equal(receipt.cases.length, 2);
      console.log(
        `Background motion qualified: ${JSON.stringify({
          sourceHead: receipt.sourceHead,
          sourceChanges: receipt.sourceChanges,
          library: receipt.library,
          cases: receipt.cases.length,
        })}`,
      );
    } finally {
      if (!hosted && !process.env.VIZE_BACKGROUND_MOTION_EVIDENCE)
        await rm(output, { recursive: true, force: true });
    }
  },
);
