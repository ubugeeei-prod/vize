import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { readPinnedArtifact, sha256 } from "../../../differential/harness.mjs";
import type { JsonRpcMessage, WireObservation } from "../../../differential/lsp-types.ts";
import { decodeFrames } from "../../../differential/lsp-wire.ts";

type Artifact = { path: string; sha256: string };
type Position = { line: number; character: number };
export type HoverRequest = {
  role: string;
  position: Position;
  range: { start: Position; end: Position };
  signature: string;
};
export type HoverSession = { id: string; file: Artifact; requests: HoverRequest[] };
export type HoverCorpus = {
  schema: string;
  version: number;
  source: Artifact;
  config: Artifact;
  originals: Artifact[];
  plannedSessions: number;
  plannedWholeResponses: number;
  initialization: { capabilities: Record<string, unknown>; options: Record<string, boolean> };
  sessions: HoverSession[];
};
export type HoverCapture = {
  id: string;
  workspaceUri: string;
  rootUri: string;
  failure: string | null;
  responses: { role: string; response: JsonRpcMessage | null; error: string | null }[];
  wire: WireObservation;
};

const IDS = ["original-lf", "original-crlf", "unicode-lf", "unicode-crlf"];
const ROLES = ["for-name", "id-name", "id-value"];
const SIGNATURES = [
  "(property) HTMLLabelElement.htmlFor: string",
  "(property) Element.id: string",
  'const id: "field"',
];
const ORIGINAL_PINS = [
  ["Field.vue.txt", "24ae0b5b278779dd760ae7fd6bcd3926dbcb87a281e5675f16c13d8b354833e7"],
  ["lsp-req.mjs.txt", "4f2d195d15af1617c5400e66b1077c3908f959ae5e7281da99687100d8173c29"],
  ["issue-body.md", "b972cf7b7992c26d057c3d62bcf8c8bb368d6284606d9018f80d16acb17886b5"],
];

export function validateHoverCorpus(corpus: HoverCorpus, fixtureRoot: string): void {
  assert.equal(corpus.schema, "vize.native-attribute-hover-range.corpus");
  assert.equal(corpus.version, 1);
  assert.equal(corpus.plannedSessions, 4);
  assert.equal(corpus.plannedWholeResponses, 12);
  assert.deepEqual(
    corpus.sessions.map(({ id }) => id),
    IDS,
  );
  assert.deepEqual(
    corpus.originals.map(({ path: file, sha256: hash }) => [file, hash]),
    ORIGINAL_PINS,
  );
  assert.deepEqual(corpus.config, {
    path: "tsconfig.json.txt",
    sha256: "3e69f1b19380514b0a42913c6dc21a2fffed65855e4bad30534c3f6a71116b23",
  });
  assert.deepEqual(corpus.initialization, {
    capabilities: {},
    options: { editor: true, hover: true, lint: false, typecheck: true },
  });
  for (const artifact of [corpus.source, corpus.config, ...corpus.originals]) {
    readPinnedArtifact(fixtureRoot, artifact);
  }
  for (const [index, session] of corpus.sessions.entries()) {
    const lines = readPinnedArtifact(fixtureRoot, session.file).toString("utf8").split(/\r?\n/);
    const positions =
      index < 2
        ? [
            [5, 10, 13],
            [6, 10, 12],
            [6, 14, 16],
          ]
        : [
            [5, 23, 26],
            [6, 22, 24],
            [6, 26, 28],
          ];
    assert.deepEqual(
      session.requests.map(({ role }) => role),
      ROLES,
    );
    for (const [requestIndex, request] of session.requests.entries()) {
      const [line, start, end] = positions[requestIndex];
      assert.deepEqual(request.position, { line, character: start });
      assert.deepEqual(request.range, {
        start: { line, character: start },
        end: { line, character: end },
      });
      assert.equal(request.signature, SIGNATURES[requestIndex]);
      assert.equal(lines[line].slice(start, end), requestIndex === 0 ? "for" : "id");
    }
  }
}

export function loadHoverCorpus(fixtureRoot: string) {
  const bytes = fs.readFileSync(path.join(fixtureRoot, "manifest.json"));
  const corpus = JSON.parse(bytes.toString("utf8")) as HoverCorpus;
  validateHoverCorpus(corpus, fixtureRoot);
  return { corpus, manifestSha256: sha256(bytes) as string };
}

