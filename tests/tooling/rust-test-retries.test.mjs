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
import { selectArchive } from "../../tools/support/compat/github/rust-archive-download.ts";

// Only the merge queue still transfers a receipt-bound archive to its shards.
for (const job of ["merge-rust-source"]) {
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
    assert.equal(download.with["artifact-ids"], "${{ steps.rust-archive.outputs.id }}");
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
      writeFileSync(join(cwd, "rust-toolchain.toml"), '[toolchain]\nchannel = "1.99.1"\n');
      git("add", "fixture", "rust-toolchain.toml");
      git("commit", "-qm", "source identity");
      const producer = { runId: "36308996672", sha: git("rev-parse", "HEAD"), attempt: "1" };
      const artifact = {
        id: 901,
        name: name(upload.with.name, producer),
        digest: `sha256:${"a".repeat(64)}`,
        expired: false,
        workflow_run: { id: Number(producer.runId), head_sha: producer.sha },
      };
      const archiveContext = (run) => ({
        run: run.runId,
        sha: run.sha,
        repository: "ubugeeei-prod/vize",
      });
      const directory = join(cwd, "artifacts", String(artifact.id));
      mkdirSync(directory, { recursive: true });
      let archive = join(directory, "tests.tar.zst");
      let receiptPath = join(directory, "receipt.json");
      const context = {
        cwd,
        nextestVersion: "cargo-nextest 0.9.146",
        rustcVersion: "rustc 1.99.1 (fixture)",
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
      const selection = selectArchive([artifact], archiveContext(retry));
      assert.deepEqual(selection, { id: artifact.id, digest: artifact.digest });
      const selected = join(cwd, "artifacts", String(selection.id));
      await verifyArchiveReceipt(
        JSON.parse(readFileSync(join(selected, "receipt.json"), "utf8")),
        join(selected, "tests.tar.zst"),
        context,
      );
      for (const changed of [{ runId: "36308996673" }, { sha: "f".repeat(40) }]) {
        assert.throws(
          () => selectArchive([artifact], archiveContext({ ...retry, ...changed })),
          /Exactly one immutable Rust archive is required/,
        );
      }

      // A full rerun reuses the source-bound name but creates a new immutable ID.
      const replacement = { ...artifact, id: 902, digest: `sha256:${"b".repeat(64)}` };
      assert.equal(name(upload.with.name, { ...producer, attempt: "3" }), replacement.name);
      const rebuilt = selectArchive([replacement], archiveContext(retry));
      assert.notEqual(rebuilt.id, selection.id);
      const rebuiltDirectory = join(cwd, "artifacts", String(rebuilt.id));
      mkdirSync(rebuiltDirectory);
      archive = join(rebuiltDirectory, "tests.tar.zst");
      receiptPath = join(rebuiltDirectory, "receipt.json");
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
