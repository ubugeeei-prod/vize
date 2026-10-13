import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

const read = (name) => fs.readFileSync(new URL(`../../${name}`, import.meta.url), "utf8");
const original = parse(read(".github/workflows/release-typed-defaults-acceptance.yml"));
const supplement = parse(read(".github/workflows/release-setup-emits-acceptance.yml"));

await test("installed setup357 has a distinct main-only manual workflow and preserves old392 guard", () => {
  assert.deepEqual(Object.keys(supplement.on), ["workflow_dispatch"]);
  assert.deepEqual(supplement.on, original.on);
  assert.deepEqual(supplement.permissions, original.permissions);
  assert.equal(supplement.concurrency["cancel-in-progress"], false);
  assert.equal(supplement.concurrency.group, "released-setup-emits-${{ inputs.release_run }}");
  const job = supplement.jobs["public-cli"],
    old = original.jobs["public-cli"];
  assert.equal(job["runs-on"], old["runs-on"]);
  assert.deepEqual(job.env, old.env);
  for (const value of Object.values(job.env)) assert.doesNotMatch(value, /\$\{\{\s*runner\./);
  assert.equal(job.steps[0].run, old.steps[0].run);
  assert.equal(
    job.steps[0].env.PUB_CONSUMER,
    "${{ runner.temp }}/vize-setup-emits-consumer-${{ github.run_id }}-${{ github.run_attempt }}",
  );
  assert.equal(
    job.steps[0].env.PUBLIC_OUTPUT,
    "${{ runner.temp }}/vize-setup-emits-evidence-${{ github.run_id }}-${{ github.run_attempt }}",
  );
  assert.deepEqual(job.steps[1], old.steps[1]);
  assert.deepEqual(job.steps[2], old.steps[2]);
  assert.deepEqual(job.steps[3].env, old.steps[3].env);
  assert.equal(job.steps[3].run, "node tools/support/release/typed_defaults/setup_emits_main.mjs");
  assert.equal(job.steps[4].if, "always()");
  assert.equal(job.steps[4].with["if-no-files-found"], "error");
  assert.equal(job.steps[4].with.path, job.steps[0].env.PUBLIC_OUTPUT);
  assert.equal(job.steps[4].with["include-hidden-files"], true);
  assert.equal(old.steps[3].run, "node tools/support/release/typed_defaults/main.mjs");
});
