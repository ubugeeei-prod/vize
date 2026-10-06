import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { decodeFrames } from "../../../differential/lsp-wire.ts";
import { sha256 } from "../../../performance/support/warm-type-backed-source.ts";
import { finishWire, object } from "../../../performance/support/warm-type-backed-packets.ts";
import type { VerifiedPublishedLspLaunch } from "./published-launch.ts";
import { root } from "./paths.ts";
import type { run } from "./responsive-lint-observations.ts";

export async function publicWire(
  captureRoot: string,
  side: Awaited<ReturnType<typeof run>>,
  installation: VerifiedPublishedLspLaunch,
  typecheck: boolean,
) {
  const wire = await finishWire(root, [], captureRoot);
  const observation = wire.observation;
  assert.equal(observation.launchAuthority, "published-release");
  assert.equal(observation.captureByteLimit, 16 * 1024 * 1024);
  assert.equal(Object.hasOwn(observation, "buildReceipt"), false);
  assert.deepEqual(observation.publicationReceipt, installation.receipt);
  assert.deepEqual(observation.binary, installation.expected);
  const streams = object(observation.streams);
  const bytes = Object.fromEntries(
    ["client", "server", "stderr"].map((kind) => {
      const stream = object(streams[kind]);
      assert.equal(stream.path, `${kind}.bin`);
      const data = fs.readFileSync(path.join(wire.capture, `${kind}.bin`));
      assert.equal(sha256(data), stream.sha256);
      assert.equal(data.length, stream.capturedBytes);
      return [kind, data];
    }),
  );
  const client = decodeFrames(bytes.client).messages;
  const server = decodeFrames(bytes.server).messages;
  assert.equal(client.length, typecheck ? 12 : 11);
  assert.equal(server.length, typecheck ? 15 : 10);
  assert.deepEqual(
    client.map((message) => message.method),
    [
      "initialize",
      "initialized",
      "textDocument/didOpen",
      ...Array.from({ length: typecheck ? 7 : 6 }, () => "textDocument/didChange"),
      "shutdown",
      "exit",
    ],
  );
  assert.deepEqual(client[1], { jsonrpc: "2.0", method: "initialized", params: {} });
  assert.deepEqual(client[2], {
    jsonrpc: "2.0",
    method: "textDocument/didOpen",
    params: {
      textDocument: {
        uri: side.uri,
        languageId: "vue",
        version: 1,
        text: side.rows[1].source,
      },
    },
  });
  assert.deepEqual(
    client.slice(3, -2),
    side.rows.map((row) => ({
      jsonrpc: "2.0",
      method: "textDocument/didChange",
      params: {
        textDocument: { uri: side.uri, version: row.version },
        contentChanges: [{ text: row.source }],
      },
    })),
  );
  const responses = server.filter((message) => Object.hasOwn(message, "id"));
  assert.deepEqual(responses, [
    { jsonrpc: "2.0", id: client[0].id, result: side.initialization },
    { jsonrpc: "2.0", id: client.at(-2)!.id, result: null },
  ]);
  assert.deepEqual(client.at(-2), { jsonrpc: "2.0", id: 2, method: "shutdown" });
  assert.deepEqual(client.at(-1), { jsonrpc: "2.0", method: "exit" });
  assert.deepEqual(object(client[0].params).initializationOptions, {
    lint: true,
    typecheck,
    editor: true,
  });
  const info = object(object(side.initialization).serverInfo);
  assert.equal(info.name, "vize-maestro");
  assert.equal(info.version, installation.receipt.authority.releaseVersion);
  const publications = server.filter(
    (message) => message.method === "textDocument/publishDiagnostics",
  );
  assert.deepEqual(
    publications,
    side.notifications.map(({ params }) => ({
      jsonrpc: "2.0",
      method: "textDocument/publishDiagnostics",
      params,
    })),
  );
  assert.equal(publications.length, typecheck ? 12 : 7);
  assert.deepEqual(
    publications.map((message) => message.params?.version),
    typecheck ? [1, 2, 3, 3, 4, 5, 5, 6, 7, 7, 8, 8] : [1, 2, 3, 4, 5, 6, 7],
  );
  const other = server.filter(
    (message) =>
      !Object.hasOwn(message, "id") && message.method !== "textDocument/publishDiagnostics",
  );
  assert.equal(other.length, 1);
  assert.equal(other[0].method, "window/logMessage");
  assert.equal(bytes.stderr.subarray(0, Buffer.byteLength(side.stderr)).toString(), side.stderr);
  const stderr = bytes.stderr.toString("utf8");
  return {
    ...wire,
    client,
    server,
    stderr,
    diagnosticTimeouts: [...stderr.matchAll(/corsa diagnostics timed out/gu)].length,
    snapshotReleaseWarnings: [...stderr.matchAll(/failed to release corsa snapshot/gu)].length,
    limits: "raw warnings retained; no native lifetime, OS reaping, general speedup or CPU claim",
  };
}
