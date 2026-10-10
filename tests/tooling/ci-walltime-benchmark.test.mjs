import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

void test("walltime comparisons use the same source and runner without publishing authority", () => {
  const workflow = parse(
    readFileSync(
      new URL("../../.github/workflows/ci-walltime-benchmark.yml", import.meta.url),
      "utf8",
    ),
  );
  assert.deepEqual(workflow.permissions, { contents: "read" });
  assert.deepEqual(workflow.on.pull_request.types, ["opened", "reopened", "labeled"]);
  assert.deepEqual(Object.keys(workflow.jobs).sort(), ["docs", "js", "native", "rust"]);
  for (const job of Object.values(workflow.jobs)) {
    assert.equal(job["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
    assert.match(job.if, /ci-walltime-benchmark/);
    assert.match(job.if, /startsWith\(github.head_ref, 'ci\/optimize-'\)/);
    assert.equal(job.steps[0].with.ref, "${{ github.sha }}");
    const upload = job.steps.at(-1);
    assert.equal(upload.if, "${{ always() }}");
    assert.equal(upload.with["if-no-files-found"], "error");
  }
  const steps = (job) => workflow.jobs[job].steps.map((step) => step.run ?? "").join("\n");
  assert.match(steps("js"), /--host-mode serial/);
  assert.match(steps("js"), /--host-mode concurrent/);
  assert.match(steps("docs"), /--workers 1/);
  assert.match(steps("docs"), /--workers 2/);
  assert.match(steps("rust"), /cargo test --locked --workspace --no-run/);
  assert.match(steps("rust"), /--runner cargo/);
  assert.match(steps("rust"), /--runner nextest/);
  assert.doesNotMatch(steps("rust"), /--filter|--partition|DISABLE_TSGO/);
  assert.match(steps("native"), /moonbit-publish-concurrency-benchmark\.test\.ts/);
});
