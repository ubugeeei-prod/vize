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
import { requireRustTier } from "../../tools/support/compat/github/require-rust-tier.ts";

void test("Rust archives reject different source, baked paths, runner, nextest, and contents", async () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-rust-archive-"));
  const git = (...args) => execFileSync("git", args, { cwd, encoding: "utf8" });
  try {
    git("init", "-q");
    git("config", "user.name", "CI Test");
    git("config", "user.email", "ci@example.invalid");
    writeFileSync(join(cwd, "fixture"), "source\n");
    writeFileSync(join(cwd, "rust-toolchain.toml"), '[toolchain]\nchannel = "1.99.1"\n');
    git("add", ".");
    git("commit", "-qm", "fixture");
    const archive = join(cwd, "tests.tar.zst");
    writeFileSync(archive, "archive bytes");
    const context = {
      cwd,
      nextestVersion: "cargo-nextest 0.9.146",
      rustcVersion: "rustc 1.99.1 (fixture)",
      env: { VIZE_TEST_REQUIRE_TSGO: "1", VIZE_NUXT_CONFIG_ITERATIONS: "100" },
    };
    const receipt = await createArchiveReceipt(archive, context);
    await assert.rejects(
      createArchiveReceipt(archive, { ...context, rustcVersion: "rustc 1.98.0 (old producer)" }),
      /pinned CI toolchain and TSGO runtime envelope/,
    );
    await assert.rejects(
      createArchiveReceipt(archive, { ...context, rustcVersion: "rustc 1.99.0 (old patch)" }),
      /pinned CI toolchain and TSGO runtime envelope/,
    );
    // An uncommitted downgrade cannot change the source authority.
    writeFileSync(join(cwd, "rust-toolchain.toml"), '[toolchain]\nchannel = "1.99.0"\n');
    await verifyArchiveReceipt(receipt, archive, context);
    writeFileSync(join(cwd, "rust-toolchain.toml"), '[toolchain]\nchannel = "1.99.1"\n');
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
    "merge-rust-differential": success,
    "pr-rust-build": success,
    "pr-rust-shard": success,
  };
  assert.equal(requireRustTier("pull_request", "true", needs).exitCode, 0);
  for (const [event, jobs] of [
    ["pull_request", ["pr-rust-build", "pr-rust-shard"]],
    [
      "merge_group",
      ["merge-rust-source", "merge-rust-differential", "pr-rust-build", "pr-rust-shard"],
    ],
  ]) {
    assert.equal(requireRustTier(event, "true", needs).exitCode, 0);
    for (const job of jobs) {
      for (const result of ["failure", "cancelled", "skipped", "unknown"]) {
        assert.equal(requireRustTier(event, "true", { ...needs, [job]: { result } }).exitCode, 1);
      }
      const absent = { ...needs };
      delete absent[job];
      assert.throws(() => requireRustTier(event, "true", absent), {
        message: `Missing Rust tier job: ${job}`,
      });
      for (const result of [undefined, null, {}]) {
        assert.throws(() => requireRustTier(event, "true", { ...needs, [job]: result }), {
          message: `Job ${job} reported no result in the needs context`,
        });
      }
    }
  }
  assert.equal(
    requireRustTier("merge_group", "true", { ...needs, "merge-rust-source": { result: "skipped" } })
      .exitCode,
    1,
  );
  for (const event of ["pull_request", "merge_group"])
    for (const plan of ["false", "", undefined, null, false, true, {}, "TRUE"])
      assert.throws(() => requireRustTier(event, plan, needs), {
        message: "Rust source checks must run for every pull request and merge group",
      });
  for (const event of ["push", "schedule", "workflow_dispatch", "workflow_call", "", null])
    assert.throws(() => requireRustTier(event, "true", needs), {
      message: "Unexpected Rust tier event",
    });
});

void test("archive compiler authority follows the exact committed reviewed patch", async (t) => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-rust-archive-toolchain-"));
  t.after(() => rmSync(cwd, { recursive: true, force: true }));
  const git = (...args) => execFileSync("git", args, { cwd, encoding: "utf8" });
  git("init", "-q");
  git("config", "user.name", "CI Test");
  git("config", "user.email", "ci@example.invalid");
  const archive = join(cwd, "tests.tar.zst");
  writeFileSync(archive, "archive bytes");
  for (const channel of ["1.99.0", "1.99.1", "1.99.2", "stable"]) {
    writeFileSync(join(cwd, "rust-toolchain.toml"), `[toolchain]\nchannel = "${channel}"\n`);
    git("add", "rust-toolchain.toml");
    git("commit", "-qm", `test: source Rust ${channel}`);
    const context = {
      cwd,
      nextestVersion: "cargo-nextest 0.9.146",
      rustcVersion: `rustc ${channel} (fixture)`,
      env: { VIZE_TEST_REQUIRE_TSGO: "1", VIZE_NUXT_CONFIG_ITERATIONS: "100" },
    };
    if (["1.99.0", "1.99.1"].includes(channel)) {
      const receipt = await createArchiveReceipt(archive, context);
      await verifyArchiveReceipt(receipt, archive, context);
      await assert.rejects(
        createArchiveReceipt(archive, {
          ...context,
          rustcVersion: `rustc ${channel}-nightly (fixture)`,
        }),
        /pinned CI toolchain/,
      );
    } else {
      await assert.rejects(
        createArchiveReceipt(archive, context),
        /reviewed committed source toolchain/,
      );
    }
  }
});
