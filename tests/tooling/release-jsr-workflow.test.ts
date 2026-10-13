import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { parse } from "yaml";

interface Step {
  readonly if?: string;
  readonly run?: string;
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
