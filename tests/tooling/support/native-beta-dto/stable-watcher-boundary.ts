import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export const historicalSource = "13fa41356d075047a229f580014b637d3bbeb57f";
export const stableImageSha256 = "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4";
export interface WatcherContext {
  source: string;
  role: string;
  sdkVersion: string;
  sdkSha256: string;
  sdkExecutable: string;
  projectRoots: readonly string[];
}
type Packet = Record<string, unknown>;
function object(value: unknown): Packet {
  assert.ok(
    value && typeof value === "object" && !Array.isArray(value),
    "whole native packet required",
  );
  return value as Packet;
}
function closed(value: unknown, keys: readonly string[]): Packet {
  const result = object(value);
  assert.deepEqual(
    Object.keys(result).sort(),
    [...keys].sort(),
    "unexpected watcher payload fields",
  );
  return result;
}
function id(value: unknown): string {
  assert.ok(
    typeof value === "string" || (typeof value === "number" && Number.isSafeInteger(value)),
    "native request ID required",
  );
  return `${typeof value}:${value}`;
}
function one<T>(values: T[], message: string): T {
  assert.equal(values.length, 1, message);
  return values[0]!;
}

/** Complete request/reply closure, with one distinct historical shutdown observation. */
export function verifyNativeRelations(
  clientValue: readonly unknown[],
  serverValue: readonly unknown[],
  context: WatcherContext,
) {
  const client = clientValue.map(object),
    server = serverValue.map(object);
  for (const packet of [...client, ...server]) assert.equal(packet.jsonrpc, "2.0");
  const unmatched: Array<{ direction: number; request: Packet }> = [];
  for (const [direction, [requests, replies]] of [
    [client, server],
    [server, client],
  ].entries()) {
    const calls = requests.filter((row) => "id" in row && "method" in row);
    const answers = replies.filter((row) => "id" in row && !("method" in row));
    const ids = calls.map((row) => id(row.id)),
      answered = answers.map((row) => id(row.id));
    assert.equal(new Set(ids).size, ids.length, "duplicate native request ID");
    assert.equal(new Set(answered).size, answered.length, "duplicate native response ID");
    assert.deepEqual(
      answered.filter((value) => !ids.includes(value)),
      [],
      "native response without request",
    );
    for (const request of calls)
      if (!answered.includes(id(request.id))) unmatched.push({ direction, request });
  }
  if (unmatched.length === 0) return null;
  assert.equal(context.role, "before", "only the immutable before role may observe this refusal");
  assert.equal(
    context.source,
    historicalSource,
    "only the immutable before source may observe this refusal",
  );
  assert.equal(context.sdkVersion, "7.0.2", "beta and after roles require complete replies");
  assert.equal(context.sdkSha256, stableImageSha256, "exact stable native image required");
  assert.match(
    context.sdkExecutable,
    /^\/[^\n]*\/node_modules\/@typescript\/typescript-linux-x64\/lib\/tsc$/u,
  );
  const { direction, request } = one(
    unmatched,
    "exactly one historical watcher refusal is permitted",
  );
  assert.equal(direction, 1, "client requests must all receive replies");
  closed(request, ["jsonrpc", "id", "method", "params"]);
  assert.equal(request.jsonrpc, "2.0");
  assert.equal(request.method, "client/registerCapability");
  assert.equal(typeof request.id, "string");
  assert.match(request.id as string, /^ts[1-9]\d*$/u);

  const shutdown = one(
    client.filter((row) => row.method === "shutdown"),
    "one actual shutdown required",
  );
  const shutdownAck = one(
    server.filter((row) => "id" in row && !("method" in row) && id(row.id) === id(shutdown.id)),
    "whole shutdown ACK required",
  );
  assert.equal(shutdownAck.result, null);
  assert.equal("error" in shutdownAck, false);
  const requestIndex = server.indexOf(request),
    shutdownAckIndex = server.indexOf(shutdownAck);
  assert.ok(
    requestIndex > shutdownAckIndex,
    "watcher refusal must follow the shutdown ACK on the raw stream",
  );

  const projectInfo = one(
    client.filter((row) => row.method === "custom/projectInfo"),
    "one owned project identity request required",
  );
  const projectInfoAck = one(
    server.filter((row) => "id" in row && !("method" in row) && id(row.id) === id(projectInfo.id)),
    "owned project identity reply required",
  );
  const config = closed(projectInfoAck.result, ["configFilePath"]).configFilePath;
  assert.equal("error" in projectInfoAck, false);
  assert.equal(typeof config, "string");
  const project = path.posix.dirname(config as string);
  assert.ok(project.startsWith("/tmp/") && path.posix.normalize(project) === project);
  assert.ok(
    context.projectRoots.includes(project),
    "watcher project must belong to the source observer",
  );
  assert.equal(config, project + "/tsconfig.json");
  const document = object(object(projectInfo.params).textDocument).uri;
  assert.equal(typeof document, "string");
  const filename = fileURLToPath(document as string);
  assert.equal(path.posix.dirname(filename), project);
  assert.equal(pathToFileURL(filename).href, document, "exact owned document URI required");
  const initialize = one(
    client.filter((row) => row.method === "initialize"),
    "one native initialization required",
  );
  const initializeAck = one(
    server.filter((row) => "id" in row && !("method" in row) && id(row.id) === id(initialize.id)),
    "actual native initialization reply required",
  );
  assert.equal("error" in initializeAck, false);
  assert.ok(client.indexOf(initialize) < client.indexOf(projectInfo));
  assert.ok(client.indexOf(projectInfo) < client.indexOf(shutdown));
  assert.ok(server.indexOf(initializeAck) < server.indexOf(projectInfoAck));
  assert.ok(server.indexOf(projectInfoAck) < shutdownAckIndex);
  assert.deepEqual(object(object(initializeAck.result).serverInfo), {
    name: "typescript-go",
    version: "7.0.2",
  });
  const initializedRoot = object(initialize.params).rootPath;
  assert.ok(
    initializedRoot === project ||
      initializedRoot === project + "/node_modules/.vize/corsa-overlay",
    "native owner belongs to another project",
  );
  assert.equal(object(initialize.params).rootUri, pathToFileURL(initializedRoot as string).href);

  const registrations = closed(request.params, ["registrations"]).registrations;
  assert.ok(Array.isArray(registrations));
  const registration = closed(one(registrations, "one complete watcher registration required"), [
    "id",
    "method",
    "registerOptions",
  ]);
  assert.equal(registration.method, "workspace/didChangeWatchedFiles");
  const watchers = closed(registration.registerOptions, ["watchers"]).watchers;
  assert.ok(Array.isArray(watchers));
  const watcher = closed(one(watchers, "one complete native watcher required"), [
    "globPattern",
    "kind",
  ]);
  assert.equal(watcher.kind, 7);
  let boundary: "historical-root-files" | "post-shutdown-sdk-program-files";
  if (registration.id === `root files for ${config} watcher 6.0`) {
    boundary = "historical-root-files";
    assert.equal(watcher.globPattern, project + "/**/*");
  } else {
    boundary = "post-shutdown-sdk-program-files";
    assert.equal(registration.id, `program files for ${config} watcher 7.0`);
    assert.equal(watcher.globPattern, path.posix.dirname(context.sdkExecutable) + "/**/*");
    const rootRequest = one(
      server.filter((row) => {
        if (row.method !== "client/registerCapability") return false;
        const params = object(row.params),
          entries = params.registrations;
        return (
          Array.isArray(entries) &&
          entries.length === 1 &&
          object(entries[0]).id === `root files for ${config} watcher 6.0`
        );
      }),
      "SDK watcher requires the earlier actual root watcher",
    );
    closed(rootRequest, ["jsonrpc", "id", "method", "params"]);
    assert.match(rootRequest.id as string, /^ts[1-9]\d*$/u);
    assert.deepEqual(closed(rootRequest.params, ["registrations"]).registrations, [
      {
        id: `root files for ${config} watcher 6.0`,
        method: "workspace/didChangeWatchedFiles",
        registerOptions: { watchers: [{ globPattern: project + "/**/*", kind: 7 }] },
      },
    ]);
    assert.ok(server.indexOf(rootRequest) < shutdownAckIndex);
    const rootAck = one(
      client.filter(
        (row) => "id" in row && !("method" in row) && id(row.id) === id(rootRequest.id),
      ),
      "earlier root watcher must be acknowledged",
    );
    assert.equal(rootAck.result, null);
    assert.equal("error" in rootAck, false);
  }
  return {
    schema: "vize.nativeStableWatcher.postShutdownObservation.v1",
    boundary,
    context,
    nativeQualification: false,
    lifecyclePass: false,
    registrationPass: false,
    requestIndex,
    shutdownAckIndex,
    request,
    shutdown,
    shutdownAck,
    projectInfo,
    projectInfoAck,
    clientPackets: clientValue,
    serverPackets: serverValue,
  };
}

