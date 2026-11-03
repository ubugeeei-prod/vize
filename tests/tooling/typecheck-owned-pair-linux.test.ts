/** Runner integration only: no project benchmark and no timing threshold. */
import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { runOwnedInvocation } from "../../tools/benchmarks/scripts/typecheck-owned-pair-run.ts";

const enabled =
  process.platform === "linux" &&
  process.env.GITHUB_ACTIONS === "true" &&
  process.env.VIZE_PAIR_RUNNER_LABEL === "blacksmith-32vcpu-ubuntu-2404";
test(
  "Linux runner closes raw files, GNU time, fresh cache and separate live tree memory",
  { skip: !enabled },
  async () => {
    const out = mkdtempSync(join(tmpdir(), "owned-pair-linux-test-"));
    try {
      const run = await runOwnedInvocation({
        command: [
          process.execPath,
          "-e",
          "process.stdout.write(Buffer.from([0,255,10])); setTimeout(()=>{},400)",
        ],
        cwd: out,
        out: join(out, "sample"),
        native: null,
        memory: true,
        timeoutMs: 10_000,
      });
      assert.equal(run.raw.status, 0);
      assert.deepEqual(run.raw.stdout, Buffer.from([0, 255, 10]));
      assert.deepEqual(readFileSync(join(out, "sample/stdout.raw")), run.raw.stdout);
      assert.equal(run.raw.stderr.length, 0);
      assert.ok(run.receipt.treeCpuMs != null);
      assert.ok(run.receipt.gnuTime.maxSingleProcessRssKb > 0);
      assert.equal(run.receipt.memoryQualified, true);
      const tree = JSON.parse(readFileSync(join(out, "sample/tree-memory.json"), "utf8"));
      assert.ok(tree.samples.some((sample: { processes: number }) => sample.processes > 0));
      assert.ok(tree.executablePaths.length > 0);
    } finally {
      rmSync(out, { recursive: true, force: true });
    }
  },
);

test(
  "Linux runner retains failed command stdout/stderr and status without retry",
  { skip: !enabled },
  async () => {
    const out = mkdtempSync(join(tmpdir(), "owned-pair-linux-failure-"));
    try {
      const run = await runOwnedInvocation({
        command: [
          process.execPath,
          "-e",
          "process.stdout.write('partial'); process.stderr.write('exact failure\\n'); process.exitCode=7",
        ],
        cwd: out,
        out: join(out, "sample"),
        native: null,
        memory: false,
        timeoutMs: 10_000,
      });
      assert.equal(run.raw.status, 7);
      assert.equal(run.raw.stdout.toString(), "partial");
      assert.equal(run.raw.stderr.toString(), "exact failure\n");
      assert.equal(JSON.parse(readFileSync(join(out, "sample/run.json"), "utf8")).status, 7);
    } finally {
      rmSync(out, { recursive: true, force: true });
    }
  },
);
