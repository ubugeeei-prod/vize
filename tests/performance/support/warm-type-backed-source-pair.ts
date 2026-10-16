import assert from "node:assert/strict";

export function sourcePair(
  environment: Record<string, string | undefined>,
  driver: string,
  git: (args: string[]) => string,
) {
  const head = environment.SOURCE_SHA ?? "";
  const base = environment.SOURCE_BASE_SHA ?? "";
  if (!head && !base) return null;
  assert.ok(head && base, "both literal source-pair inputs are required");
  for (const key of [
    "ORIGINAL_CONTROL_SHA",
    "ORIGINAL_AFTER_SHA",
    "ORIGINAL_AFTER_TREE",
    "ORIGINAL_MANIFEST_SHA256",
    "ORIGINAL_PUBLIC_AUTHORITY",
    "ORIGINAL_PUBLIC_AUTHORITY_SHA256",
  ])
    assert.ok(!environment[key], "source qualification cannot mix cut/public authority");
  for (const sha of [head, base]) assert.match(sha, /^[a-f0-9]{40}$/u);
  assert.equal(environment.GITHUB_EVENT_NAME, "workflow_dispatch");
  assert.equal(head, driver, "the source head must be the literal checkout");
  assert.equal(head, environment.GITHUB_SHA, "the dispatch must select this source head");
  assert.equal(
    head,
    environment.GITHUB_WORKFLOW_SHA,
    "the loaded workflow must be this source head",
  );
  assert.notEqual(head, base, "a source pair needs distinct commits");
  for (const sha of [base, head]) assert.equal(git(["rev-parse", `${sha}^{commit}`]), sha);
  git(["merge-base", "--is-ancestor", base, head]);
  return { head, base, authority: "immutable-source-pair-qualification" };
}

export function assertUnchangedNonCargoLocks(
  before: Record<string, string>,
  after: Record<string, string>,
) {
  const other = (locks: Record<string, string>) =>
    Object.fromEntries(
      Object.entries(locks).filter(([name]) => !["Cargo.toml", "Cargo.lock"].includes(name)),
    );
  assert.deepEqual(
    other(before),
    other(after),
    "all non-Cargo source/runtime/toolchain locks stay identical",
  );
}
