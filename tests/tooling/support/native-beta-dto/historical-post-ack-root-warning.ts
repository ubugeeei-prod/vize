import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  historicalWarningTranscript,
  type HistoricalWarningArgs as Args,
} from "./historical-inflight-transcript.ts";
import {
  historicalRootWarning,
  observeRootWatchWarning as observeStrictRootWatchWarning,
} from "./historical-inflight-root-warning.ts";
import { micros, timedPackets } from "./stable-watcher-timing.ts";
export { historicalRootWarning } from "./historical-inflight-root-warning.ts";

const one = <T>(rows: readonly T[]) => {
  assert.equal(rows.length, 1);
  return rows[0]!;
};

/** Whole framed channels only: no syscall completion, EOF, delivery or cancellation conclusion. */
export function historicalPostAckTranscript(
  client: Args[0],
  server: Args[1],
  context: Args[2],
  api: Args[6],
) {
  const proof = historicalWarningTranscript(client, server, context, api);
  assert.ok(server.indexOf(proof.configuration) < server.indexOf(proof.shutdownAck));
  assert.ok(server.indexOf(proof.shutdownAck) < server.indexOf(proof.watcher));
  assert.equal(api.precedingNonProtocolBytes, 5);
  return proof;
}

/** Source-provided actor/IO rows restrict a historical negative, never native qualification. */
export function observeHistoricalPostAckRootWarning(...args: Args) {
  const [client, server, context, io, eofReads, owner, api] = args;
  const proof = historicalPostAckTranscript(client, server, context, api);
  assert.deepEqual(
    timedPackets(io, "read", 0).map((row) => row.packet),
    client,
  );
  assert.deepEqual(
    timedPackets(io, "write", 1).map((row) => row.packet),
    server,
  );
  assert.ok(Number.isSafeInteger(owner.nativeOwner) && owner.nativeOwner > 0);
  const members = new Map(owner.members.map((row) => [row.tid, row.parent]));
  assert.equal(members.size, owner.members.length);
  assert.equal(members.get(owner.nativeOwner), null);
  for (const tid of members.keys()) {
    assert.ok(Number.isSafeInteger(tid) && tid > 0);
    let current = tid;
    const seen = new Set<number>();
    while (current !== owner.nativeOwner) {
      assert.ok(!seen.has(current));
      seen.add(current);
      const parent = members.get(current);
      assert.ok(typeof parent === "number" && members.has(parent));
      current = parent;
    }
  }
  assert.equal(owner.failedWrites.length, 0);
  assert.equal(owner.unfinishedOperations.length, 0);
  const attempt = one(owner.unknownWrites);
  assert.ok(members.has(attempt.tid));
  assert.equal(attempt.file, `syscalls.${attempt.tid}`);
  assert.ok(Number.isSafeInteger(attempt.line) && attempt.line >= 0);
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
  assert.equal(attempt.requestedBytes, 32);
  assert.deepEqual(attempt.data, historicalRootWarning);
  const stderr = io
    .filter((row) => row.op === "write" && row.fd === 2)
    .toSorted((a, b) => a.at.localeCompare(b.at) || a.tid - b.tid || a.line - b.line);
  assert.ok(stderr.length > 0);
  const header = Buffer.from(
    `Added new watch: root files for ${proof.config} watcher 6.0\n\t${proof.root}/**/*\n\n`,
  );
  assert.equal(header.length, 97);
  assert.deepEqual(Buffer.concat(stderr.map((row) => Buffer.from(row.data))), header);
  for (const row of io) {
    assert.ok(members.has(row.tid));
    assert.ok(Number.isSafeInteger(row.line) && row.line >= 0);
  }
  for (const row of stderr) {
    assert.equal(row.tid, attempt.tid);
    assert.ok(
      row.completion,
      "completed payload rows must retain their original completion metadata",
    );
  }
  return {
    schema: "vize.nativeStableWatcher.postAckRootWarningNegative.v1",
    kind: "post-ack-root-warning-negative" as const,
    context,
    nativeQualification: false,
    candidateFunctionalPass: false,
    lifecyclePass: false,
    registrationPass: false,
    observedRootRegistrationAcknowledged: true,
    stderrOutputComplete: false,
    rawIOComplete: false,
    nativeOwner: owner.nativeOwner,
    nativeOwnerSourceRows: owner,
    clientPackets: client,
    serverPackets: server,
    api,
    channelFacts: {
      configuration: { index: server.indexOf(proof.configuration), packet: proof.configuration },
      shutdownAck: { index: server.indexOf(proof.shutdownAck), packet: proof.shutdownAck },
      rootWatch: { index: server.indexOf(proof.watcher), packet: proof.watcher },
      clientRootAck: { index: client.indexOf(proof.watcherAck), packet: proof.watcherAck },
    },
    completedStderr: {
      bytes: header.length,
      sha256: createHash("sha256").update(header).digest("hex"),
      hex: header.toString("hex"),
    },
    stderrWrites: stderr,
    unknownAttempts: [
      {
        sourceRow: attempt,
        returnedBytes: null,
        deliveredBytes: null,
        actualUnknownReturnTime: null,
        eofReturnOrder: "UNPROVEN" as const,
      },
    ],
    sourceIoRows: io,
    sourceEofRowsUninterpreted: eofReads,
    scope:
      "CHANNEL-ONLY HISTORICAL NEGATIVE: whole paired framed channels and source-provided actor rows; no independently replayed whole thread graph, EOF return, completion ordering, cancellation or delivery claim.",
  };
}

/** Only this audit entry adds the distinct post-ACK negative; original exports stay strict. */
export function observeRootWatchWarning(...args: Args) {
  const owner = args[5];
  if (
    owner.unknownWrites.length === 1 &&
    owner.exitGroups.length === 1 &&
    micros(owner.unknownWrites[0]!.at) < micros(owner.exitGroups[0]!.at)
  ) {
    const proof = historicalWarningTranscript(args[0], args[1], args[2], args[6]);
    if (args[1].indexOf(proof.shutdownAck) < args[1].indexOf(proof.watcher))
      return observeHistoricalPostAckRootWarning(...args);
  }
  return observeStrictRootWatchWarning(...args);
}
