import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { readPinnedArtifact } from "../differential/harness.mjs";
import { decodeFrames, frameMessage } from "../differential/lsp-wire.ts";
import {
  assertWholeNativeHovers,
  loadHoverCorpus,
  validateHoverCorpus,
  type HoverCapture,
} from "./support/lsp/native-attribute-hover-contract.ts";
import { root } from "./support/lsp/paths.ts";

const fixtureRoot = path.join(
  root,
  "tests/_fixtures/differential/lsp/native-attribute-hover-range-original",
);
const { corpus } = loadHoverCorpus(fixtureRoot);

// These frames test the comparator only. They are never a native/runtime observation.
function comparatorControls(): HoverCapture[] {
  return corpus.sessions.map((session) => {
    const workspaceUri = `file:///comparator-only/${session.id}/Field.vue`;
    const rootUri = `file:///comparator-only/${session.id}`;
    const responses = session.requests.map((request, index) => ({
      role: request.role,
      error: null,
      response: {
        jsonrpc: "2.0",
        id: index + 2,
        result: {
          contents: {
            kind: "markdown",
            value: `${request.signature}\n\nComplete documentation Ω🧭.`,
          },
          range: structuredClone(request.range),
        },
      },
    }));
    const client = [
      {
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: {
          processId: 1,
          rootUri,
          capabilities: corpus.initialization.capabilities,
          initializationOptions: corpus.initialization.options,
        },
      },
      { jsonrpc: "2.0", method: "initialized", params: {} },
      {
        jsonrpc: "2.0",
        method: "textDocument/didOpen",
        params: {
          textDocument: {
            uri: workspaceUri,
            languageId: "vue",
            version: 1,
            text: readPinnedArtifact(fixtureRoot, session.file).toString("utf8"),
          },
        },
      },
      ...session.requests.map((request, index) => ({
        jsonrpc: "2.0",
        id: index + 2,
        method: "textDocument/hover",
        params: { textDocument: { uri: workspaceUri }, position: request.position },
      })),
      { jsonrpc: "2.0", id: 5, method: "shutdown" },
      { jsonrpc: "2.0", method: "exit" },
    ];
    const server = [
      { jsonrpc: "2.0", id: 1, result: { capabilities: { hoverProvider: true } } },
      {
        jsonrpc: "2.0",
        method: "textDocument/publishDiagnostics",
        params: { uri: workspaceUri, version: 1, diagnostics: [] },
      },
      ...responses.map(({ response }) => response),
      { jsonrpc: "2.0", id: 5, result: null },
    ];
    return {
      id: session.id,
      workspaceUri,
      rootUri,
      failure: null,
      responses,
      wire: {
        clientWireBase64: Buffer.concat(client.map(frameMessage)).toString("base64"),
        serverWireBase64: Buffer.concat(server.map(frameMessage)).toString("base64"),
        stderrBase64: "",
        exitStatus: 0,
        signal: null,
        processError: null,
      },
    };
  });
}

await test("closed original hover corpus preserves inputs and physical UTF16 vectors", () => {
  validateHoverCorpus(corpus, fixtureRoot);
  assertWholeNativeHovers(corpus, comparatorControls());
  const dropped = structuredClone(corpus);
  dropped.sessions.pop();
  assert.throws(() => validateHoverCorpus(dropped, fixtureRoot));
  const virtual = structuredClone(corpus);
  virtual.sessions[0].requests[0].range.end.character = 24;
  assert.throws(() => validateHoverCorpus(virtual, fixtureRoot));
  const changedInput = structuredClone(corpus);
  changedInput.sessions[0].file.sha256 = "0".repeat(64);
  assert.throws(() => validateHoverCorpus(changedInput, fixtureRoot));
});

await test("whole native comparator rejects dropped, null, extra, truncated and differing answers", () => {
  const mutations = [
    (rows: HoverCapture[]) => {
      rows.pop();
    },
    (rows: HoverCapture[]) => {
      rows[1].responses.pop();
    },
    (rows: HoverCapture[]) => {
      rows[1].responses[0].response!.result = null;
    },
    (rows: HoverCapture[]) => {
      rows[1].responses[0].response!.error = null;
    },
    (rows: HoverCapture[]) => {
      const result = rows[1].responses[0].response!.result as Record<string, unknown>;
      result.extra = true;
    },
    (rows: HoverCapture[]) => {
      const result = rows[1].responses[0].response!.result as Record<string, unknown>;
      delete result.range;
    },
    (rows: HoverCapture[]) => {
      const result = rows[1].responses[0].response!.result as Record<string, unknown>;
      result.range = null;
    },
    (rows: HoverCapture[]) => {
      rows[1].failure = "failed actual session";
    },
    (rows: HoverCapture[]) => {
      rows[1].wire.serverWireBase64 = Buffer.from(rows[1].wire.serverWireBase64, "base64")
        .subarray(0, -1)
        .toString("base64");
    },
    (rows: HoverCapture[]) => {
      const result = rows[1].responses[0].response!.result as { contents: { value: string } };
      result.contents.value += " removed or changed full documentation";
    },
    (rows: HoverCapture[]) => {
      const result = rows[0].responses[0].response!.result as {
        range: { end: { character: number } };
      };
      result.range.end.character = 24;
    },
  ];
  for (const mutate of mutations) {
    const rows = comparatorControls();
    const beforeFrames = rows.map(({ wire }) => wire.serverWireBase64);
    mutate(rows);
    // Keep complete returned frames in sync for shape/content mutations: these
    // refusals must reach the whole-answer comparator, not only the wire join.
    if (
      rows.length === 4 &&
      rows.every(({ responses }) => responses.length === 3) &&
      rows.every(({ wire }, index) => wire.serverWireBase64 === beforeFrames[index])
    ) {
      for (const row of rows) {
        const bytes = Buffer.from(row.wire.serverWireBase64, "base64");
        const messages = decodeFrames(bytes).messages.map(
          (message) =>
            row.responses.find(({ response }) => response?.id === message.id)?.response ?? message,
        );
        row.wire.serverWireBase64 = Buffer.concat(messages.map(frameMessage)).toString("base64");
      }
    }
    assert.throws(() => assertWholeNativeHovers(corpus, rows));
  }
});
