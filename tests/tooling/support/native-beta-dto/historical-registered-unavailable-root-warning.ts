import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { isDeepStrictEqual } from "node:util";
import {
  historicalWarningTranscript,
  type HistoricalWarningArgs as Args,
} from "./historical-inflight-transcript.ts";
import { historicalRootWarning } from "./historical-inflight-root-warning.ts";
import { requireCompletedBefore, requireObservedEntryBefore } from "./native-io.ts";
import { micros, timedPackets } from "./stable-watcher-timing.ts";
import { requireExitTeardownUnfinished } from "./exit-teardown.ts";

const one = <T>(rows: readonly T[]): T => {
  assert.equal(rows.length, 1);
  return rows[0]!;
};

/**
 * A bounded historical refusal where the RPC log acknowledges the root watcher,
 * but strace never observes the warning write's result.
 */
export function observeRegisteredUnavailableRootWarning(...args: Args) {
  const [client, server, context, io, eofReads, owner, api] = args;
  const proof = historicalWarningTranscript(client, server, context, api, "required");
  assert.ok(proof.watchLog);
  assert.equal(api.precedingNonProtocolBytes, 5);
  assert.ok(server.indexOf(proof.configuration) < server.indexOf(proof.watcher));
  assert.ok(server.indexOf(proof.watcher) < server.indexOf(proof.shutdownAck));
  assert.ok(server.indexOf(proof.shutdownAck) < server.indexOf(proof.watchLog));

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
  const locate = (rows: typeof input, packet: unknown) =>
    one(rows.filter((row) => isDeepStrictEqual(row.packet, packet)));
  const watcherAck = locate(input, proof.watcherAck),
    shutdownAck = locate(output, proof.shutdownAck),
    watchLog = locate(output, proof.watchLog);

  assert.ok(Number.isSafeInteger(owner.nativeOwner) && owner.nativeOwner > 0);
  const members = new Map(owner.members.map((row) => [row.tid, row.parent]));
  assert.equal(members.size, owner.members.length);
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
  assert.ok(typeof group.line === "number" && Number.isSafeInteger(group.line));
  for (const exit of exits.values()) {
    assert.equal(exit.code, 0);
    assert.ok(Number.isSafeInteger(exit.line));
    assert.ok(micros(group.at) < micros(exit.at));
  }
  for (const row of io) {
    assert.ok(members.has(row.tid));
    assert.ok(micros(row.at) < micros(exits.get(row.tid)!.at));
  }
  assert.equal(owner.failedWrites.length, 0);
  requireExitTeardownUnfinished(owner);
  assert.equal(owner.zeroCapacityReads.length, 0);

  const eof = one(eofReads);
  assert.equal(eof.fd, 0);
  assert.ok(members.has(eof.tid));
  assert.ok(eof.completion && eof.completion.returnedAt === null);
  assert.ok(eof.completion.returnedBy);
  for (const part of watcherAck.parts) requireCompletedBefore(part, eof);
  for (const part of shutdownAck.parts) requireCompletedBefore(part, eof);
  assert.ok(micros(eof.at) < micros(watchLog.at));

  const stderr = one(io.filter((row) => row.op === "write" && row.fd === 2));
  assert.deepEqual(Buffer.from(stderr.data), Buffer.from("\n"));
  assert.ok(stderr.completion);
  assert.ok(micros(eof.at) < micros(stderr.at));
  const attempt = one(owner.unknownWrites);
  assert.ok(members.has(attempt.tid));
  assert.equal(attempt.file, `syscalls.${attempt.tid}`);
  assert.equal(attempt.op, "write");
  assert.equal(attempt.syscall, "write");
  assert.equal(attempt.fd, 2);
  assert.equal(attempt.argumentsComplete, true);
  assert.equal(attempt.iovCount, null);
  assert.equal(attempt.iovecs, null);
  assert.equal(attempt.outcome, "unknown");
  assert.equal(attempt.rawResult, "?");
  assert.equal(attempt.returnedBytes, null);
  assert.equal(attempt.completion, undefined);
  assert.deepEqual(attempt.data, historicalRootWarning);
  assert.equal(attempt.requestedBytes, historicalRootWarning.length);
  assert.equal(stderr.tid, attempt.tid);
  requireCompletedBefore(stderr, attempt);
  requireObservedEntryBefore(attempt, { ...group, line: group.line });
  assert.ok(micros(group.at) < micros(exits.get(attempt.tid)!.at));

  const completed = Buffer.from(stderr.data);
  return {
    schema: "vize.nativeStableWatcher.registeredUnavailableRootWarning.v1",
    kind: "registered-unavailable-root-warning" as const,
    context,
    nativeQualification: false,
    candidateFunctionalPass: false,
    lifecyclePass: false,
    registrationPass: false,
    observedRootRegistrationAcknowledged: true,
    stderrOutputComplete: false,
    rawIOComplete: false,
    nativeOwner: owner.nativeOwner,
    members: owner.members,
    threadExits: owner.threadExits,
    exitGroupRequestEntry: group,
    nativeStdinEof: eof,
    completedStderr: {
      bytes: completed.length,
      sha256: createHash("sha256").update(completed).digest("hex"),
      hex: completed.toString("hex"),
    },
    completedStderrWrites: [stderr],
    unknownAttempts: [
      {
        sourceRow: attempt,
        returnedBytes: null,
        deliveredBytes: null,
        actualUnknownReturnTime: null,
      },
    ],
    watcherAck,
    shutdownAck,
    rootWatchLog: watchLog,
    clientPackets: client,
    serverPackets: server,
    api,
    scope:
      "Immutable baseline refusal only: the RPC watch log is complete, but the 32-byte stderr write result and delivery remain unknown. Merged order proves only completed newline before warning entry before exit-request entry.",
  };
}