export function assertWholeNativeHovers(corpus: HoverCorpus, captures: HoverCapture[]): void {
  assert.deepEqual(
    captures.map(({ id }) => id),
    IDS,
    "all four sessions are mandatory",
  );
  const baseline = captures[0].responses;
  for (const [sessionIndex, capture] of captures.entries()) {
    assert.equal(capture.failure, null, `${capture.id}: complete session`);
    assert.equal(capture.wire.exitStatus, 0);
    assert.equal(capture.wire.signal, null);
    assert.equal(capture.wire.processError, null);
    const client = decodeFrames(Buffer.from(capture.wire.clientWireBase64, "base64")).messages;
    const server = decodeFrames(Buffer.from(capture.wire.serverWireBase64, "base64")).messages;
    assert.deepEqual(
      client.map(({ method }) => method),
      [
        "initialize",
        "initialized",
        "textDocument/didOpen",
        "textDocument/hover",
        "textDocument/hover",
        "textDocument/hover",
        "shutdown",
        "exit",
      ],
    );
    const initialization = client[0].params;
    assert.ok(initialization && Number.isSafeInteger(initialization.processId));
    assert.deepEqual(client[0], {
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: {
        processId: initialization.processId,
        rootUri: capture.rootUri,
        capabilities: corpus.initialization.capabilities,
        initializationOptions: corpus.initialization.options,
      },
    });
    assert.deepEqual(client[1], { jsonrpc: "2.0", method: "initialized", params: {} });
    const opened = client[2].params?.textDocument as Record<string, unknown>;
    assert.equal(typeof opened?.text, "string");
    assert.equal(
      sha256(Buffer.from(opened.text as string)),
      corpus.sessions[sessionIndex].file.sha256,
    );
    assert.deepEqual(client[2], {
      jsonrpc: "2.0",
      method: "textDocument/didOpen",
      params: {
        textDocument: {
          uri: capture.workspaceUri,
          languageId: "vue",
          text: opened.text,
          version: 1,
        },
      },
    });
    const initialized = server.filter(({ id }) => id === 1);
    assert.equal(initialized.length, 1);
    assert.deepEqual(Object.keys(initialized[0]).sort(), ["id", "jsonrpc", "result"]);
    const capabilities = (initialized[0].result as { capabilities: Record<string, unknown> })
      .capabilities;
    assert.equal(capabilities.hoverProvider, true);
    assert.ok(
      server.some(
        ({ method, params }) =>
          method === "textDocument/publishDiagnostics" &&
          params?.uri === capture.workspaceUri &&
          params?.version === 1,
      ),
    );
    assert.deepEqual(
      capture.responses.map(({ role }) => role),
      ROLES,
    );
    for (const [index, outcome] of capture.responses.entries()) {
      const request = corpus.sessions[sessionIndex].requests[index];
      assert.equal(outcome.error, null);
      const response = outcome.response;
      assert.ok(response);
      assert.deepEqual(Object.keys(response).sort(), ["id", "jsonrpc", "result"]);
      assert.equal(response.id, index + 2);
      assert.equal(response.jsonrpc, "2.0");
      assert.deepEqual(
        server.filter(({ id }) => id === response.id),
        [response],
      );
      const hover = response.result as {
        contents: { kind: string; value: string };
        range: unknown;
      };
      assert.ok(hover && typeof hover === "object");
      assert.deepEqual(Object.keys(hover).sort(), ["contents", "range"]);
      assert.deepEqual(Object.keys(hover.contents).sort(), ["kind", "value"]);
      assert.equal(hover.contents.kind, "markdown");
      assert.equal(typeof hover.contents.value, "string");
      assert.ok(
        hover.contents.value.includes(request.signature),
        `${outcome.role}: real native type`,
      );
      const baselineHover = baseline[index].response?.result as typeof hover;
      assert.deepEqual(
        hover,
        { contents: baselineHover.contents, range: request.range },
        `${capture.id}/${outcome.role}: complete contents and physical range`,
      );
      assert.deepEqual(client[index + 3], {
        jsonrpc: "2.0",
        id: index + 2,
        method: "textDocument/hover",
        params: { textDocument: { uri: capture.workspaceUri }, position: request.position },
      });
    }
    assert.deepEqual(
      server.filter(({ id }) => id === 5),
      [{ jsonrpc: "2.0", id: 5, result: null }],
    );
    assert.deepEqual(client[6], { jsonrpc: "2.0", id: 5, method: "shutdown" });
    assert.deepEqual(client[7], { jsonrpc: "2.0", method: "exit" });
  }
}
