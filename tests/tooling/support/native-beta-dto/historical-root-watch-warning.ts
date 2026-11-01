import assert from "node:assert/strict";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { createHash } from "node:crypto";
import {
  historicalSource,
  stableImageSha256,
  verifyNativeRelations,
} from "./stable-watcher-boundary.ts";
import { micros, timedPackets } from "./stable-watcher-timing.ts";
import { requireCompletedBefore, type NativeTimedObservation } from "./native-io.ts";
import type { observeTerminalStableStderr } from "./terminal-stderr.ts";

type Packet = Record<string, unknown>;
type Args = Parameters<typeof observeTerminalStableStderr>;
export interface HistoricalProbeApi {
  client: readonly unknown[];
  server: readonly unknown[];
  precedingNonProtocolBytes: number;
}
const object = (value: unknown): Packet => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Packet;
};
const closed = (value: unknown, keys: string[]) => {
  const row = object(value);
  assert.deepEqual(Object.keys(row).sort(), keys.sort());
  return row;
};
const one = <T>(rows: readonly T[]): T => {
  assert.equal(rows.length, 1);
  return rows[0]!;
};
const id = (value: unknown) => {
  assert.ok(
    typeof value === "string" || (typeof value === "number" && Number.isSafeInteger(value)),
  );
  return `${typeof value}:${value}`;
};
export const historicalRootWarning = Buffer.from("1 paths ineligible for watching\n");

/** Complete observed pairs and unanswered records are facts, not qualification. */
function pairFacts(client: readonly unknown[], server: readonly unknown[]) {
  const facts = (requests: readonly unknown[], replies: readonly unknown[]) => {
    const calls = requests.map(object).filter((row) => "id" in row && "method" in row);
    const answers = replies.map(object).filter((row) => "id" in row && !("method" in row));
    return {
      requests: calls.length,
      responses: answers.length,
      unanswered: calls.filter((row) => !answers.some((answer) => id(answer.id) === id(row.id))),
    };
  };
  return { client: facts(client, server), server: facts(server, client) };
}

