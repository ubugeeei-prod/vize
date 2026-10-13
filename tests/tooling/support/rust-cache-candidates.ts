import assert from "node:assert/strict";

type TraceRow = { readonly kind: string; readonly name?: string; readonly commit?: unknown };

// Untrusted candidates mount never-committed clones; failing every mount
// exercises their trusted Actions-seed fallback instead.
export const allMountFailures = ["registry", "git", "target", "secondary"];

export const isCandidate = (event: string) => event === "pull_request" || event === "merge_group";

export function assertNoCandidateCommit(trace: readonly TraceRow[]) {
  for (const row of trace.filter(({ kind }) => kind === "sticky" || kind === "sticky-post"))
    assert.equal(row.commit, "false", `untrusted ${row.kind} for ${row.name} may commit`);
}

/** Candidates may read a clone but never commit it; other untrusted events never mount. */
export function assertUntrustedStickyRows(trace: readonly TraceRow[], event: string) {
  if (isCandidate(event)) assertNoCandidateCommit(trace);
  else
    assert.equal(trace.filter(({ kind }) => kind === "sticky" || kind === "sticky-post").length, 0);
}
