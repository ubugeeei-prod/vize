//! Full original client/server frames for the closed native-capacity source session.
import assert from "node:assert/strict";
import type { CapacityPacket, CapacityProbe } from "../tooling/support/lsp/diagnostic-capacity.ts";
import { decodeFrames } from "./lsp-wire.ts";
import {
  NATIVE_CAPACITY_INITIALIZATION,
  NATIVE_CAPACITY_PLAN,
} from "./lsp-native-capacity-contract.ts";

export type HistoricalReport = {
  producer: { schema: string; version: number; sourceRevision: string; binarySha256: string };
  input: unknown;
  packets: { atMs: number; packet: CapacityPacket }[];
  events: { stage: string; value?: unknown }[];
  stderr: string;
  qualified: boolean;
  cleanup: { complete: boolean; errors: unknown[] };
  reverseDependentCounts?: string[];
};
const open = (probe: CapacityProbe, version: number, text: string) => ({
  textDocument: { uri: probe.uri, languageId: "vue", version, text },
});
const change = (probe: CapacityProbe, version: number, text: string) => ({
  textDocument: { uri: probe.uri, version },
  contentChanges: [{ text }],
});

export function validateCapacityWire(
  report: HistoricalReport,
  client: Buffer,
  server: Buffer,
  reopen: boolean,
): void {
  const messages = decodeFrames(client).messages;
  const published = decodeFrames(server)
    .messages.filter((message) => message.method === "textDocument/publishDiagnostics")
    .map((message) => message.params);
  assert.deepEqual(
    published,
    report.packets.map(({ packet }) => packet),
    "every original publication, including duplicates and all fields, must survive",
  );
  const requests = messages.filter((message) => message.method && message.id !== undefined);
  assert.deepEqual(
    requests.map((message) => message.method),
    ["initialize", "textDocument/documentColor", "shutdown"],
  );
  assert.equal(messages.filter((message) => !message.method && message.id !== undefined).length, 0);
  assert.deepEqual(requests[0]?.params?.initializationOptions, NATIVE_CAPACITY_INITIALIZATION);
  const notifications = (method: string) =>
    messages.filter((message) => message.method === method).map((message) => message.params);
  assert.equal(notifications("initialized").length, 1);
  assert.equal(notifications("exit").length, 1);
  if (!reopen) {
    const probes = report.input as CapacityProbe[];
    const seed = probes[0];
    assert.ok(seed);
    assert.equal(messages.length, NATIVE_CAPACITY_PLAN.client.messages);
    const opens = Array.from({ length: 129 }, () => "textDocument/didOpen");
    const edits = Array.from({ length: 129 }, () => "textDocument/didChange");
    assert.deepEqual(
      messages.map((message) => message.method),
      [
        "initialize",
        "initialized",
        "textDocument/didOpen",
        "textDocument/didChange",
        ...opens,
        "textDocument/documentColor",
        ...edits,
        ...edits,
        "textDocument/didChange",
        "shutdown",
        "exit",
      ],
    );
    assert.deepEqual(
      notifications("textDocument/didOpen"),
      probes.map((probe) => open(probe, 1, probe.invalid)),
    );
    assert.deepEqual(notifications("textDocument/didChange"), [
      change(seed, 2, seed.invalid + "\n"),
      ...probes.slice(1).map((probe) => change(probe, 2, probe.valid)),
      ...probes.slice(1).map((probe) => change(probe, 3, probe.valid)),
      change(seed, 3, seed.valid),
    ]);
    assert.equal(notifications("textDocument/didClose").length, 0);
  } else {
    const { warm, old, next } = report.input as Record<"warm" | "old" | "next", CapacityProbe>;
    assert.equal(messages.length, 12);
    assert.deepEqual(
      messages.map((message) => message.method),
      [
        "initialize",
        "initialized",
        "textDocument/didOpen",
        "textDocument/didOpen",
        "textDocument/didChange",
        "textDocument/didClose",
        "textDocument/didOpen",
        "textDocument/documentColor",
        "textDocument/didChange",
        "textDocument/didChange",
        "shutdown",
        "exit",
      ],
    );
    assert.deepEqual(notifications("textDocument/didOpen"), [
      open(warm, 1, warm.invalid),
      open(old, 0, old.valid),
      open(next, 1, next.valid),
    ]);
    assert.deepEqual(notifications("textDocument/didClose"), [{ textDocument: { uri: old.uri } }]);
    assert.deepEqual(notifications("textDocument/didChange"), [
      change(old, 1, old.invalid),
      change(next, 2, next.invalid),
      change(next, 3, next.valid),
    ]);
  }
}
