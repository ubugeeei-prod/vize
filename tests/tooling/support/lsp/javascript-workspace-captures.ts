import assert from "node:assert/strict";

export function ownedSessionDirectory(
  before: readonly string[],
  after: readonly string[],
  observerPid: number,
): string {
  assert.ok(Number.isSafeInteger(observerPid) && observerPid > 0);
  // LspSessionCapture prefixes each directory with its observer process PID.
  const added = after.filter(
    (name) => !before.includes(name) && name.startsWith(`${observerPid}-`),
  );
  assert.equal(added.length, 1, "exactly one owned real stdio session");
  return added[0];
}
