import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import {
  createArchiveReceipt,
  verifyArchiveReceipt,
} from "../../tools/support/compat/github/rust-test-archive.mjs";
import { requireRustTier } from "../../tools/support/compat/github/require-rust-tier.mjs";

void test("Rust archives reject different source, baked paths, runner, nextest, and contents", async () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-rust-archive-"));
  const git = (...args) => execFileSync("git", args, { cwd, encoding: "utf8" });
  try {
    git("init", "-q");
    git("config", "user.name", "CI Test");
    git("config", "user.email", "ci@example.invalid");
    writeFileSync(join(cwd, "fixture"), "source\n");
    git("add", ".");
    git("commit", "-qm", "fixture");
    const archive = join(cwd, "tests.tar.zst");
    writeFileSync(archive, "archive bytes");
    const context = {
      cwd,
      nextestVersion: "cargo-nextest 0.9.146",
      rustcVersion: "rustc 1.98.0 (fixture)",
      env: { VIZE_TEST_REQUIRE_TSGO: "1", VIZE_NUXT_CONFIG_ITERATIONS: "100" },
    };
    const receipt = await createArchiveReceipt(archive, context);
    await verifyArchiveReceipt(receipt, archive, context);
    for (const key of [
      "schemaVersion",
      "sha",
      "tree",
      "workspaceRoot",
      "platform",
      "arch",
      "nextestVersion",
      "rustcVersion",
      "cargoProfile",
      "requireTsgo",
      "disableTsgo",
      "nuxtIterations",
      "archiveSha256",
    ]) {
      await assert.rejects(
        verifyArchiveReceipt({ ...receipt, [key]: "different" }, archive, context),
        { message: `Rust test archive ${key} mismatch` },
      );
    }
    await assert.rejects(
      verifyArchiveReceipt(receipt, archive, {
        ...context,
        env: { VIZE_TEST_DISABLE_TSGO: "1", VIZE_NUXT_CONFIG_ITERATIONS: "100" },
      }),
      { message: "Rust test archive requireTsgo mismatch" },
    );
    for (const env of [
      {},
      {
        VIZE_TEST_REQUIRE_TSGO: "1",
        VIZE_TEST_DISABLE_TSGO: "",
        VIZE_NUXT_CONFIG_ITERATIONS: "100",
      },
      {
        VIZE_TEST_REQUIRE_TSGO: "",
        VIZE_TEST_DISABLE_TSGO: "1",
        VIZE_NUXT_CONFIG_ITERATIONS: "100",
      },
      {
        VIZE_TEST_REQUIRE_TSGO: "1",
        VIZE_TEST_DISABLE_TSGO: "1",
        VIZE_NUXT_CONFIG_ITERATIONS: "100",
      },
    ]) {
      await assert.rejects(
        createArchiveReceipt(archive, { ...context, env }),
        /pinned CI toolchain and TSGO runtime envelope/,
      );
    }
    writeFileSync(archive, "corrupted bytes");
    await assert.rejects(verifyArchiveReceipt(receipt, archive, context), {
      message: "Rust test archive archiveSha256 mismatch",
    });
    writeFileSync(join(cwd, "fixture"), "different source\n");
    git("add", "fixture");
    git("commit", "-qm", "changed source");
    await assert.rejects(verifyArchiveReceipt(receipt, archive, context), {
      message: "Rust test archive sha mismatch",
    });
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

void test("Rust tier gate rejects failed, cancelled, absent, and skipped required shards", () => {
  const success = { result: "success" };
  const needs = {
    "merge-rust-source": success,
    "pr-rust-build": success,
    "pr-rust-shard": success,
  };
  assert.equal(requireRustTier("pull_request", "true", needs).exitCode, 0);
  for (const [event, jobs] of [
    ["pull_request", ["pr-rust-build", "pr-rust-shard"]],
    ["merge_group", ["merge-rust-source", "pr-rust-shard"]],
  ])
    for (const job of jobs) {
      for (const result of ["failure", "cancelled", "skipped"]) {
        assert.equal(requireRustTier(event, "true", { ...needs, [job]: { result } }).exitCode, 1);
      }
      const absent = { ...needs };
      delete absent[job];
      assert.throws(() => requireRustTier(event, "true", absent), {
        message: `Missing Rust tier job: ${job}`,
      });
    }
  assert.equal(
    requireRustTier("merge_group", "true", { ...needs, "merge-rust-source": { result: "skipped" } })
      .exitCode,
    1,
  );
  assert.equal(requireRustTier("pull_request", "false", needs).exitCode, 0);
  assert.throws(() => requireRustTier("merge_group", "false", needs), {
    message: "Merge queue must select the complete Rust workspace",
  });
  assert.throws(() => requireRustTier("pull_request", "", needs), {
    message: "Missing Rust source plan",
  });
});
