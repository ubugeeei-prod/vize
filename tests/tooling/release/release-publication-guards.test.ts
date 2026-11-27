import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { readAutomationPrograms } from "../support/automation-source.ts";

type Step = { name?: string; env?: Record<string, string>; run?: string };
type Job = { env?: Record<string, string>; steps?: Step[] };
const release = parse(readAutomationPrograms(".github", "workflows", "release.yml")) as {
  jobs: Record<string, Job>;
};

test("a requested release source is one full commit SHA already on main", () => {
  const step = release.jobs.version.steps?.find((candidate) => candidate.env?.RELEASE_SOURCE);
  const run = String(step?.run);
  const shape = run.indexOf('if [[ ! "$RELEASE_SOURCE" =~ ^[0-9a-f]{40}$ ]]; then');
  assert.equal(step?.env?.RELEASE_SOURCE, "${{ inputs.source }}");
  assert.ok(shape >= 0, "the source is checked to be a full commit SHA");
  for (const use of ["gh api", "git fetch"]) {
    assert.ok(shape < run.indexOf(use), `the check precedes ${use}`);
  }
  assert.match(run.slice(shape, run.indexOf("gh api")), /^ +exit 1\nfi$/m);
});

test("the tag lands only on the candidate commit this run stamped", () => {
  const step = release.jobs.tag.steps?.find((candidate) => candidate.run?.includes("git push"));
  const run = String(step?.run);
  const check = run.indexOf('"$head" != "$RELEASE_SHA"');
  assert.equal(step?.env?.RELEASE_SHA, "${{ needs.version.outputs.sha }}");
  assert.match(run, /^head=\$\(git rev-parse HEAD\)$/m);
  assert.ok(check >= 0 && check < run.indexOf("git push"), "the check precedes every push");
  assert.match(run.slice(check, run.indexOf("git push")), /^ +exit 1\nfi$/m);
});

test("each marketplace token reaches only the step that publishes with it", () => {
  for (const [name, token] of [
    ["publish-vscode", "VSCE_PAT"],
    ["publish-vscode-platforms", "VSCE_PAT"],
    ["publish-open-vsx", "OVSX_PAT"],
    ["publish-open-vsx-platforms", "OVSX_PAT"],
  ] as const) {
    const job = release.jobs[name];
    const steps = job.steps ?? [];
    const publish = steps.at(-1);
    assert.match(String(publish?.run), /publish_(?:vscode|open_vsx)_extension\/main\.ts/, name);
    assert.deepEqual(publish?.env, { [token]: `\${{ secrets.${token} }}` }, name);
    assert.doesNotMatch(JSON.stringify([job.env, steps.slice(0, -1)]), /secrets\./, name);
  }
});
