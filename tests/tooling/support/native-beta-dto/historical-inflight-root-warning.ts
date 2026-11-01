import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { isDeepStrictEqual } from "node:util";
import {
  historicalRootWarning,
  observeHistoricalRootWatchWarning,
} from "./historical-root-watch-warning.ts";
import { historicalWarningTranscript } from "./historical-inflight-transcript.ts";
import { micros, timedPackets } from "./stable-watcher-timing.ts";
import { requireCompletedBefore, type NativeTimedObservation } from "./native-io.ts";
export { historicalRootWarning } from "./historical-root-watch-warning.ts";

type Args = Parameters<typeof observeHistoricalRootWatchWarning>;
type Packet = Record<string, unknown>;
const one = <T>(rows: readonly T[]): T => {
  assert.equal(rows.length, 1);
  return rows[0]!;
};
/** The original exported contract always keeps its strict pre-shutdown root-watch order. */
export function historicalInflightTranscript(
  client: Args[0],
  server: Args[1],
  context: Args[2],
  api: Args[6],
) {
  const { configuration, ...proof } = historicalWarningTranscript(client, server, context, api);
  assert.ok(
    server.indexOf(configuration) < server.indexOf(proof.watcher) &&
      server.indexOf(proof.watcher) < server.indexOf(proof.shutdownAck),
  );
  return proof;
}

