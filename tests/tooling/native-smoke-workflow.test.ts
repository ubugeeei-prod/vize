import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

function workflowJobBody(workflow: string, jobName: string): string {
  const jobStart = workflow.indexOf(`\n  ${jobName}:\n`);
  assert.notEqual(jobStart, -1, `${jobName} job missing`);
  const remaining = workflow.slice(jobStart + 1);
  const nextJobMatch = /\n  [a-z0-9-]+:\n/g.exec(remaining.slice(1));
  return remaining.slice(0, nextJobMatch ? nextJobMatch.index + 1 : undefined);
}

test("native smoke keeps MoonBit out of the npm fresh install path", () => {
  const workflow = readRepoFile(".github", "workflows", "native-smoke.yml");
  const hostJob = workflowJobBody(workflow, "host-native-smoke");
  const freshJob = workflowJobBody(workflow, "fresh-install-smoke");

  assert.match(hostJob, /target:\s*darwin-x64/);
  assert.match(
    hostJob,
    /uses:\s*\.\/\.github\/actions\/setup-moonbit\s*\n\s*if:\s*matrix\.target != 'darwin-x64'/,
  );
  assert.match(freshJob, /target:\s*darwin-x64/);
  assert.doesNotMatch(freshJob, /setup-moonbit/);
  assert.match(freshJob, /vp run --filter '\.\/npm\/cli' build/);
});
