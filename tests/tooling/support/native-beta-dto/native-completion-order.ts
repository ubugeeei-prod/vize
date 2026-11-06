import assert from "node:assert/strict";
import type { MergedTracePoint } from "./merged-trace.ts";

export type NativeCompletion = {
  returnedAt: string | null;
  returnedBy: {
    at: string;
    line: number;
    basis: "explicit-resume" | "next-same-thread-event";
  } | null;
  merged?: Readonly<{
    captureSha256: string;
    entryLine: number;
    completionLine: number;
  }>;
};
export type NativeTimedObservation = {
  at: string;
  tid: number;
  line: number;
  startedLine?: number;
  completion?: NativeCompletion;
  mergedEntry?: MergedTracePoint;
};

const verifiedMergedPoints = new WeakSet<object>();
const verifiedMergedCompletions = new WeakSet<object>();

export const registerMergedPoint = (point: object) => verifiedMergedPoints.add(point);
export const registerMergedCompletion = (completion: object) =>
  verifiedMergedCompletions.add(completion);

export function requireCompletedBefore(
  earlier: NativeTimedObservation,
  later: NativeTimedObservation,
) {
  assert.ok(earlier.completion, "known completed syscall required");
  if (earlier.completion.merged || later.mergedEntry) {
    const completion = earlier.completion.merged;
    const entry = later.mergedEntry;
    assert.ok(completion && entry, "complete merged provenance required");
    assert.ok(
      verifiedMergedCompletions.has(completion) && verifiedMergedPoints.has(entry),
      "raw-derived merged provenance required",
    );
    assert.equal(
      completion.captureSha256,
      entry.captureSha256,
      "one original merged stream required",
    );
    assert.ok(
      completion.completionLine < entry.line,
      "observed completion must precede later observed entry",
    );
    return;
  }
  if (earlier.tid === later.tid && earlier.line < (later.startedLine ?? later.line)) return;
  const bound = earlier.completion.returnedAt ?? earlier.completion.returnedBy?.at;
  assert.ok(bound, "observed completion order required");
  const time = (at: string) => {
    assert.match(at, /^[0-9]+\.[0-9]{6}$/u);
    return BigInt(at.replace(".", ""));
  };
  assert.ok(time(bound) < time(later.at), "completion must precede later syscall entry");
}

/** Authenticate two observed entries without pretending either one is a kernel return time. */
export function requireObservedEntryBefore(
  earlier: NativeTimedObservation,
  later: NativeTimedObservation,
) {
  if (earlier.mergedEntry || later.mergedEntry) {
    const first = earlier.mergedEntry,
      second = later.mergedEntry;
    assert.ok(first && second, "complete merged entry provenance required");
    assert.ok(
      verifiedMergedPoints.has(first) && verifiedMergedPoints.has(second),
      "raw-derived merged entry provenance required",
    );
    assert.equal(first.captureSha256, second.captureSha256, "one original merged stream required");
    assert.ok(first.line < second.line, "earlier observed entry must precede later observed entry");
    return;
  }
  const time = (at: string) => {
    assert.match(at, /^[0-9]+\.[0-9]{6}$/u);
    return BigInt(at.replace(".", ""));
  };
  assert.ok(time(earlier.at) < time(later.at), "earlier entry must precede later entry");
}

export function compareNativeCompletionOrder(
  left: NativeTimedObservation,
  right: NativeTimedObservation,
): number {
  const leftMerged = left.completion?.merged;
  const rightMerged = right.completion?.merged;
  if (leftMerged || rightMerged) {
    assert.ok(leftMerged && rightMerged, "one complete merged ordering capability required");
    assert.ok(
      verifiedMergedCompletions.has(leftMerged) && verifiedMergedCompletions.has(rightMerged),
      "raw-derived merged completion order required",
    );
    assert.equal(leftMerged.captureSha256, rightMerged.captureSha256);
    return leftMerged.completionLine - rightMerged.completionLine;
  }
  return left.at.localeCompare(right.at) || left.tid - right.tid || left.line - right.line;
}
