import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { createArchiveReceipt, verifyArchiveReceipt } from "./rust-test-archive.mjs";

const workspaceRoot = fileURLToPath(new URL("../../../../", import.meta.url));

void test("archive receipt accepts the same pinned nextest release across GNU and musl", async (t) => {
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-nextest-receipt-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const archive = path.join(directory, "tests.tar.zst");
  writeFileSync(archive, "archive bytes");
  const common = {
    cwd: workspaceRoot,
    rustcVersion: "rustc 1.98.0 (pinned)",
    env: { VIZE_TEST_DISABLE_TSGO: "1", VIZE_NUXT_CONFIG_ITERATIONS: "100" },
  };
  const stamped = await createArchiveReceipt(archive, {
    ...common,
    nextestVersion: "cargo-nextest 0.9.146 (x86_64-unknown-linux-gnu)",
  });
  await verifyArchiveReceipt(stamped, archive, {
    ...common,
    nextestVersion: "cargo-nextest 0.9.146 (x86_64-unknown-linux-musl)",
  });
  await assert.rejects(
    verifyArchiveReceipt(stamped, archive, {
      ...common,
      nextestVersion: "cargo-nextest 0.9.147 (x86_64-unknown-linux-musl)",
    }),
    /pinned CI toolchain/,
  );
});
