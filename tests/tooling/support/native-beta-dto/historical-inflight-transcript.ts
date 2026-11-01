import assert from "node:assert/strict";
import path from "node:path";
import { pathToFileURL } from "node:url";
import type { observeHistoricalRootWatchWarning } from "./historical-root-watch-warning.ts";
import {
  historicalSource,
  stableImageSha256,
  verifyNativeRelations,
} from "./stable-watcher-boundary.ts";

export type HistoricalWarningArgs = Parameters<typeof observeHistoricalRootWatchWarning>;
type Args = HistoricalWarningArgs;
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
export function historicalWarningTranscript(
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
    configuration,
  };
}
