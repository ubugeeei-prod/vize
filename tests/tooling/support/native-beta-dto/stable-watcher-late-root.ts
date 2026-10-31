import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import {
  historicalSource,
  stableImageSha256,
  verifyNativeRelations,
  type WatcherContext,
} from "./stable-watcher-boundary.ts";
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

/** One immutable baseline: ACKed late root, followed by a distinct unanswered SDK watcher. */
export function observeLateRootProgramWatcher(
  clientValue: readonly unknown[],
  serverValue: readonly unknown[],
  context: WatcherContext,
) {
  const client = clientValue.map(object),
    server = serverValue.map(object);
  for (const packet of [...client, ...server]) assert.equal(packet.jsonrpc, "2.0");
  assert.deepEqual(
    client.filter((row) => "method" in row).map((row) => row.method),
    ["initialize", "initialized", "custom/initializeAPISession", "custom/projectInfo", "shutdown"],
  );
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
  assert.equal(client.length, 7, "whole seven-packet historical client required");
  assert.equal(server.length, 27, "whole 27-packet historical server required");
  for (const packet of [...client, ...server]) {
    assert.equal("error" in packet, false, "error response refuses historical observation");
    if ("id" in packet && !("method" in packet)) assert.ok("result" in packet);
  }
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
  const configuration = one(
    server.filter(
      (row) => "id" in row && "method" in row && row !== request && row !== rootRequest,
    ),
    "one configuration registration required",
  );
  closed(configuration, ["jsonrpc", "id", "method", "params"]);
  assert.equal(configuration.method, "client/registerCapability");
  assert.deepEqual(configuration.params, {
    registrations: [
      {
        id: "typescript-config-watch-id",
        method: "workspace/didChangeConfiguration",
        registerOptions: { section: ["js/ts", "typescript", "javascript", "editor"] },
      },
    ],
  });
  const rootRequestIndex = server.indexOf(rootRequest);
  assert.ok(shutdownAckIndex < rootRequestIndex, "raw root watcher must follow shutdown ACK");
  assert.ok(rootRequestIndex < requestIndex, "raw root watcher must precede SDK watcher");
  const rootAck = one(
    client.filter((row) => "id" in row && !("method" in row) && id(row.id) === id(rootRequest.id)),
    "earlier root watcher must be acknowledged",
  );
  closed(rootAck, ["jsonrpc", "id", "result"]);
  assert.equal(rootAck.result, null);
  assert.equal("error" in rootAck, false);
  assert.ok(
    client.indexOf(shutdown) < client.indexOf(rootAck),
    "root ACK must follow shutdown send",
  );
  const header = one(
    server.filter(
      (row) =>
        row.method === "window/logMessage" &&
        object(row.params).message === `Added new watch: root files for ${config} watcher 6.0`,
    ),
    "exact owned late root watch header required",
  );
  assert.deepEqual(closed(header.params, ["type", "message"]), {
    type: 3,
    message: `Added new watch: root files for ${config} watcher 6.0`,
  });
  assert.ok(rootRequestIndex < server.indexOf(header) && server.indexOf(header) < requestIndex);

  return {
    schema: "vize.nativeStableWatcher.lateRootProgramObservation.v1",
    boundary: "post-shutdown-root-and-sdk-program-files" as const,
    rootRequestIndex,
    rootAckIndex: client.indexOf(rootAck),
    rootRequest,
    rootAck,
    observedRootRegistrationAcknowledged: true,
    dtoFunctionalPass: false,
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

/** Select only by actual stream order; every other shape retains the original strict verifier. */
export function verifyHistoricalWatcherRelations(
  client: readonly unknown[],
  server: readonly unknown[],
  context: WatcherContext,
) {
  const rows = server.map(object),
    shutdown = client.map(object).find((row) => row.method === "shutdown");
  const shutdownIndex = shutdown
    ? rows.findIndex((row) => "id" in row && !("method" in row) && id(row.id) === id(shutdown.id))
    : -1;
  const lateRoot =
    shutdownIndex >= 0 &&
    rows.some((row, index) => {
      if (index <= shutdownIndex || row.method !== "client/registerCapability") return false;
      const registrations = object(row.params).registrations;
      return (
        Array.isArray(registrations) &&
        registrations.some(
          (entry) =>
            typeof object(entry).id === "string" &&
            (object(entry).id as string).startsWith("root files for "),
        )
      );
    });
  const programUnanswered = rows.some((row) => {
    if (row.method !== "client/registerCapability" || !("id" in row)) return false;
    const entries = object(row.params).registrations;
    return (
      Array.isArray(entries) &&
      entries.some(
        (entry) =>
          typeof object(entry).id === "string" &&
          (object(entry).id as string).startsWith("program files for "),
      ) &&
      !client
        .map(object)
        .some((reply) => "id" in reply && !("method" in reply) && id(reply.id) === id(row.id))
    );
  });
  return lateRoot && programUnanswered
    ? observeLateRootProgramWatcher(client, server, context)
    : verifyNativeRelations(client, server, context);
}
