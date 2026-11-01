import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { isDeepStrictEqual } from "node:util";
import {
  historicalWarningTranscript,
  type HistoricalWarningArgs as Args,
} from "./historical-inflight-transcript.ts";
import {
  historicalRootWarning,
  observeRootWatchWarning as observePreviousRootWarning,
} from "./historical-post-ack-root-warning.ts";
import { requireCompletedBefore, type NativeTimedObservation } from "./native-io.ts";
import { micros, timedPackets } from "./stable-watcher-timing.ts";
export { historicalRootWarning } from "./historical-post-ack-root-warning.ts";

const one = <T>(rows: readonly T[]) => {
  assert.equal(rows.length, 1);
  return rows[0]!;
};
const terminalPoint = <T extends { tid: number; at: string; line?: number }>(row: T) => {
  assert.ok(typeof row.line === "number" && Number.isSafeInteger(row.line));
  return { ...row, line: row.line };
};
const acknowledgedHeaderWarning = (bytes: Buffer) =>
  /^Added new watch: root files for (\/tmp\/[^\r\n]+)\/tsconfig\.json watcher 6\.0\n\t\1\/\*\*\/\*\n\n1 paths ineligible for watching\n$/u.test(
    bytes.toString("utf8"),
  );

/** Dispatch only; the source/owner/channel/EOF contract is checked by the observer. */
export const matchesHistoricalRootWarning = (bytes: Buffer) =>
  bytes.equals(historicalRootWarning) || acknowledgedHeaderWarning(bytes);

