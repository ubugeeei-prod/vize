import assert from "node:assert/strict";

const sha = /^[0-9a-f]{40}$/;

// The observer recipe is supplied by the PR merge actor (or the selected manual
// branch), while historical products retain their own immutable source commits.
export function sourceProvenance(observerActor, env = process.env) {
  assert.match(observerActor, sha, "The actual observer git HEAD must be a full SHA");
  assert.match(env.GITHUB_SHA || "", sha, "The workflow actor must be a full SHA");
  assert.equal(observerActor, env.GITHUB_SHA, "Observer HEAD must equal the workflow actor");
  assert(["pull_request", "workflow_dispatch"].includes(env.GITHUB_EVENT_NAME));
  let triggerHead = null;
  if (env.GITHUB_EVENT_NAME === "pull_request") {
    triggerHead = env.VIZE_PACKAGE_REFRESH_TRIGGER_HEAD;
    assert.match(triggerHead || "", sha, "The PR trigger head must be recorded separately");
  } else {
    assert.equal(env.VIZE_PACKAGE_REFRESH_TRIGGER_HEAD || "", "", "Manual runs have no PR head");
  }
  return { event: env.GITHUB_EVENT_NAME, observerActor, githubSha: env.GITHUB_SHA, triggerHead };
}

export function validateSourceProvenance(value, observerActor, env = process.env) {
  const expected = sourceProvenance(observerActor, env);
  assert.deepEqual(value, expected, "The producer and consumer must bind the same observer actor");
  return expected;
}

export function comparisonSources(observerActor, historical) {
  assert.match(observerActor, sha);
  assert.equal(typeof historical, "boolean");
  return historical
    ? [
        ["before-policy", "13fa41356d075047a229f580014b637d3bbeb57f"],
        ["runtime-policy", "9469f272b189d0220a9838539f2c878fc5d70edd"],
        ["fixed", observerActor],
      ]
    : [["fixed", observerActor]];
}
