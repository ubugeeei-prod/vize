import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";
import { validateBuildReceipt } from "./build-receipt.ts";
import { sha256, validateResultEnvelope } from "./harness.mjs";
import { initializeCapabilities, materializeWorkspace, NATIVE_REASON } from "./lsp-manifest.ts";
import { decodeFrames } from "./lsp-wire.ts";
import {
  hasCapabilities,
  type JsonRpcMessage,
  type LspFixture,
  type WireObservation,
  type ResponseComparison,
  type LoadedLspManifest,
  type LspReport,
  type BuildIdentity,
} from "./lsp-types.ts";

function uniqueResponse(messages: JsonRpcMessage[], id: number): JsonRpcMessage {
  const responses = messages.filter((message) => message.id === id && !message.method);
  assert.equal(responses.length, 1, `one complete response required for request ${id}`);
  return responses[0];
}

export function inspectLspObservation(
  fixture: LspFixture,
  observation: WireObservation,
): ResponseComparison[] {
  assert.equal(observation.exitStatus, 0, "successful shutdown is required");
  assert.equal(observation.signal, null);
  assert.equal(observation.processError, null);
  for (const stream of ["client", "server"] as const) {
    assert.equal(typeof observation[`${stream}WireBase64`], "string");
    const bytes = Buffer.from(observation[`${stream}WireBase64`], "base64");
    assert.equal(sha256(bytes), observation[`${stream}WireSha256`]);
  }
  const workspaceUri = observation.workspaceUri;
  assert(typeof workspaceUri === "string");
  assert.equal(new URL(workspaceUri).protocol, "file:");
  const workspace = fileURLToPath(workspaceUri);
  assert.equal(pathToFileURL(workspace).href, workspaceUri, "canonical workspace URI required");
  const entryUri = pathToFileURL(path.join(workspace, fixture.entry)).href;
  const client = decodeFrames(Buffer.from(observation.clientWireBase64, "base64")).messages;
  const server = decodeFrames(Buffer.from(observation.serverWireBase64, "base64")).messages;
  const replies = server.filter((message) => Object.hasOwn(message, "id") && !message.method);
  assert.equal(replies.length, fixture.requests.length + 2, "all server replies must be accounted");
  assert.deepEqual(
    replies.map((reply) => reply.id),
    Array.from({ length: fixture.requests.length + 2 }, (_, index) => index + 1),
    "sequential requests must retain response order and identity",
  );
  assert.equal(client.length, fixture.requests.length + 5, "complete authored lifecycle required");
  const initializeParams = client[0]?.params;
  assert(initializeParams && Number.isSafeInteger(initializeParams.processId));
  assert.deepEqual(client[0], {
    jsonrpc: "2.0",
    id: 1,
    method: "initialize",
    params: {
      processId: initializeParams.processId,
      rootUri: workspaceUri,
      capabilities: initializeCapabilities(fixture),
      initializationOptions: fixture.initializationOptions,
      workspaceFolders: [{ uri: workspaceUri, name: path.basename(workspace) }],
    },
  });
  const initialize = uniqueResponse(server, 1);
  assert.equal(initialize.jsonrpc, "2.0");
  assert(!Object.hasOwn(initialize, "error") && hasCapabilities(initialize));
  assert.deepEqual(client[1], { jsonrpc: "2.0", method: "initialized", params: {} });
  assert.deepEqual(client[2], {
    jsonrpc: "2.0",
    method: "textDocument/didOpen",
    params: {
      textDocument: {
        uri: entryUri,
        languageId: "vue",
        version: fixture.data.documentVersion,
        text: fixture.files
          .find((file) => file.runtimePath === fixture.entry)
          ?.bytes.toString("utf8"),
      },
    },
  });
  const diagnosticsIndex = server.findIndex(
    (message) =>
      message.method === "textDocument/publishDiagnostics" &&
      message.params?.uri === entryUri &&
      message.params?.version === fixture.data.documentVersion &&
      Array.isArray(message.params?.diagnostics),
  );
  assert(diagnosticsIndex >= 0, "exact entry and version diagnostics are required");
  const comparisons = fixture.requests.map((request, index) => {
    const id = index + 2;
    assert.deepEqual(client[index + 3], {
      jsonrpc: "2.0",
      id,
      method: request.method ?? fixture.data.method,
      params: { textDocument: { uri: entryUri }, ...request.params },
    });
    const response = uniqueResponse(server, id);
    assert(server.indexOf(response) > diagnosticsIndex, "response must follow readiness");
    const expected = {
      jsonrpc: "2.0",
      id,
      result: materializeWorkspace(request.result, workspaceUri),
    };
    return { requestId: id, state: isDeepStrictEqual(response, expected) ? "equal" : "different" };
  });
  const shutdownId = fixture.requests.length + 2;
  assert.deepEqual(client.at(-2), { jsonrpc: "2.0", id: shutdownId, method: "shutdown" });
  assert.deepEqual(uniqueResponse(server, shutdownId), {
    jsonrpc: "2.0",
    id: shutdownId,
    result: null,
  });
  assert.deepEqual(client.at(-1), { jsonrpc: "2.0", method: "exit" });
  return comparisons;
}