import { requireCompletedBefore, type NativeTimedObservation } from "./native-io.ts";
import { micros, timedPackets, type WatcherTimedIo } from "./stable-watcher-timing.ts";
export { micros };
export type { WatcherTimedIo };
export function requireCandidateStableStderr(bytes: Uint8Array) {
  assert.equal(bytes.byteLength, 0, "Candidate stable stderr must be empty");
}

/** One historical log split across protocol stdout and post-EOF stderr. */
export function stableWatchContext(
  client: readonly unknown[],
  server: readonly unknown[],
  context: WatcherContext,
  io: readonly WatcherTimedIo[],
  eofReads: readonly NativeTimedObservation[],
) {
  assert.equal(context.role, "before", "split watch log is a baseline observation only");
  assert.equal(context.source, historicalSource);
  assert.equal(context.sdkVersion, "7.0.2");
  assert.equal(context.sdkSha256, stableImageSha256);
  assert.match(
    context.sdkExecutable,
    /^\/[^\n]*\/node_modules\/@typescript\/typescript-linux-x64\/lib\/tsc$/u,
  );
  for (const row of io) micros(row.at);
  const input = timedPackets(io, "read", 0),
    output = timedPackets(io, "write", 1);
  assert.deepEqual(
    input.map((row) => row.packet),
    client,
    "whole owned stdin required",
  );
  assert.deepEqual(
    output.map((row) => row.packet),
    server,
    "whole owned stdout required",
  );
  assert.ok(
    input.every((row) => row.packet.method !== "exit"),
    "EOF-only native shutdown required",
  );
  assert.equal(
    verifyNativeRelations(client, server, context),
    null,
    "all requests must be answered",
  );
  const call = (method: string) =>
    one(
      input.filter((row) => row.packet.method === method),
      `one ${method} required`,
    );
  const reply = (request: Packet) =>
    one(
      output.filter(
        (row) =>
          "id" in row.packet && !("method" in row.packet) && id(row.packet.id) === id(request.id),
      ),
      "one whole native reply required",
    );
  const projectInfo = call("custom/projectInfo"),
    projectReply = reply(projectInfo.packet);
  const config = closed(projectReply.packet.result, ["configFilePath"]).configFilePath;
  assert.ok(typeof config === "string");
  const project = path.posix.dirname(config as string);
  assert.ok(/^\/tmp\/[^\r\n]+$/u.test(project) && path.posix.normalize(project) === project);
  assert.ok(context.projectRoots.includes(project), "exact source project required");
  assert.equal(config, project + "/tsconfig.json");
  assert.deepEqual(projectInfo.packet.params, {
    textDocument: { uri: pathToFileURL(project + "/main.ts").href },
  });
  const initialize = call("initialize"),
    initialized = reply(initialize.packet);
  assert.deepEqual(object(object(initialized.packet.result).serverInfo), {
    name: "typescript-go",
    version: "7.0.2",
  });
  const initializedRoot = object(initialize.packet.params).rootPath;
  assert.equal(initializedRoot, project + "/node_modules/.vize/corsa-overlay");
  assert.equal(
    object(initialize.packet.params).rootUri,
    pathToFileURL(initializedRoot as string).href,
  );
  const shutdown = call("shutdown"),
    shutdownAck = reply(shutdown.packet);
  for (const [request, answer] of [
    [initialize, initialized],
    [projectInfo, projectReply],
    [shutdown, shutdownAck],
  ])
    assert.ok(micros(request!.at) < micros(answer!.at));
  assert.ok(
    micros(initialize.at) < micros(projectInfo.at) && micros(projectInfo.at) < micros(shutdown.at),
  );
  assert.equal(shutdownAck.packet.result, null);
  closed(shutdownAck.packet, ["jsonrpc", "id", "result"]);
  for (const row of [...input, ...output])
    if ("id" in row.packet && !("method" in row.packet)) assert.equal("error" in row.packet, false);
  assert.ok(
    micros(initialized.at) < micros(projectReply.at) &&
      micros(projectReply.at) < micros(shutdownAck.at),
  );
  const eof = one([...eofReads], "one owned native stdin EOF required");
  for (const part of shutdownAck.parts) requireCompletedBefore(part, eof);
  const header = one(
    output.filter(
      (row) =>
        row.packet.method === "window/logMessage" &&
        object(row.packet.params).message ===
          `Added new watch: root files for ${config} watcher 6.0`,
    ),
    "one complete watch header required",
  );
  assert.deepEqual(header.packet, {
    jsonrpc: "2.0",
    method: "window/logMessage",
    params: {
      type: 3,
      message: `Added new watch: root files for ${config} watcher 6.0`,
    },
  });
  assert.ok(
    micros(eof.at) < micros(header.at),
    "header entry follows EOF entry, not its unobserved return",
  );
  const watcher = one(
    output.filter(
      (row) =>
        row.packet.method === "client/registerCapability" &&
        JSON.stringify(row.packet.params) ===
          JSON.stringify({
            registrations: [
              {
                id: `root files for ${config} watcher 6.0`,
                method: "workspace/didChangeWatchedFiles",
                registerOptions: { watchers: [{ globPattern: project + "/**/*", kind: 7 }] },
              },
            ],
          }),
    ),
    "one exact owned root watcher required",
  );
  closed(watcher.packet, ["jsonrpc", "id", "method", "params"]);
  assert.equal(typeof watcher.packet.id, "string");
  assert.match(watcher.packet.id as string, /^ts[1-9]\d*$/u);
  const watcherAck = one(
    input.filter(
      (row) =>
        "id" in row.packet &&
        !("method" in row.packet) &&
        id(row.packet.id) === id(watcher.packet.id),
    ),
    "one acknowledged root watcher required",
  );
  assert.equal(watcherAck.packet.result, null);
  closed(watcherAck.packet, ["jsonrpc", "id", "result"]);
  assert.ok(micros(watcher.at) < micros(watcherAck.at));
  for (const part of watcherAck.parts) requireCompletedBefore(part, eof);
  const stderr = io
    .filter((row) => row.op === "write" && row.fd === 2)
    .sort((a, b) => a.at.localeCompare(b.at) || a.tid - b.tid || a.line - b.line);
  return {
    context,
    project,
    config,
    input,
    output,
    header,
    projectInfo,
    projectReply,
    shutdown,
    shutdownAck,
    watcher,
    watcherAck,
    eof,
    stderr,
  };
}

