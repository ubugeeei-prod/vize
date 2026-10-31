import assert from "node:assert/strict";
import {
  requireCompletedBefore,
  type NativeTimedObservation,
  type nativeOwnerTerminalState,
} from "./native-io.ts";
import { micros, type WatcherTimedIo, type timedPackets } from "./stable-watcher-timing.ts";

type Owner = ReturnType<typeof nativeOwnerTerminalState>;
type TimedPacket = ReturnType<typeof timedPackets>[number];
const one = <T>(rows: readonly T[]) => {
  assert.equal(rows.length, 1, "one actual owned fragment provenance required");
  return rows[0]!;
};
function completedBefore(earlier: NativeTimedObservation, later: NativeTimedObservation) {
  assert.ok(earlier.completion);
  if (earlier.tid === later.tid && earlier.line < (later.startedLine ?? later.line)) return true;
  const bound = earlier.completion.returnedAt ?? earlier.completion.returnedBy?.at;
  return bound !== undefined && bound !== null && micros(bound) < micros(later.at);
}
export function threadExitBound(event: NativeTimedObservation, owner: Owner) {
  assert.equal(event.completion?.returnedAt, null, "no point-return claim");
  const upper = event.completion?.returnedBy;
  assert.ok(upper && upper.basis === "next-same-thread-event" && upper.line > event.line);
  assert.ok(micros(event.at) < micros(upper.at));
  return one(
    owner.threadExits.filter(
      (row) =>
        row.tid === event.tid && row.line === upper.line && row.at === upper.at && row.code === 0,
    ),
  );
}

/** A completed frame is retained without inventing cross-thread return order. */
export function historicalRootAckFragments(
  ack: TimedPacket,
  eof: NativeTimedObservation,
  io: readonly WatcherTimedIo[],
  owner: Owner,
) {
  if (ack.parts.every((part) => completedBefore(part, eof))) {
    for (const part of ack.parts) requireCompletedBefore(part, eof);
    return null;
  }
  assert.equal(ack.parts.length, 2, "exact header/body split required");
  const fragments = ack.parts.map((part) => {
    const row = one(
      io.filter((candidate) => candidate.tid === part.tid && candidate.line === part.line),
    );
    assert.equal(row.op, "read");
    assert.equal(row.fd, 0);
    for (const key of ["at", "tid", "line", "startedLine", "completion"] as const)
      assert.deepEqual(row[key], part[key]);
    assert.ok(row.completion && row.data.byteLength > 0);
    return row;
  });
  const [header, body] = fragments as [WatcherTimedIo, WatcherTimedIo];
  const expectedBody = Buffer.from(JSON.stringify(ack.packet));
  assert.deepEqual(Buffer.from(body.data), expectedBody, "whole root ACK body required");
  assert.deepEqual(
    Buffer.from(header.data),
    Buffer.from(`Content-Length: ${expectedBody.length}\r\n\r\n`),
  );
  assert.equal(header.data.byteLength, 22, "whole observed header, no trailing bytes");
  assert.notEqual(header.tid, body.tid);
  assert.equal(body.tid, eof.tid);
  assert.equal(
    body.line + 1,
    eof.startedLine ?? eof.line,
    "body immediately precedes EOF in its actual thread",
  );
  assert.equal(body.startedLine ?? body.line, body.line, "no resumed-body surrogate");
  assert.equal(eof.startedLine ?? eof.line, eof.line);
  assert.ok(micros(header.at) < micros(body.at) && micros(body.at) < micros(eof.at));
  requireCompletedBefore(body, eof);
  assert.equal(header.completion!.returnedAt, null);
  const headerExit = threadExitBound(header, owner);
  assert.ok(micros(eof.at) < micros(headerExit.at), "header return bracket overlaps EOF entry");
  const eofExit = threadExitBound(eof, owner);
  const { data: headerData, ...headerEvidence } = header;
  const { data: bodyData, ...bodyEvidence } = body;
  return {
    schema: "vize.nativeStableWatcher.rootAckFragmentOrder.v1",
    header: { ...headerEvidence, hex: Buffer.from(headerData).toString("hex") },
    body: { ...bodyEvidence, hex: Buffer.from(bodyData).toString("hex") },
    headerReturnBeforeBodyEntry: "UNPROVEN" as const,
    headerReturnBeforeEofEntry: "UNPROVEN" as const,
    bodyReturnBeforeEofEntry: "PROVEN" as const,
    headerReturnUpperBoundThreadExit: headerExit,
    eofReturnUpperBoundThreadExit: eofExit,
  };
}
