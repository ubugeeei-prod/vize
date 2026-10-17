import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { qualifyHeadingHashes } from "../../docs/scripts/heading-hash-qualifier.ts";

await test(
  "actual native SSG anchors preserve encoded IDs and browser hash flows",
  { timeout: 120_000 },
  async () => {
    const output = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "docs-heading-hashes-")));
    try {
      const receipt = await qualifyHeadingHashes(output);
      assert.equal(receipt.completed, true);
      assert.equal(receipt.observations.length, 40);
      assert.deepEqual(receipt.errors, []);
    } finally {
      fs.rmSync(output, { recursive: true, force: true });
    }
  },
);
