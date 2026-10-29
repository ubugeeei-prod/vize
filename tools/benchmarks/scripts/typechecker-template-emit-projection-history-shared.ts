import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { projectionFamilies } from "./typechecker-template-emit-projection-history-model.ts";
import type {
  ArtifactCustody,
  HistoricalEdit,
  ProjectionBytes,
} from "./typechecker-template-emit-projection-history-model.ts";

interface ByteLine {
  start: number;
  end: number;
  bytes: Buffer;
}

interface PendingEdit {
  leftStart: number;
  rightStart: number;
}

/**
 * Preserve the last successful projection receipt and the first failing
 * scheduled packet as independent evidence. Qualification has three layers:
 * exact artifact custody, byte-exact helper inverses, and fail-closed generated
 * coordinate inverses. Current output is checked separately and is never
 * rewritten to force an old digest.
 */
export const retainedFamilies = projectionFamilies;
export const retainedManifestUrl = new URL(
  "../../../tests/_fixtures/differential/typechecker/template-dollar-emit/projection-retained-10907.manifest.json",
  import.meta.url,
);
export const retainedArtifactCustody: ArtifactCustody = {
  baseline: {
    sourceSha: "c62d96e4b2267987105f9d9d2698a000bf73aad9",
    driverSha: "c62d96e4b2267987105f9d9d2698a000bf73aad9",
    runId: 38236803949,
    jobId: 114769365483,
    artifactId: 11741738141,
    artifactBytes: 96630181,
    createdAt: "2026-10-12T10:45:14Z",
    expiresAt: "2026-11-11T10:45:13Z",
    archiveBytes: 96628392,
    archiveSha256: "3cdce277d4b2bccb725c95135b0c54fa7085ea2af2c1e9d9966ba4659917af48",
    manifestSha256: "92fd955ff55e3e68ac4f74264874eb4a2ae454f9a12088db4fc4142cf8f33361",
    receiptVerifierSha256: "ab51a5f0b704c31eece8af5614ff58fb027f97d325d66b401014f4ece37732c5",
    qualification: "success",
  },
  incident: {
    sourceSha: "b09ac660cd0684396c21d24249c4b1e2193e965a",
    driverSha: "b09ac660cd0684396c21d24249c4b1e2193e965a",
    runId: 40415562052,
    jobId: 121896883497,
    artifactId: 12694328570,
    artifactBytes: 101632777,
    createdAt: "2026-10-26T22:58:12Z",
    expiresAt: "2026-11-25T22:58:08Z",
    archiveBytes: 101630987,
    archiveSha256: "f14356db34626150e0f8056f16dff2ffc2beb36ebfa1652ada79074cbadb80f5",
    manifestSha256: "6592c2a4b0be8caeb2f6bbac8f7d6bf353fc1f577cdea42fc02fd142c1ef705f",
    qualification: "failed-before-receipt",
  },
};
// Updated only after regenerating the fixture twice from the two immutable
// artifacts and observing byte-identical output.
export const retainedManifestSha256: string =
  "559ea18dbbde55fac6eaf80f63a5e32959672c11d0698cff9f4579e95caee4cc";

export const sha256 = (bytes: ProjectionBytes): string =>
  createHash("sha256").update(bytes).digest("hex");
export const asBuffer = (value: ProjectionBytes): Buffer =>
  Buffer.isBuffer(value) ? value : Buffer.from(value);

function lines(bytes: Buffer): ByteLine[] {
  const rows: ByteLine[] = [];
  let start = 0;
  for (let index = 0; index < bytes.length; index += 1) {
    if (bytes[index] === 0x0a) {
      rows.push({ start, end: index + 1, bytes: bytes.subarray(start, index + 1) });
      start = index + 1;
    }
  }
  if (start < bytes.length) rows.push({ start, end: bytes.length, bytes: bytes.subarray(start) });
  return rows;
}

function boundary(rows: readonly ByteLine[], index: number, length: number): number {
  return index === rows.length ? length : rows[index].start;
}

function trimEdit(before: Buffer, after: Buffer, edit: HistoricalEdit): HistoricalEdit {
  while (
    edit.beforeStart < edit.beforeEnd &&
    edit.afterStart < edit.afterEnd &&
    before[edit.beforeStart] === after[edit.afterStart]
  ) {
    edit.beforeStart += 1;
    edit.afterStart += 1;
  }
  while (
    edit.beforeStart < edit.beforeEnd &&
    edit.afterStart < edit.afterEnd &&
    before[edit.beforeEnd - 1] === after[edit.afterEnd - 1]
  ) {
    edit.beforeEnd -= 1;
    edit.afterEnd -= 1;
  }
  return edit;
}