export function validateLspReport(
  loaded: LoadedLspManifest,
  report: LspReport,
  expectedBuild: BuildIdentity,
) {
  validateResultEnvelope(loaded, report, expectedBuild.sourceRevision);
  assert.deepEqual(report.argv, ["lsp"]);
  if (report.buildReceipt) validateBuildReceipt(report.buildReceipt, expectedBuild);
  if (report.binary) {
    assert(report.buildReceipt, "completed executable observations require a source-build receipt");
    assert(report.binaryProbe, "raw executable version probe is required");
    assert.equal(report.binaryProbe.exitStatus, 0);
    assert.equal(report.binaryProbe.signal, null);
    assert.equal(report.binaryProbe.processError, null);
    assert.equal(
      Buffer.from(report.binaryProbe.stdoutBase64, "base64").toString().trim(),
      expectedBuild.cliVersion,
    );
    assert.deepEqual(report.binary, {
      path: expectedBuild.binaryPath,
      sha256: expectedBuild.binarySha256,
      version: expectedBuild.cliVersion,
    });
  }
  const rows = new Map(report.rows.map((row) => [row.id, row]));
  const summary = {
    plannedCases: loaded.cases.length,
    legacyMatches: 0,
    legacyFailures: 0,
    baselineDrift: 0,
    nativeUnsupported: loaded.cases.length,
    pairedComparisons: 0,
    nativeHandled: 0,
    nativeEquivalent: 0,
  };
  for (const fixture of loaded.cases) {
    const row = rows.get(fixture.id);
    assert(row, "every planned row is required");
    assert.deepEqual(row.native, { state: "unsupported", reason: NATIVE_REASON });
    assert.deepEqual(row.comparison, { state: "not-compared", reason: NATIVE_REASON });
    assert.equal(row.legacy.comparator, "complete-jsonrpc-response");
    assert(["completed", "failed"].includes(row.legacy.state));
    if (row.legacy.state === "failed") {
      assert.equal(row.legacy.verdict, "failed");
      assert(typeof row.legacy.error === "string");
      assert(row.legacy.error.length > 0);
      summary.legacyFailures += 1;
    } else {
      assert(report.binary, "completed response comparison requires executable identity");
      assert(row.legacy.observation, "complete wire observation required");
      const comparisons = inspectLspObservation(fixture, row.legacy.observation);
      assert.deepEqual(row.legacy.responses, comparisons, "forged response comparison");
      const matched = comparisons.every((comparison) => comparison.state === "equal");
      assert.equal(row.legacy.verdict, matched ? "matched-reference" : "baseline-drift");
      summary[matched ? "legacyMatches" : "baselineDrift"] += 1;
    }
  }
  assert.deepEqual(report.summary, summary, "every planned LSP case must remain accounted");
  return summary;
}