/** A fully captured acknowledged-root observation never grants native or lifecycle credit. */
export function observeAcknowledgedRootWarning(...args: Args) {
  const [client, server, context, io, eofReads, owner, api] = args;
  const proof = historicalWarningTranscript(client, server, context, api);
  assert.equal(api.precedingNonProtocolBytes, 5);
  const input = timedPackets(io, "read", 0),
    output = timedPackets(io, "write", 1);
  assert.deepEqual(
    input.map((row) => row.packet),
    client,
  );
  assert.deepEqual(
    output.map((row) => row.packet),
    server,
  );
  const locate = (rows: typeof input, expected: unknown) =>
    one(rows.filter((row) => isDeepStrictEqual(row.packet, expected)));
  const watcher = locate(output, proof.watcher),
    watcherAck = locate(input, proof.watcherAck),
    shutdownAck = locate(output, proof.shutdownAck);
  assert.ok(server.indexOf(proof.configuration) < server.indexOf(proof.watcher));
  assert.ok(server.indexOf(proof.watcher) < server.indexOf(proof.shutdownAck));
  const members = new Map(owner.members.map((row) => [row.tid, row.parent]));
  assert.equal(members.size, owner.members.length);
  assert.ok(Number.isSafeInteger(owner.nativeOwner) && owner.nativeOwner > 0);
  assert.equal(members.get(owner.nativeOwner), null);
  for (const tid of members.keys()) {
    assert.ok(Number.isSafeInteger(tid) && tid > 0);
    const seen = new Set<number>();
    let current = tid;
    while (current !== owner.nativeOwner) {
      assert.ok(!seen.has(current));
      seen.add(current);
      const parent = members.get(current);
      assert.ok(typeof parent === "number" && members.has(parent));
      current = parent;
    }
  }
  const exits = new Map(owner.threadExits.map((row) => [row.tid, row]));
  assert.equal(exits.size, owner.threadExits.length);
  assert.deepEqual(
    [...members.keys()].sort((a, b) => a - b),
    [...exits.keys()].sort((a, b) => a - b),
  );
  const group = one(owner.exitGroups);
  assert.ok(members.has(group.tid));
  assert.equal(group.code, 0);
  assert.equal(group.rawResult, "?");
  assert.ok(Number.isSafeInteger(group.line));
  for (const exit of exits.values()) {
    assert.equal(exit.code, 0);
    assert.ok(micros(group.at) < micros(exit.at));
    assert.ok(Number.isSafeInteger(exit.line));
  }
  for (const row of io) {
    assert.ok(members.has(row.tid));
    assert.ok(Number.isSafeInteger(row.line) && row.line >= 0);
    assert.ok(micros(row.at) < micros(exits.get(row.tid)!.at));
  }
  assert.equal(owner.unknownWrites.length, 0);
  assert.equal(owner.failedWrites.length, 0);
  assert.equal(owner.unfinishedOperations.length, 0);
  assert.equal(owner.zeroCapacityReads.length, 0);
  const eof = one(eofReads);
  assert.equal(eof.fd, 0);
  assert.ok(members.has(eof.tid));
  assert.ok(micros(eof.at) < micros(group.at));
  assert.ok(eof.completion);
  assert.equal(eof.completion.returnedAt, null);
  assert.ok(eof.completion.returnedBy);
  assert.equal(eof.completion.returnedBy.basis, "next-same-thread-event");
  assert.ok(eof.completion.returnedBy.line > eof.line);
  const upper = eof.completion.returnedAt ?? eof.completion.returnedBy?.at;
  assert.ok(upper && micros(eof.at) <= micros(upper));
  if (eof.completion.returnedBy) {
    const bound = eof.completion.returnedBy;
    assert.ok(
      io.some((row) => row.tid === eof.tid && row.line === bound.line && row.at === bound.at) ||
        owner.threadExits.some(
          (row) => row.tid === eof.tid && row.line === bound.line && row.at === bound.at,
        ) ||
        owner.exitGroups.some(
          (row) => row.tid === eof.tid && row.line === bound.line && row.at === bound.at,
        ),
      "EOF completion bound must identify an actual owned event",
    );
  }
  for (const part of watcherAck.parts) requireCompletedBefore(part, eof);
  for (const part of shutdownAck.parts) requireCompletedBefore(part, eof);
  const stderr = io
    .filter((row) => row.op === "write" && row.fd === 2)
    .toSorted((a, b) => a.at.localeCompare(b.at) || a.tid - b.tid || a.line - b.line);
  const expected = [
    `Added new watch: root files for ${proof.config} watcher 6.0\n`,
    `\t${proof.root}/**/*\n`,
    "\n",
    historicalRootWarning.toString("utf8"),
  ].map((value) => Buffer.from(value));
  assert.deepEqual(
    stderr.map((row) => Buffer.from(row.data)),
    expected,
  );
  const completed = Buffer.concat(expected);
  assert.ok(acknowledgedHeaderWarning(completed));
  const relation = (earlier: NativeTimedObservation, later: NativeTimedObservation) => {
    assert.ok(earlier.completion);
    if (earlier.tid === later.tid && earlier.line < (later.startedLine ?? later.line))
      return "PROVEN";
    const bound = earlier.completion.returnedAt ?? earlier.completion.returnedBy?.at;
    return bound && micros(bound) < micros(later.at) ? "PROVEN" : "UNPROVEN";
  };
  const writes = stderr.map((row) => {
    assert.ok(row.completion && row.data.byteLength > 0);
    assert.ok(micros(row.at) < micros(group.at));
    requireCompletedBefore(eof, row);
    requireCompletedBefore(row, terminalPoint(exits.get(row.tid)!));
    return {
      ...row,
      hex: Buffer.from(row.data).toString("hex"),
      eofReturnOrder: "PROVEN" as const,
      exitGroupReturnOrder: relation(row, terminalPoint(group)),
    };
  });
  return {
    schema: "vize.nativeStableWatcher.acknowledgedRootWarningObservation.v1",
    kind: "acknowledged-root-header-warning" as const,
    context,
    nativeQualification: false,
    candidateFunctionalPass: false,
    lifecyclePass: false,
    registrationPass: false,
    observedRootRegistrationAcknowledged: true,
    stderrOutputComplete: true,
    rawIOComplete: true,
    nativeOwner: owner.nativeOwner,
    members: owner.members,
    threadExits: owner.threadExits,
    exitGroupRequestEntry: group,
    nativeStdinEof: eof,
    stderrWrites: writes,
    completedStderr: {
      bytes: completed.length,
      sha256: createHash("sha256").update(completed).digest("hex"),
      hex: completed.toString("hex"),
    },
    watcher,
    watcherAck,
    shutdownAck,
    clientPackets: client,
    serverPackets: server,
    api,
    scope:
      "Captured immutable baseline only. ACK before EOF and EOF before stderr are proven; no ACK-before-shutdown assumption, no synthesized cancellation, no native qualification.",
  };
}

/** Existing canceled/unobserved historical paths retain their original implementation. */
export function observeRootWatchWarning(...args: Args) {
  const bytes = Buffer.concat(
    args[3]
      .filter((row) => row.op === "write" && row.fd === 2)
      .toSorted((a, b) => a.at.localeCompare(b.at) || a.tid - b.tid || a.line - b.line)
      .map((row) => Buffer.from(row.data)),
  );
  return acknowledgedHeaderWarning(bytes)
    ? observeAcknowledgedRootWarning(...args)
    : observePreviousRootWarning(...args);
}