// The retained programs are line-oriented generated TypeScript. A line LCS keeps
// independent helper changes independent, then byte trimming narrows single-line
// replacements to their exact payload. The result is stored, not recomputed, by
// qualification so a later diff implementation cannot reinterpret the evidence.
export function diffGeneratedBytes(
  beforeValue: ProjectionBytes,
  afterValue: ProjectionBytes,
): HistoricalEdit[] {
  const before = asBuffer(beforeValue);
  const after = asBuffer(afterValue);
  const left = lines(before);
  const right = lines(after);
  assert(left.length < 65_536 && right.length < 65_536, "retained line count overflow");
  const table = Array.from({ length: left.length + 1 }, () => new Uint16Array(right.length + 1));
  for (let leftIndex = left.length - 1; leftIndex >= 0; leftIndex -= 1) {
    for (let rightIndex = right.length - 1; rightIndex >= 0; rightIndex -= 1) {
      table[leftIndex][rightIndex] = left[leftIndex].bytes.equals(right[rightIndex].bytes)
        ? table[leftIndex + 1][rightIndex + 1] + 1
        : Math.max(table[leftIndex + 1][rightIndex], table[leftIndex][rightIndex + 1]);
    }
  }
  const edits: HistoricalEdit[] = [];
  let leftIndex = 0;
  let rightIndex = 0;
  let pending: PendingEdit | null = null;
  const flush = () => {
    if (!pending) return;
    const edit = trimEdit(before, after, {
      beforeStart: boundary(left, pending.leftStart, before.length),
      beforeEnd: boundary(left, leftIndex, before.length),
      afterStart: boundary(right, pending.rightStart, after.length),
      afterEnd: boundary(right, rightIndex, after.length),
    });
    if (edit.beforeStart !== edit.beforeEnd || edit.afterStart !== edit.afterEnd) edits.push(edit);
    pending = null;
  };
  while (leftIndex < left.length || rightIndex < right.length) {
    if (
      leftIndex < left.length &&
      rightIndex < right.length &&
      left[leftIndex].bytes.equals(right[rightIndex].bytes)
    ) {
      flush();
      leftIndex += 1;
      rightIndex += 1;
    } else {
      pending ??= { leftStart: leftIndex, rightStart: rightIndex };
      if (
        rightIndex === right.length ||
        (leftIndex < left.length &&
          table[leftIndex + 1][rightIndex] >= table[leftIndex][rightIndex + 1])
      )
        leftIndex += 1;
      else rightIndex += 1;
    }
  }
  flush();
  assert.deepEqual(applyInverseEdits(after, before, edits), before);
  return edits;
}

/** Reconstruct the successful generated document from a newer retained packet. */
export function applyInverseEdits(
  afterValue: ProjectionBytes,
  beforeValue: ProjectionBytes,
  edits: readonly HistoricalEdit[],
): Buffer {
  const after = asBuffer(afterValue);
  const before = asBuffer(beforeValue);
  const chunks: Uint8Array[] = [];
  let cursor = 0;
  let previousBefore = 0;
  for (const edit of edits) {
    for (const key of ["beforeStart", "beforeEnd", "afterStart", "afterEnd"] as const)
      assert(Number.isSafeInteger(edit[key]) && edit[key] >= 0, `invalid edit ${key}`);
    assert(edit.beforeStart >= previousBefore && edit.beforeEnd >= edit.beforeStart);
    assert(edit.afterStart >= cursor && edit.afterEnd >= edit.afterStart);
    assert(edit.beforeEnd <= before.length && edit.afterEnd <= after.length);
    assert(
      edit.beforeStart !== edit.beforeEnd || edit.afterStart !== edit.afterEnd,
      "empty historical edit",
    );
    chunks.push(after.subarray(cursor, edit.afterStart));
    chunks.push(before.subarray(edit.beforeStart, edit.beforeEnd));
    cursor = edit.afterEnd;
    previousBefore = edit.beforeEnd;
  }
  chunks.push(after.subarray(cursor));
  return Buffer.concat(chunks);
}

/** Map a stable generated offset back through every retained helper edit. */
export function historicalOffset(value: number | string, edits: readonly HistoricalEdit[]): number {
  const offset = Number(value);
  assert(Number.isSafeInteger(offset) && offset >= 0, "invalid generated offset");
  let delta = 0;
  for (const edit of edits) {
    const beforeLength = edit.beforeEnd - edit.beforeStart;
    const afterLength = edit.afterEnd - edit.afterStart;
    if (offset < edit.afterStart) return offset + delta;
    if (offset === edit.afterStart) {
      assert(afterLength > 0 || beforeLength === 0, "offset meets a deleted historical range");
      return edit.beforeStart;
    }
    assert(offset >= edit.afterEnd, "generated offset entered a changed helper range");
    if (offset === edit.afterEnd) return edit.beforeEnd;
    delta += beforeLength - afterLength;
  }
  return offset + delta;
}

/** Map a stable generated interval and reject any interval touching changed code. */
export function historicalInterval(
  start: number | string,
  length: number | string,
  edits: readonly HistoricalEdit[],
): [number, number] {
  const currentStart = Number(start);
  const currentEnd = currentStart + Number(length);
  assert(Number.isSafeInteger(currentEnd) && Number(length) >= 0, "invalid generated interval");
  for (const edit of edits) {
    assert(
      currentEnd <= edit.afterStart || currentStart >= edit.afterEnd,
      "generated interval overlaps a changed helper range",
    );
  }
  const beforeStart = historicalOffset(currentStart, edits);
  const beforeEnd = historicalOffset(currentEnd, edits);
  return [beforeStart, beforeEnd - beforeStart];
}
