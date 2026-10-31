import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  stableWatchActor,
  micros,
  type WatcherContext,
  type WatcherTimedIo,
} from "./stable-watcher-boundary.ts";
import {
  requireCompletedBefore,
  type NativeTimedObservation,
  type nativeOwnerTerminalState,
} from "./native-io.ts";

const one = <T>(rows: readonly T[], message: string): T => {
  assert.equal(rows.length, 1, message);
  return rows[0]!;
};
/** An observed terminal write attempt is never promoted to delivered bytes. */
export function observeTerminalStableStderr(
  client: readonly unknown[],
  server: readonly unknown[],
  context: WatcherContext,
  io: readonly WatcherTimedIo[],
  eofReads: readonly (NativeTimedObservation & { fd: number })[],
  terminal: ReturnType<typeof nativeOwnerTerminalState>,
) {
  const actor = stableWatchActor(client, server, context, io, eofReads);
  const members = new Map(terminal.members.map((row) => [row.tid, row.parent]));
  assert.equal(members.size, terminal.members.length, "unique native threads required");
  assert.equal(members.get(terminal.nativeOwner), null, "actual native leader required");
  for (const [tid, parent] of members) {
    if (tid === terminal.nativeOwner) continue;
    assert.notEqual(parent, null);
    const visited = new Set<number>();
    let current = tid;
    while (current !== terminal.nativeOwner) {
      assert.ok(!visited.has(current), "cyclic native thread group");
      visited.add(current);
      const next = members.get(current);
      assert.ok(typeof next === "number");
      current = next;
    }
  }
  const exits = new Map(terminal.threadExits.map((row) => [row.tid, row]));
  assert.equal(exits.size, terminal.threadExits.length);
  assert.deepEqual(
    [...exits.keys()].sort((a, b) => a - b),
    [...members.keys()].sort((a, b) => a - b),
    "whole native thread closure required",
  );
  const group = one(terminal.exitGroups, "one actual native exit_group start required");
  assert.ok(members.has(group.tid));
  assert.equal(group.code, 0);
  for (const row of exits.values()) {
    assert.equal(row.code, 0);
    assert.ok(micros(group.at) < micros(row.at));
  }
  for (const row of io) {
    const exit = exits.get(row.tid);
    assert.ok(exit, "IO belongs to another native owner");
    assert.ok(micros(row.at) < micros(exit.at), "IO entry must precede its thread exit");
  }
  const eof = one(eofReads, "one actual owned EOF required");
  assert.equal(eof.fd, 0);
  assert.ok(members.has(eof.tid));
  assert.ok(micros(eof.at) < micros(group.at));
  const attempt = one(terminal.unknownWrites, "exactly one owned unobserved write required");
  assert.equal(attempt.file, `syscalls.${attempt.tid}`);
  assert.equal(attempt.op, "write");
  assert.equal(attempt.fd, 2);
  assert.equal(attempt.argumentsComplete, true);
  if (attempt.syscall === "write") {
    assert.equal(attempt.iovCount, null);
    assert.equal(attempt.iovecs, null);
  } else {
    assert.equal(attempt.syscall, "writev");
    assert.ok(attempt.iovecs);
    assert.equal(attempt.iovCount, attempt.iovecs.length);
    assert.ok(attempt.iovecs.every((row) => row.data.length === row.requestedBytes));
    assert.deepEqual(Buffer.concat(attempt.iovecs.map((row) => row.data)), attempt.data);
    assert.equal(
      attempt.iovecs.reduce((sum, row) => sum + row.requestedBytes, 0),
      attempt.requestedBytes,
    );
  }
  assert.equal(attempt.outcome, "unknown");
  assert.equal(attempt.returnedBytes, null);
  assert.equal(attempt.rawResult, "?");
  const expectedAttempt = Buffer.from(
    "errors updating watches: [failed to register file watcher: context canceled failed to register file watcher: context canceled]\n",
  );
  assert.deepEqual(attempt.data, expectedAttempt);
  assert.equal(attempt.requestedBytes, expectedAttempt.length);
  const writerExit = exits.get(attempt.tid);
  assert.ok(writerExit, "attempt belongs to another native owner");
  requireCompletedBefore(eof, attempt);
  assert.ok(micros(group.at) < micros(attempt.at) && micros(attempt.at) < micros(writerExit.at));
  const completed = Buffer.concat(actor.stderr.map((row) => Buffer.from(row.data)));
  assert.deepEqual(completed, Buffer.from("\n1 paths ineligible for watching\n"));
  assert.ok(
    terminal.unfinishedOperations.every(
      (row) =>
        members.has(row.tid) &&
        micros(group.at) < micros(row.at) &&
        micros(row.at) < micros(exits.get(row.tid)!.at),
    ),
    "unresolved terminal operations must retain owned lifetime bounds",
  );
  for (const failed of terminal.failedWrites) {
    assert.equal(failed.outcome, "failed");
    assert.ok(failed.returnedBytes < 0);
    assert.equal(failed.op, "write");
    assert.equal(failed.file, `syscalls.${failed.tid}`);
    const exit = exits.get(failed.tid);
    assert.ok(exit);
    assert.ok(micros(failed.at) < micros(exit.at));
  }
  const { data: attemptedData, ...attemptMetadata } = attempt;
  return {
    schema: "vize.nativeStableWatcher.unobservedTerminalStderr.v1",
    context,
    nativeQualification: false,
    candidateFunctionalPass: false,
    lifecyclePass: false,
    registrationPass: true,
    stderrOutputComplete: false,
    rawIOComplete: false,
    completedStderr: {
      bytes: completed.length,
      sha256: createHash("sha256").update(completed).digest("hex"),
      hex: completed.toString("hex"),
    },
    attemptedStderr: {
      ...attemptMetadata,
      attemptedHex: attemptedData.toString("hex"),
      returnedBytes: null,
      deliveredBytes: null,
    },
    exitGroupStart: group,
    members: terminal.members,
    threadExits: terminal.threadExits,
    unfinishedOperations: terminal.unfinishedOperations,
    failedWrites: terminal.failedWrites,
    header: actor.header,
    watcher: actor.watcher,
    watcherAck: actor.watcherAck,
    projectInfo: actor.projectInfo,
    projectReply: actor.projectReply,
    shutdown: actor.shutdown,
    shutdownAck: actor.shutdownAck,
    nativeStdinEof: eof,
    completedStderrWrites: actor.stderr.map((row) => ({
      at: row.at,
      completion: row.completion,
      tid: row.tid,
      line: row.line,
      hex: Buffer.from(row.data).toString("hex"),
    })),
    clientPackets: client,
    serverPackets: server,
  };
}