/** Strict default: every stderr attempt must follow observed EOF completion. */
export function stableWatchActor(...args: Parameters<typeof stableWatchContext>) {
  const actor = stableWatchContext(...args);
  for (const row of actor.stderr) requireCompletedBefore(actor.eof, row);
  return actor;
}

export function observeStableSplitWatchLog(
  client: readonly unknown[],
  server: readonly unknown[],
  context: WatcherContext,
  io: readonly WatcherTimedIo[],
  eofReads: readonly NativeTimedObservation[],
) {
  const actor = stableWatchActor(client, server, context, io, eofReads);
  const expected =
    `\t${actor.project}/**/*\n\n1 paths ineligible for watching\n` +
    "errors updating watches: [failed to register file watcher: context canceled failed to register file watcher: context canceled]\n";
  assert.deepEqual(
    Buffer.concat(actor.stderr.map((row) => Buffer.from(row.data))),
    Buffer.from(expected),
  );
  return {
    schema: "vize.nativeStableWatcher.splitWatchLogObservation.v1",
    context,
    nativeQualification: false,
    candidateFunctionalPass: false,
    lifecyclePass: false,
    registrationPass: true,
    header: actor.header,
    projectInfo: actor.projectInfo,
    projectReply: actor.projectReply,
    shutdown: actor.shutdown,
    shutdownAck: actor.shutdownAck,
    watcher: actor.watcher,
    watcherAck: actor.watcherAck,
    nativeStdinEof: actor.eof,
    stderrWrites: actor.stderr.map((row) => ({
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
