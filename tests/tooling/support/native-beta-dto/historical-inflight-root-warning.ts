import assert from "node:assert/strict";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { createHash } from "node:crypto";
import { isDeepStrictEqual } from "node:util";
import {
  historicalRootWarning,
  observeHistoricalRootWatchWarning,
} from "./historical-root-watch-warning.ts";
import {
  historicalSource,
  stableImageSha256,
  verifyNativeRelations,
} from "./stable-watcher-boundary.ts";
import { micros, timedPackets } from "./stable-watcher-timing.ts";
import { requireCompletedBefore, type NativeTimedObservation } from "./native-io.ts";
export { historicalRootWarning } from "./historical-root-watch-warning.ts";

type Args = Parameters<typeof observeHistoricalRootWatchWarning>;
type Packet = Record<string, unknown>;
const object = (value: unknown): Packet => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Packet;
};
const one = <T>(rows: readonly T[]): T => {
  assert.equal(rows.length, 1);
  return rows[0]!;
};
const closed = (value: unknown, keys: string[]) => {
  const row = object(value);
  assert.deepEqual(Object.keys(row).sort(), keys.sort());
  return row;
};
const id = (value: unknown) => {
  assert.ok(
    typeof value === "string" || (typeof value === "number" && Number.isSafeInteger(value)),
  );
  return `${typeof value}:${value}`;
};

/** Whole saved RPC facts can be checked independently of a bounded timed trace subset. */
export function historicalInflightTranscript(
  clientValue: Args[0],
  serverValue: Args[1],
  context: Args[2],
  api: Args[6],
) {
  assert.equal(context.role, "before");
  assert.equal(context.source, historicalSource);
  assert.equal(context.sdkVersion, "7.0.2");
  assert.equal(context.sdkSha256, stableImageSha256);
  assert.match(
    context.sdkExecutable,
    /^\/[^\n]*\/node_modules\/@typescript\/typescript-linux-x64\/lib\/tsc$/u,
  );
  const client = clientValue.map(object),
    server = serverValue.map(object);
  assert.equal(verifyNativeRelations(clientValue, serverValue, context), null);
  assert.equal(verifyNativeRelations(api.client, api.server, context), null);
  for (const row of [...client, ...server, ...api.client.map(object), ...api.server.map(object)]) {
    assert.equal(row.jsonrpc, "2.0");
    if ("id" in row && !("method" in row)) closed(row, ["jsonrpc", "id", "result"]);
  }
  const calls = client.filter((row) => "method" in row && "id" in row);
  assert.deepEqual(
    calls.map((row) => row.method),
    ["initialize", "custom/initializeAPISession", "custom/projectInfo", "shutdown"],
  );
  assert.ok(client.every((row) => row.method !== "exit"));
  const call = (method: string) => one(calls.filter((row) => row.method === method));
  const reply = (request: Packet) =>
    one(server.filter((row) => "id" in row && !("method" in row) && id(row.id) === id(request.id)));
  const projectInfo = call("custom/projectInfo"),
    projectReply = reply(projectInfo);
  const config = closed(projectReply.result, ["configFilePath"]).configFilePath;
  assert.equal(typeof config, "string");
  const root = path.posix.dirname(config as string);
  assert.match(root, /^\/tmp\/[^\r\n]+$/u);
  assert.equal(path.posix.normalize(root), root);
  assert.ok(context.projectRoots.includes(root));
  assert.equal(config, root + "/tsconfig.json");
  assert.deepEqual(projectInfo.params, {
    textDocument: { uri: pathToFileURL(root + "/main.ts").href },
  });
  const initialize = call("initialize"),
    initialized = reply(initialize);
  assert.deepEqual(object(initialized.result).serverInfo, {
    name: "typescript-go",
    version: "7.0.2",
  });
  const selectedRoot = root + "/node_modules/.vize/corsa-overlay";
  assert.equal(object(initialize.params).rootPath, selectedRoot);
  assert.equal(object(initialize.params).rootUri, pathToFileURL(selectedRoot).href);
  const shutdown = call("shutdown"),
    shutdownAck = reply(shutdown);
  assert.deepEqual(shutdownAck, { jsonrpc: "2.0", id: shutdown.id, result: null });
  assert.ok(
    server.indexOf(initialized) < server.indexOf(projectReply) &&
      server.indexOf(projectReply) < server.indexOf(shutdownAck),
  );
  assert.deepEqual(
    api.client.map((row) => object(row).method),
    ["initialize", "parseConfigFile"],
  );
  assert.equal(object(api.client[0]).params, null);
  assert.deepEqual(object(api.client[1]).params, { file: selectedRoot + "/tsconfig.json" });
  assert.equal(api.server.length, 2);
  assert.ok(
    Number.isSafeInteger(api.precedingNonProtocolBytes) && api.precedingNonProtocolBytes >= 0,
  );
  const registrations = server.filter((row) => "id" in row && "method" in row);
  assert.equal(registrations.length, 2);
  const watcher = one(
    registrations.filter(
      (row) =>
        object((object(row.params).registrations as unknown[])[0]).id ===
        `root files for ${config} watcher 6.0`,
    ),
  );
  const configuration = one(registrations.filter((row) => row !== watcher));
  for (const registration of registrations) {
    closed(registration, ["jsonrpc", "id", "method", "params"]);
    assert.equal(registration.method, "client/registerCapability");
    assert.equal(typeof registration.id, "string");
    assert.match(registration.id as string, /^ts[1-9]\d*$/u);
  }
  assert.deepEqual(configuration.params, {
    registrations: [
      {
        id: "typescript-config-watch-id",
        method: "workspace/didChangeConfiguration",
        registerOptions: { section: ["js/ts", "typescript", "javascript", "editor"] },
      },
    ],
  });
  assert.deepEqual(watcher.params, {
    registrations: [
      {
        id: `root files for ${config} watcher 6.0`,
        method: "workspace/didChangeWatchedFiles",
        registerOptions: { watchers: [{ globPattern: root + "/**/*", kind: 7 }] },
      },
    ],
  });
  assert.ok(
    server.indexOf(configuration) < server.indexOf(watcher) &&
      server.indexOf(watcher) < server.indexOf(shutdownAck),
  );
  const watcherAck = one(
    client.filter((row) => "id" in row && !("method" in row) && id(row.id) === id(watcher.id)),
  );
  assert.deepEqual(watcherAck, { jsonrpc: "2.0", id: watcher.id, result: null });
  assert.equal(
    server.filter(
      (row) =>
        row.method === "window/logMessage" &&
        object(row.params).message === `Added new watch: root files for ${config} watcher 6.0`,
    ).length,
    0,
  );
  return {
    clientValue,
    serverValue,
    context,
    api,
    config,
    root,
    initialize,
    initialized,
    projectInfo,
    projectReply,
    watcher,
    watcherAck,
    shutdown,
    shutdownAck,
  };
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
