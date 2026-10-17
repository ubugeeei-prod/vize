import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { parse } from "yaml";

interface Step {
  readonly name?: string;
  readonly if?: string;
  readonly run?: string;
  readonly uses?: string;
  readonly env?: Readonly<Record<string, string>>;
  readonly with?: Readonly<Record<string, unknown>>;
}

interface Job {
  readonly if?: string;
  readonly needs?: string | readonly string[];
  readonly steps?: readonly Step[];
  readonly uses?: string;
}
interface Workflow {
  readonly jobs: Readonly<Record<string, Job>>;
}

function workflow(file: string): Workflow {
  return parse(readFileSync(new URL(`../../.github/workflows/${file}`, import.meta.url), "utf8"));
}

const enabled = "needs.channel.outputs.enabled == 'true'";
const jsr = workflow("release-jsr.yml");
const publishSteps = jsr.jobs.publish?.steps ?? [];
const runs = (step: Step, command: RegExp) => command.test(step.run ?? "");

await test("every release runs the JSR publish dry-run, whatever the channel says", () => {
  assert.equal(jsr.jobs.publish?.if, undefined, "the publish job itself is never skipped");
  const dryRuns = publishSteps.filter((step) => runs(step, /deno publish --dry-run/));
  assert.equal(dryRuns.length, 1);
  assert.equal(dryRuns[0]?.if, undefined, "the dry-run has no condition");
});

await test("only an enabled channel publishes to JSR and runs the consumers", () => {
  const publications = publishSteps.filter(
    (step) => runs(step, /deno publish/) && !runs(step, /--dry-run/),
  );
  assert.equal(publications.length, 1);
  assert.equal(publications[0]?.if, enabled);
  assert.equal(jsr.jobs.consumer?.if, enabled);
});

await test("the release calls the JSR workflow after the npm packages it re-exports", () => {
  const release = workflow("release.yml").jobs["release-jsr"];
  assert.equal(release?.uses, "./.github/workflows/release-jsr.yml");
  const needs = new Set([release?.needs ?? []].flat());
  for (const job of ["release-npm-cli", "release-npm-native", "release-npm-vite-plugin"]) {
    assert.ok(needs.has(job), `release-jsr waits for ${job}`);
  }
});

for (const [file, job, condition] of [
  ["release-jsr.yml", "consumer", "matrix.runtime == 'deno'"],
  ["jsr-package.yml", "package", undefined],
] as const) {
  await test(`${file} applies the shared scoped age policy only to Deno`, () => {
    const steps = workflow(file).jobs[job]?.steps ?? [];
    const deno = steps.filter((step) => runs(step, /\bdeno run\b/));
    assert.equal(deno.length, 1);
    assert.equal(deno[0]?.if, condition);
    const script = deno[0]?.run ?? "";
    const copied = script.match(
      /\bcp\s+"?(?:\$GITHUB_WORKSPACE\/)?tools\/support\/release\/jsr\/consumer-deno\.json"?\s+"?([^"\s]+)/,
    );
    assert.ok(copied, "copy the canonical policy into the fresh consumer");
    const config = script.match(/\bdeno run\b[^\n]*--config(?:=|\s+)"?([^"\s]+)/);
    assert.equal(config?.[1], copied[1], "explicitly load the copied policy");
    assert.deepEqual(
      steps.filter((step) => runs(step, /consumer-deno\.json/)),
      deno,
      "Node/Bun and shared setup never load the Deno policy",
    );
    assert.ok(steps.every((step) => !runs(step, /--minimum-dependency-age/)));
  });
}

await test("the hosted package check installs frozen tooling before importing yaml", () => {
  const steps = workflow("jsr-package.yml").jobs.package?.steps ?? [];
  const setup = steps.findIndex((step) => step.uses?.startsWith("voidzero-dev/setup-vp@"));
  const install = steps.findIndex((step) => runs(step, /^vp install /));
  const tests = steps.findIndex((step) => runs(step, /^node --test /));
  assert.ok(setup >= 0 && setup < install && install < tests);
  assert.equal(steps[setup]?.with?.["run-install"], false);
  assert.equal(
    steps[install]?.run,
    "vp install --frozen-lockfile --prefer-offline --filter vize-workspace --ignore-scripts",
  );
});

await test("pinned Deno runtime proof retains independent fresh-cache controls and public evidence", () => {
  const steps = workflow("jsr-package.yml").jobs.package?.steps ?? [];
  const deno = steps.findIndex((step) => step.uses?.startsWith("denoland/setup-deno@"));
  const proof = steps.findIndex(
    (step) => step.name === "Verify scoped dependency age with the actual Deno runtime",
  );
  assert.ok(deno >= 0 && deno < proof);
  assert.equal(steps[deno]?.with?.["deno-version"], "2.9.7");
  assert.equal(
    steps[proof]?.run,
    'node tools/support/release/jsr/verify-consumer-age.ts "$RUNNER_TEMP/jsr-consumer-age"',
  );
  assert.equal(
    steps[proof]?.env?.JSR_CONSUMER_PUBLISHED_VERSION,
    "${{ github.event_name == 'workflow_dispatch' && inputs.published-version || '' }}",
  );
  const staged = steps.find((step) => step.name === "Import the staged entry points under Deno");
  assert.equal(staged?.env?.DENO_DIR, "${{ runner.temp }}/staged-jsr-deno");
  const upload = steps.find(
    (step) => step.name === "Preserve actual Deno consumer commands and outcomes",
  );
  assert.equal(upload?.if, "${{ always() }}");
  assert.equal(upload?.with?.name, "jsr-consumer-age-${{ github.sha }}-${{ github.run_attempt }}");
  assert.equal(upload?.with?.path, "${{ runner.temp }}/jsr-consumer-age/");
});
