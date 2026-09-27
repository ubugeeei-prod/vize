import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import {
  createArchiveReceipt,
  verifyArchiveReceipt,
} from "../../tools/support/compat/github/rust-test-archive.mjs";

for (const job of ["pr-rust-build", "merge-rust-source"]) {
  void test(`${job} failed-only retry selects its completed producer's exact archive`, async () => {
    const workflow = parse(
      readFileSync(new URL("../../.github/workflows/pr-rust-checks.yml", import.meta.url), "utf8"),
    );
    const upload = workflow.jobs[job].steps.find((step) =>
      step.uses?.startsWith("actions/upload-artifact@"),
    );
    const download = workflow.jobs["pr-rust-shard"].steps.find((step) =>
      step.uses?.startsWith("actions/download-artifact@"),
    );
    assert.equal(upload.with.overwrite, true);
    const name = (template, context) =>
      template
        .replaceAll("${{ github.run_id }}", context.runId)
        .replaceAll("${{ github.sha }}", context.sha)
        .replaceAll("${{ github.run_attempt }}", context.attempt);
    const cwd = mkdtempSync(join(tmpdir(), "vize-rust-retry-"));
    const git = (...args) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
    try {
      git("init", "-q");
      git("config", "user.name", "CI Test");
      git("config", "user.email", "ci@example.invalid");
      writeFileSync(join(cwd, "fixture"), "source\n");
      git("add", "fixture");
      git("commit", "-qm", "source identity");
      const producer = { runId: "36308996672", sha: git("rev-parse", "HEAD"), attempt: "1" };
      const directory = join(cwd, "artifacts", name(upload.with.name, producer));
      mkdirSync(directory, { recursive: true });
      const archive = join(directory, "tests.tar.zst");
      const receiptPath = join(directory, "receipt.json");
      const context = {
        cwd,
        nextestVersion: "cargo-nextest 0.9.146",
        rustcVersion: "rustc 1.98.0 (fixture)",
        env: {
          VIZE_NUXT_CONFIG_ITERATIONS: "100",
          ...(job === "merge-rust-source"
            ? { VIZE_TEST_REQUIRE_TSGO: "1" }
            : { VIZE_TEST_DISABLE_TSGO: "1" }),
        },
      };
      writeFileSync(archive, "producer attempt 1 archive bytes");
      const first = await createArchiveReceipt(archive, context);
      writeFileSync(receiptPath, JSON.stringify(first));

      // Retry the failed consumer only: there is deliberately no attempt-2 producer.
      const retry = { ...producer, attempt: "2" };
      const selected = join(cwd, "artifacts", name(download.with.name, retry));
      await verifyArchiveReceipt(
        JSON.parse(readFileSync(join(selected, "receipt.json"), "utf8")),
        join(selected, "tests.tar.zst"),
        context,
      );
      for (const changed of [{ runId: "36308996673" }, { sha: "f".repeat(40) }]) {
        const unrelated = join(
          cwd,
          "artifacts",
          name(download.with.name, { ...retry, ...changed }),
        );
        assert.throws(() => readFileSync(join(unrelated, "receipt.json")), { code: "ENOENT" });
      }

      // A completed full rerun replaces that same exact artifact and receipt together.
      assert.equal(
        name(upload.with.name, { ...producer, attempt: "3" }),
        name(download.with.name, retry),
      );
      writeFileSync(archive, "full rerun archive bytes");
      await assert.rejects(verifyArchiveReceipt(first, archive, context), /archiveSha256 mismatch/);
      writeFileSync(receiptPath, JSON.stringify(await createArchiveReceipt(archive, context)));
      await verifyArchiveReceipt(JSON.parse(readFileSync(receiptPath, "utf8")), archive, context);
      writeFileSync(join(cwd, "fixture"), "different source\n");
      git("commit", "-qam", "changed source");
      await assert.rejects(
        verifyArchiveReceipt(JSON.parse(readFileSync(receiptPath, "utf8")), archive, context),
        /sha mismatch/,
      );
    } finally {
      rmSync(cwd, { recursive: true, force: true });
    }
  });
}
