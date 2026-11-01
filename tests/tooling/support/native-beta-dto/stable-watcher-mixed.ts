import assert from "node:assert/strict";
import {
  stableWatchContext,
  historicalSplitWatchContext,
  observeStableSplitWatchLog,
} from "./stable-watcher-boundary.ts";
import { micros } from "./stable-watcher-timing.ts";
import { requireCompletedBefore, type nativeOwnerTerminalState } from "./native-io.ts";

type ContextArgs = Parameters<typeof stableWatchContext>;
type OwnerState = ReturnType<typeof nativeOwnerTerminalState>;
function proven(earlier: ContextArgs[4][number], later: ContextArgs[3][number]) {
  assert.ok(earlier.completion, "observed completed EOF required");
  if (earlier.tid === later.tid && earlier.line < (later.startedLine ?? later.line)) return true;
  const bound = earlier.completion.returnedAt ?? earlier.completion.returnedBy?.at;
  return bound !== undefined && bound !== null && micros(bound) < micros(later.at);
}

/** A fixed historical observation never converts an unknown temporal order into PASS. */
export function observeHistoricalSplitWatchLog(...args: [...ContextArgs, OwnerState]) {
  const [client, server, context, io, eofReads, owner] = args;
  const actor = historicalSplitWatchContext(client, server, context, io, eofReads, owner);
  const observed = actor.eof.completion;
  assert.ok(observed);
  const knownBound = observed.returnedAt ?? observed.returnedBy?.at;
  if (knownBound) assert.ok(micros(actor.eof.at) <= micros(knownBound));
  if (!actor.rootAckFragmentOrder && actor.stderr.every((row) => proven(actor.eof, row)))
    return observeStableSplitWatchLog(client, server, context, io, eofReads);
  assert.equal(owner.unknownWrites.length, 0, "unobserved writes refuse mixed EOF observation");
  assert.equal(
    owner.failedWrites.length,
    0,
    "failed writes are distinct from this complete observation",
  );
  const members = new Map(owner.members.map((row) => [row.tid, row.parent]));
  assert.equal(members.size, owner.members.length);
  assert.equal(members.get(owner.nativeOwner), null, "actual native owner required");
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
  assert.deepEqual(
    [...members.keys()].sort((a, b) => a - b),
    owner.threadExits.map((row) => row.tid).sort((a, b) => a - b),
  );
  assert.ok(owner.threadExits.every((row) => row.code === 0));
  assert.ok(
    members.has(actor.eof.tid) && io.every((row) => members.has(row.tid)),
    "whole IO belongs to this native owner",
  );
  assert.ok("fd" in actor.eof);
  assert.equal(actor.eof.fd, 0);
  const completion = actor.eof.completion!;
  assert.equal(completion.returnedAt, null, "an observed return cannot become unproven");
  const upper = completion.returnedBy;
  assert.ok(upper && upper.basis === "next-same-thread-event" && upper.line > actor.eof.line);
  assert.ok(micros(actor.eof.at) < micros(upper.at), "nonempty observed return bracket required");
  const event = io.filter(
    (row) =>
      row.tid === actor.eof.tid &&
      (row.startedLine ?? row.line) === upper.line &&
      row.at === upper.at,
  );
  let exitUpperBound: OwnerState["exitGroups"][number] | undefined;
  let threadUpperBound: OwnerState["threadExits"][number] | undefined;
  if (actor.rootAckFragmentOrder) {
    assert.equal(event.length, 0, "this EOF upper bound is the actual terminal thread exit");
    assert.deepEqual(
      actor.rootAckFragmentOrder.eofReturnUpperBoundThreadExit,
      owner.threadExits.find((row) => row.tid === actor.eof.tid),
    );
  } else if (event.length === 0) {
    const groups = owner.exitGroups.filter(
      (row) =>
        row.tid === actor.eof.tid &&
        row.at === upper.at &&
        row.line === upper.line &&
        row.code === 0 &&
        row.rawResult === "?",
    );
    if (groups.length !== 0) {
      assert.equal(groups.length, 1, "one exact terminal exit-group upper bound required");
      exitUpperBound = groups[0]!;
    } else {
      assert.ok(
        actor.watcherAck.parts.every((part) => part.tid === actor.eof.tid),
        "terminal thread bound requires root ACK in the EOF reader",
      );
      const exits = owner.threadExits.filter(
        (row) =>
          row.tid === actor.eof.tid &&
          row.at === upper.at &&
          row.line === upper.line &&
          row.code === 0,
      );
      assert.equal(
        exits.length,
        1,
        "EOF upper bound requires exact same-thread terminal provenance",
      );
      threadUpperBound = exits[0]!;
    }
  } else assert.equal(event.length, 1, "EOF upper bound requires its actual same-thread IO event");
  const expected = Buffer.from(
    `\t${actor.project}/**/*\n\n1 paths ineligible for watching\n` +
      "errors updating watches: [failed to register file watcher: context canceled failed to register file watcher: context canceled]\n",
  );
  const delivered = Buffer.concat(actor.stderr.map((row) => Buffer.from(row.data)));
  assert.equal(delivered.length, 182);
  assert.deepEqual(delivered, expected);
  const relations = actor.stderr.map((row) => {
    assert.ok(row.data.byteLength > 0 && row.completion, "only positive completed stderr writes");
    assert.ok(micros(actor.eof.at) < micros(row.at), "stderr entry must follow EOF entry");
    for (const part of actor.shutdownAck.parts) requireCompletedBefore(part, row);
    const status = proven(actor.eof, row) ? "PROVEN" : "UNPROVEN";
    if (status === "UNPROVEN") {
      assert.notEqual(row.tid, actor.eof.tid, "same-thread order cannot become unknown");
      assert.ok(
        micros(row.at) <= micros(upper.at),
        "unproven write must overlap the actual return bracket",
      );
    }
    return {
      at: row.at,
      tid: row.tid,
      line: row.line,
      completion: row.completion,
      hex: Buffer.from(row.data).toString("hex"),
      eofReturnOrder: status,
    };
  });
  assert.ok(relations.some((row) => row.eofReturnOrder === "UNPROVEN"));
  if (actor.rootAckFragmentOrder) {
    assert.ok(
      relations.every((row) => row.eofReturnOrder === "UNPROVEN"),
      "fragmented root ACK observation retains every unproven stderr relation",
    );
    for (const row of actor.stderr)
      assert.equal(
        owner.threadExits.filter(
          (exit) =>
            exit.tid === row.tid &&
            exit.code === 0 &&
            exit.line !== undefined &&
            exit.line > row.line &&
            micros(row.at) < micros(exit.at),
        ).length,
        1,
        "each completed stderr write precedes its actual writer thread exit",
      );
  }
  return {
    schema: "vize.nativeStableWatcher.unprovenEofReturnObservation.v1",
    context,
    nativeQualification: false,
    candidateFunctionalPass: false,
    lifecyclePass: false,
    registrationPass: false,
    observedRootRegistrationAcknowledged: true,
    stderrOutputComplete: true,
    nativeOwner: owner.nativeOwner,
    members: owner.members,
    threadExits: owner.threadExits,
    nativeWriteAttempts: { unknown: owner.unknownWrites, failed: owner.failedWrites },
    observedShutdownAck: actor.shutdownAck,
    header: actor.header,
    projectInfo: actor.projectInfo,
    projectReply: actor.projectReply,
    watcher: actor.watcher,
    watcherAck: actor.watcherAck,
    nativeStdinEof: actor.eof,
    ...(exitUpperBound ? { eofReturnUpperBoundExitRequest: exitUpperBound } : {}),
    ...(threadUpperBound ? { eofReturnUpperBoundThreadExit: threadUpperBound } : {}),
    ...(actor.rootAckFragmentOrder ? { rootAckFragmentOrder: actor.rootAckFragmentOrder } : {}),
    stderrWrites: relations,
    clientPackets: client,
    serverPackets: server,
  };
}
