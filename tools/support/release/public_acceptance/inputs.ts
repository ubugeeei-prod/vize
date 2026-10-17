import assert from "node:assert/strict";

export function publicInputs(environment: Record<string, string | undefined>) {
  const cut = environment.PUBLIC_CUT ?? "";
  const head = environment.PUBLIC_HEAD ?? "";
  const tag = environment.PUBLIC_TAG ?? "";
  const sourcePr = environment.PUBLIC_SOURCE_PR ?? "";
  const run = environment.PUBLIC_RELEASE_RUN ?? "";
  const recoveryRun = environment.PUBLIC_CONSUMER_RECOVERY_RUN;
  assert.match(cut, /^[0-9a-f]{40}$/u, "full source cut C required");
  assert.match(head, /^[0-9a-f]{40}$/u, "full frozen source H required");
  assert.match(tag, /^v0\.[1-9][0-9]*\.0$/u, "stable minor release tag required");
  for (const [label, value] of [
    ["source PR", sourcePr],
    ["Release run", run],
  ]) {
    assert.match(value, /^[1-9][0-9]*$/u, `${label} must be a positive exact identity`);
    assert.ok(Number.isSafeInteger(Number(value)), label);
  }
  if (recoveryRun) {
    assert.match(recoveryRun, /^[1-9][0-9]*$/u, "positive exact consumer recovery run required");
    assert.ok(Number.isSafeInteger(Number(recoveryRun)), "safe recovery run required");
  }
  return { cut, head, tag, sourcePr, run, ...(recoveryRun ? { recoveryRun } : {}) };
}
