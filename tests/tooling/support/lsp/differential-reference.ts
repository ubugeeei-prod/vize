import assert from "node:assert/strict";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { sourceSessionUnsupportedLegacy } from "../../../differential/lsp-source-sessions.ts";
import { lspWorkspace } from "../../../differential/lsp-workspace.ts";
import { BUILD_RECIPE } from "../../../differential/build-receipt.ts";
import { sha256 } from "../../../differential/harness.mjs";
import {
  initializeCapabilities,
  fixtureLanguageId,
  materializeWorkspace,
  NATIVE_REASON,
} from "../../../differential/lsp-manifest.ts";
import { inspectLspObservation } from "../../../differential/lsp-report.ts";
import { decodeFrames, frameMessage } from "../../../differential/lsp-wire.ts";
import type {
  JsonRpcMessage,
  LspFixture,
  WireObservation,
  LspReport,
  LoadedLspManifest,
} from "../../../differential/lsp-types.ts";

// Synthetic frames exercise validator laws, never product runtime evidence.
export function referenceObservation(fixture: LspFixture): WireObservation {
  assert.equal(
    fixture.data.sourceSession,
    undefined,
    "source transitions cannot use synthetic initial-document wire",
  );
  const workspace = path.join(os.tmpdir(), "lsp-fixture-law");
  const workspaceUri = pathToFileURL(workspace).href;
  const entryUri = pathToFileURL(path.join(workspace, fixture.entry)).href;
  const client: JsonRpcMessage[] = [
    {
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: {
        processId: 123,
        rootUri: workspaceUri,
        capabilities: initializeCapabilities(fixture),
        initializationOptions: fixture.initializationOptions,
        workspaceFolders: [{ uri: workspaceUri, name: path.basename(workspace) }],
      },
    },
    { jsonrpc: "2.0", method: "initialized", params: {} },
  ];
  const server: JsonRpcMessage[] = [{ jsonrpc: "2.0", id: 1, result: { capabilities: {} } }];
  if (fixture.data.openDocument !== false) {
    client.push({
      jsonrpc: "2.0",
      method: "textDocument/didOpen",
      params: {
        textDocument: {
          uri: entryUri,
          languageId: fixtureLanguageId(fixture.data),
          version: fixture.data.documentVersion,
          text: fixture.files.find((file) => file.runtimePath === fixture.entry)?.bytes.toString(),
        },
      },
    });
    server.push({
      jsonrpc: "2.0",
      method: "textDocument/publishDiagnostics",
      params: {
        uri: entryUri,
        version: fixture.data.documentVersion,
        diagnostics: materializeWorkspace(fixture.data.initialDiagnostics ?? [], workspaceUri),
      },
    });
  }
  for (const [index, request] of fixture.requests.entries()) {
    const id = index + 2;
    client.push({
      jsonrpc: "2.0",
      id,
      method: request.method ?? fixture.data.method,
      params: { textDocument: { uri: entryUri }, ...request.params },
    });
    server.push({ jsonrpc: "2.0", id, result: materializeWorkspace(request.result, workspaceUri) });
  }
  const shutdownId = fixture.requests.length + 2;
  client.push({ jsonrpc: "2.0", id: shutdownId, method: "shutdown" });
  server.push({ jsonrpc: "2.0", id: shutdownId, result: null });
  client.push({ jsonrpc: "2.0", method: "exit" });
  const observation: WireObservation = {
    clientWireBase64: "",
    serverWireBase64: "",
    workspaceUri,
    stderrBase64: "",
    exitStatus: 0,
    signal: null,
    processError: null,
  };
  const streams = { client, server };
  for (const stream of ["client", "server"] as const) {
    const messages = streams[stream];
    const bytes = Buffer.concat(messages.map(frameMessage));
    observation[`${stream}WireBase64`] = bytes.toString("base64");
    observation[`${stream}WireSha256`] = sha256(bytes);
  }
  return observation;
}

export function rewriteFrames(
  observation: WireObservation,
  stream: "client" | "server",
  mutate: (messages: JsonRpcMessage[]) => void,
): void {
  const messages = decodeFrames(Buffer.from(observation[`${stream}WireBase64`], "base64")).messages;
  mutate(messages);
  const bytes = Buffer.concat(messages.map(frameMessage));
  observation[`${stream}WireBase64`] = bytes.toString("base64");
  observation[`${stream}WireSha256`] = sha256(bytes);
}

export function referenceReport(
  loaded: LoadedLspManifest,
  build: { sourceRevision: string; binaryPath: string; binarySha256: string; cliVersion: string },
): LspReport {
  return {
    schema: "vize.differential.result",
    version: 1,
    product: "lsp",
    sourceRevision: build.sourceRevision,
    manifestSha256: loaded.manifestSha256,
    argv: ["lsp"],
    buildReceipt: { schema: "vize.differential.build", version: 1, recipe: BUILD_RECIPE, ...build },
    binary: { path: build.binaryPath, sha256: build.binarySha256, version: build.cliVersion },
    binaryProbe: {
      exitStatus: 0,
      signal: null,
      stdoutBase64: Buffer.from(`${build.cliVersion}\n`).toString("base64"),
      stderrBase64: "",
      processError: null,
    },
    rows: loaded.cases.map((fixture) => {
      if (fixture.data.sourceSession)
        return {
          id: fixture.id,
          legacy: sourceSessionUnsupportedLegacy(fixture.data.sourceSession),
          native: { state: "unsupported", reason: NATIVE_REASON },
          comparison: { state: "not-compared", reason: NATIVE_REASON },
        };
      const observation = referenceObservation(fixture);
      const mapping = lspWorkspace(fixture).mapping;
      return {
        id: fixture.id,
        legacy: {
          state: "completed",
          verdict: "matched-reference",
          comparator: "complete-jsonrpc-response",
          observation,
          workspace: {
            mapping,
            before: structuredClone(mapping.files),
            after: structuredClone(mapping.files),
          },
          responses: inspectLspObservation(fixture, observation),
        },
        native: { state: "unsupported", reason: NATIVE_REASON },
        comparison: { state: "not-compared", reason: NATIVE_REASON },
      };
    }),
    summary: {
      plannedCases: loaded.cases.length,
      legacyMatches: loaded.cases.filter((fixture) => !fixture.data.sourceSession).length,
      legacyFailures: 0,
      legacyUnsupported: loaded.cases.filter((fixture) => fixture.data.sourceSession).length,
      baselineDrift: 0,
      nativeUnsupported: loaded.cases.length,
      pairedComparisons: 0,
      nativeHandled: 0,
      nativeEquivalent: 0,
    },
  };
}