/** Causal subset proof grants no whole-trace or primary-observer replay claim. */
export function historicalInflightLifecycle(
  proof: ReturnType<typeof historicalInflightTranscript>,
  io: Args[3],
  eofReads: Args[4],
  owner: Args[5],
) {
  const input = timedPackets(io, "read", 0),
    output = timedPackets(io, "write", 1);
  const subsequence = (parts: typeof input, whole: readonly unknown[]) => {
    let cursor = 0;
    for (const part of parts) {
      while (cursor < whole.length && !isDeepStrictEqual(part.packet, whole[cursor])) cursor++;
      assert.ok(cursor < whole.length, "timed packets must be unchanged whole-packet subsequences");
      cursor++;
    }
  };
  subsequence(input, proof.clientValue);
  subsequence(output, proof.serverValue);
  const packet = (rows: typeof input, expected: Packet) =>
    one(rows.filter((row) => isDeepStrictEqual(row.packet, expected)));
  const watcherAck = packet(input, proof.watcherAck),
    shutdownAck = packet(output, proof.shutdownAck);
  // The causal calls/replies must themselves all have actual syscall provenance.
  for (const expected of [proof.initialize, proof.projectInfo, proof.shutdown])
    packet(input, expected);
  for (const expected of [proof.initialized, proof.projectReply, proof.watcher])
    packet(output, expected);
  const members = new Map(owner.members.map((row) => [row.tid, row.parent]));
  assert.equal(members.size, owner.members.length);
  assert.equal(members.get(owner.nativeOwner), null);
  for (const tid of members.keys()) {
    const seen = new Set<number>();
    let current = tid;
    while (current !== owner.nativeOwner) {
      assert.ok(!seen.has(current));
      seen.add(current);
      const parent = members.get(current);
      assert.equal(typeof parent, "number");
      current = parent!;
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
  }
  for (const row of io) {
    assert.ok(members.has(row.tid));
    assert.ok(micros(row.at) < micros(exits.get(row.tid)!.at));
  }
  assert.equal(owner.failedWrites.length, 0);
  assert.equal(owner.unfinishedOperations.length, 0);
  const eof = one(eofReads);
  assert.equal(eof.fd, 0);
  assert.ok(members.has(eof.tid));
  assert.equal(eof.completion?.returnedAt, null);
  const upper = eof.completion?.returnedBy,
    eofExit = exits.get(eof.tid)!;
  assert.ok(upper && upper.basis === "next-same-thread-event");
  assert.equal(upper.at, eofExit.at);
  assert.equal(upper.line, eofExit.line);
  assert.ok(upper.line > eof.line && micros(eof.at) < micros(upper.at));
  for (const part of watcherAck.parts) requireCompletedBefore(part, eof);
  const stderr = io
    .filter((row) => row.op === "write" && row.fd === 2)
    .toSorted((a, b) => a.at.localeCompare(b.at) || a.tid - b.tid || a.line - b.line);
  assert.ok(stderr.length > 0);
  const completed = Buffer.concat(stderr.map((row) => Buffer.from(row.data)));
  assert.deepEqual(
    completed,
    Buffer.from(
      `Added new watch: root files for ${proof.config} watcher 6.0\n\t${proof.root}/**/*\n\n`,
    ),
  );
  const attempt = one(owner.unknownWrites);
  assert.equal(attempt.file, `syscalls.${attempt.tid}`);
  assert.ok(members.has(attempt.tid));
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
  assert.ok(
    micros(attempt.at) < micros(group.at) && micros(group.at) < micros(exits.get(attempt.tid)!.at),
  );
  assert.ok(
    stderr.every(
      (row) => row.tid === attempt.tid && row.line < (attempt.startedLine ?? attempt.line),
    ),
  );
  for (const row of stderr) {
    assert.ok(micros(row.at) < micros(attempt.at));
    requireCompletedBefore(row, attempt);
  }
  function timing(row: NativeTimedObservation) {
    assert.notEqual(row.tid, eof.tid);
    assert.ok(micros(eof.at) < micros(row.at) && micros(row.at) < micros(upper!.at));
    assert.ok(micros(row.at) < micros(exits.get(row.tid)!.at));
    for (const part of shutdownAck.parts) requireCompletedBefore(part, row);
    return {
      at: row.at,
      tid: row.tid,
      line: row.line,
      startedLine: row.startedLine,
      completion: row.completion,
      eofReturnOrder: "UNPROVEN" as const,
    };
  }
  const stderrWrites = stderr.map((row) => {
    assert.ok(row.completion && row.data.byteLength > 0);
    return { ...timing(row), hex: Buffer.from(row.data).toString("hex") };
  });
  const unknownAttempts = [
    {
      ...timing(attempt),
      syscall: attempt.syscall,
      fd: attempt.fd,
      file: attempt.file,
      rawArguments: attempt.rawArguments,
      rawResult: attempt.rawResult,
      requestedBytes: attempt.requestedBytes,
      attemptedHex: attempt.data.toString("hex"),
      returnedBytes: null,
      deliveredBytes: null,
      actualUnknownReturnTime: null,
    },
  ];
  return {
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
    stderrWrites,
    unknownAttempts,
    watcherAck,
    shutdownAck,
    unfinishedOperations: owner.unfinishedOperations,
    failedWrites: owner.failedWrites,
  };
}

export function observeHistoricalInflightRootWarning(...args: Args) {
  const [client, server, context, io, eofReads, owner, api] = args;
  assert.deepEqual(
    timedPackets(io, "read", 0).map((row) => row.packet),
    client,
  );
  assert.deepEqual(
    timedPackets(io, "write", 1).map((row) => row.packet),
    server,
  );
  const proof = historicalInflightTranscript(client, server, context, api);
  const lifecycle = historicalInflightLifecycle(proof, io, eofReads, owner);
  return {
    schema: "vize.nativeStableWatcher.inflightRootWarningObservation.v1",
    kind: "inflight-unobserved-warning-attempt" as const,
    context,
    nativeQualification: false,
    candidateFunctionalPass: false,
    lifecyclePass: false,
    registrationPass: false,
    observedRootRegistrationAcknowledged: true,
    stderrOutputComplete: false,
    rawIOComplete: false,
    ...lifecycle,
    clientPackets: client,
    serverPackets: server,
    api,
    observedPairs: {
      lsp: {
        client: { requests: 4, responses: 4, unanswered: [] },
        server: { requests: 2, responses: 2, unanswered: [] },
      },
      api: {
        client: { requests: 2, responses: 2, unanswered: [] },
        server: { requests: 0, responses: 0, unanswered: [] },
      },
    },
    scope:
      "Immutable baseline only: the unknown write entered before the exit request and remained unobserved at writer exit. EOF return and delivery are unproven; no cancellation is inferred.",
  };
}

/** All old paths retain the original observer and its stricter post-exit-request law. */
export function observeRootWatchWarning(...args: Args) {
  const owner = args[5];
  if (
    owner.unknownWrites.length === 1 &&
    owner.exitGroups.length === 1 &&
    micros(owner.unknownWrites[0]!.at) < micros(owner.exitGroups[0]!.at)
  )
    return observeHistoricalInflightRootWarning(...args);
  return observeHistoricalRootWatchWarning(...args);
}