/** Two exact immutable-baseline transcripts; neither grants any candidate credit. */
export function observeHistoricalRootWatchWarning(...args: [...Args, HistoricalProbeApi]) {
  const [clientValue, serverValue, context, io, eofReads, owner, api] = args;
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
  const input = timedPackets(io, "read", 0),
    output = timedPackets(io, "write", 1);
  assert.deepEqual(
    input.map((row) => row.packet),
    clientValue,
  );
  assert.deepEqual(
    output.map((row) => row.packet),
    serverValue,
  );
  assert.deepEqual(
    client.filter((row) => "method" in row && "id" in row).map((row) => row.method),
    ["initialize", "custom/initializeAPISession", "custom/projectInfo", "shutdown"],
  );
  assert.ok(client.every((row) => row.method !== "exit"));
  const call = (method: string) => one(input.filter((row) => row.packet.method === method));
  const reply = (request: Packet) =>
    one(
      output.filter(
        (row) =>
          "id" in row.packet && !("method" in row.packet) && id(row.packet.id) === id(request.id),
      ),
    );
  const projectInfo = call("custom/projectInfo"),
    projectReply = reply(projectInfo.packet);
  const config = closed(projectReply.packet.result, ["configFilePath"]).configFilePath;
  assert.equal(typeof config, "string");
  const root = path.posix.dirname(config as string);
  assert.match(root, /^\/tmp\/[^\r\n]+$/u);
  assert.equal(path.posix.normalize(root), root);
  assert.ok(context.projectRoots.includes(root));
  assert.equal(config, root + "/tsconfig.json");
  assert.deepEqual(projectInfo.packet.params, {
    textDocument: { uri: pathToFileURL(root + "/main.ts").href },
  });
  const initialize = call("initialize"),
    initialized = reply(initialize.packet);
  assert.deepEqual(object(initialized.packet.result).serverInfo, {
    name: "typescript-go",
    version: "7.0.2",
  });
  const selectedRoot = root + "/node_modules/.vize/corsa-overlay";
  assert.equal(object(initialize.packet.params).rootPath, selectedRoot);
  assert.equal(object(initialize.packet.params).rootUri, pathToFileURL(selectedRoot).href);
  const shutdown = call("shutdown"),
    shutdownAck = reply(shutdown.packet);
  assert.deepEqual(shutdownAck.packet, { jsonrpc: "2.0", id: shutdown.packet.id, result: null });
  assert.ok(
    input.indexOf(initialize) < input.indexOf(projectInfo) &&
      input.indexOf(projectInfo) < input.indexOf(shutdown),
  );
  assert.ok(
    output.indexOf(initialized) < output.indexOf(projectReply) &&
      output.indexOf(projectReply) < output.indexOf(shutdownAck),
  );
  for (const row of [...client, ...server, ...api.client.map(object), ...api.server.map(object)]) {
    assert.equal(row.jsonrpc, "2.0");
    if ("id" in row && !("method" in row)) closed(row, ["jsonrpc", "id", "result"]);
  }
  const relation = verifyNativeRelations(clientValue, serverValue, context);
  assert.equal(verifyNativeRelations(api.client, api.server, context), null);
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
  const watcher = one(
    output.filter(
      (row) =>
        row.packet.method === "client/registerCapability" &&
        Array.isArray(object(row.packet.params).registrations) &&
        object((object(row.packet.params).registrations as unknown[])[0]).id ===
          `root files for ${config} watcher 6.0`,
    ),
  );
  closed(watcher.packet, ["jsonrpc", "id", "method", "params"]);
  assert.equal(typeof watcher.packet.id, "string");
  assert.match(watcher.packet.id as string, /^ts[1-9]\d*$/u);
  assert.deepEqual(watcher.packet.params, {
    registrations: [
      {
        id: `root files for ${config} watcher 6.0`,
        method: "workspace/didChangeWatchedFiles",
        registerOptions: { watchers: [{ globPattern: root + "/**/*", kind: 7 }] },
      },
    ],
  });
  const watcherAck = one(
    input.filter(
      (row) =>
        "id" in row.packet &&
        !("method" in row.packet) &&
        id(row.packet.id) === id(watcher.packet.id),
    ),
  );
  assert.deepEqual(watcherAck.packet, { jsonrpc: "2.0", id: watcher.packet.id, result: null });
  assert.ok(output.indexOf(watcher) < output.indexOf(shutdownAck));
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
  for (const exit of exits.values()) {
    assert.equal(exit.code, 0);
    assert.ok(micros(group.at) < micros(exit.at));
  }
  for (const row of io) {
    assert.ok(members.has(row.tid));
    assert.ok(micros(row.at) < micros(exits.get(row.tid)!.at));
  }
  assert.equal(owner.failedWrites.length, 0);
  const eof = one(eofReads);
  assert.equal(eof.fd, 0);
  assert.ok(members.has(eof.tid));
  assert.ok(eof.completion);
  for (const part of watcherAck.parts) requireCompletedBefore(part, eof);
  const stderr = io
    .filter((row) => row.op === "write" && row.fd === 2)
    .toSorted((a, b) => a.at.localeCompare(b.at) || a.tid - b.tid || a.line - b.line);
  const completed = Buffer.concat(stderr.map((row) => Buffer.from(row.data)));
  const header = `Added new watch: root files for ${config} watcher 6.0\n\t${root}/**/*\n\n`;
  const unknown = owner.unknownWrites;
  let kind: "completed-warning-unanswered-sdk" | "unobserved-warning-attempt";
  let stdoutWatchLog: typeof output = [];
  if (unknown.length === 0) {
    kind = "completed-warning-unanswered-sdk";
    assert.deepEqual(completed, historicalRootWarning);
    assert.equal(owner.unfinishedOperations.length, 0);
    assert.ok(relation);
    assert.equal(relation.boundary, "post-shutdown-sdk-program-files");
    const headerIndex = output.findIndex(
      (row) =>
        row.packet.method === "window/logMessage" &&
        object(row.packet.params).message ===
          `Added new watch: root files for ${config} watcher 6.0`,
    );
    assert.ok(headerIndex > output.indexOf(shutdownAck));
    stdoutWatchLog = output.slice(headerIndex, headerIndex + 3);
    assert.deepEqual(
      stdoutWatchLog.map((row) => row.packet),
      [`Added new watch: root files for ${config} watcher 6.0`, `\t${root}/**/*`, ""].map(
        (message) => ({
          jsonrpc: "2.0",
          method: "window/logMessage",
          params: { type: 3, message },
        }),
      ),
    );
    const sdkIndex = output.findIndex(
      (row) =>
        row.packet.method === "client/registerCapability" &&
        id(row.packet.id) === id(relation.request.id),
    );
    assert.deepEqual(output[sdkIndex]?.packet, relation.request);
    assert.ok(headerIndex + 2 < sdkIndex);
  } else {
    kind = "unobserved-warning-attempt";
    assert.equal(relation, null);
    assert.deepEqual(completed, Buffer.from(header));
    assert.equal(
      output.filter(
        (row) =>
          row.packet.method === "window/logMessage" &&
          object(row.packet.params).message ===
            `Added new watch: root files for ${config} watcher 6.0`,
      ).length,
      0,
    );
    const attempt = one(unknown);
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
      micros(group.at) < micros(attempt.at) &&
        micros(attempt.at) < micros(exits.get(attempt.tid)!.at),
    );
    assert.ok(owner.unfinishedOperations.length <= 1);
    for (const row of owner.unfinishedOperations) {
      assert.equal(row.tid, owner.nativeOwner);
      assert.equal(row.file, `syscalls.${row.tid}`);
      assert.equal(row.raw, `${row.at} ???( `);
      assert.ok(
        micros(group.at) < micros(row.at) && micros(row.at) < micros(exits.get(row.tid)!.at),
      );
    }
  }
  function timing(row: NativeTimedObservation) {
    assert.ok(members.has(row.tid));
    assert.ok(micros(eof.at) < micros(row.at) && micros(row.at) < micros(exits.get(row.tid)!.at));
    for (const part of shutdownAck.parts) requireCompletedBefore(part, row);
    const completion = eof.completion!;
    let proven = eof.tid === row.tid && eof.line < (row.startedLine ?? row.line);
    const bound = completion.returnedAt ?? completion.returnedBy?.at;
    if (bound && micros(bound) < micros(row.at)) proven = true;
    if (!proven) {
      assert.equal(completion.returnedAt, null);
      const upper = completion.returnedBy;
      assert.ok(upper && upper.basis === "next-same-thread-event" && upper.line > eof.line);
      assert.notEqual(row.tid, eof.tid);
      assert.ok(micros(eof.at) < micros(upper.at) && micros(row.at) <= micros(upper.at));
      assert.ok(
        io.some(
          (event) =>
            event.tid === eof.tid &&
            (event.startedLine ?? event.line) === upper.line &&
            event.at === upper.at,
        ) ||
          owner.exitGroups.some((event) => event.tid === eof.tid && event.at === upper.at) ||
          owner.threadExits.some((event) => event.tid === eof.tid && event.at === upper.at),
        "actual same-thread IO or terminal event must bound EOF return",
      );
    }
    return {
      at: row.at,
      tid: row.tid,
      line: row.line,
      startedLine: row.startedLine,
      completion: row.completion,
      eofReturnOrder: proven ? "PROVEN" : "UNPROVEN",
    };
  }
  const stderrWrites = stderr.map((row) => {
    assert.ok(row.completion && row.data.byteLength > 0);
    return { ...timing(row), hex: Buffer.from(row.data).toString("hex") };
  });
  const attempts = unknown.map((row) => ({
    ...timing(row),
    syscall: row.syscall,
    fd: row.fd,
    file: row.file,
    rawArguments: row.rawArguments,
    rawResult: row.rawResult,
    requestedBytes: row.requestedBytes,
    attemptedHex: row.data.toString("hex"),
    returnedBytes: null,
    deliveredBytes: null,
    actualUnknownReturnTime: null,
  }));
  return {
    schema: "vize.nativeStableWatcher.rootWarningObservation.v1",
    kind,
    context,
    nativeQualification: false,
    candidateFunctionalPass: false,
    lifecyclePass: false,
    registrationPass: false,
    observedRootRegistrationAcknowledged: true,
    stderrOutputComplete: unknown.length === 0,
    rawIOComplete: unknown.length === 0,
    observedPairs: {
      lsp: pairFacts(clientValue, serverValue),
      api: pairFacts(api.client, api.server),
    },
    nativeOwner: owner.nativeOwner,
    members: owner.members,
    threadExits: owner.threadExits,
    exitGroupRequestEntry: group,
    unfinishedOperations: owner.unfinishedOperations,
    failedWrites: owner.failedWrites,
    nativeStdinEof: eof,
    completedStderr: {
      bytes: completed.length,
      sha256: createHash("sha256").update(completed).digest("hex"),
      hex: completed.toString("hex"),
    },
    stderrWrites,
    unknownAttempts: attempts,
    stdoutWatchLog,
    projectInfo,
    projectReply,
    initialize,
    initialized,
    watcher,
    watcherAck,
    shutdown,
    shutdownAck,
    unansweredRegistration: relation?.request ?? null,
    clientPackets: clientValue,
    serverPackets: serverValue,
    api,
    scope:
      "Immutable baseline observation only; entries and return brackets do not establish an unobserved EOF return or unknown delivery. No cancellation is synthesized.",
  };
}
