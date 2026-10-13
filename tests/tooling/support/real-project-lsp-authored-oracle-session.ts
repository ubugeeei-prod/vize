import assert from "node:assert/strict";

import { completionLabels } from "./lsp/assertions.ts";
import type { LspRange, PublishDiagnosticsParams, WorkspaceEdit } from "./lsp/protocol.ts";
import {
  diagnosticPayload,
  textDocumentPosition,
  type CompletionResponse,
  type Location,
  type OracleSession,
} from "./real-project-lsp-authored-utils.ts";
import type { FixtureProject, LspAuthoredOracle } from "./real-project-lsp-report.ts";

export function open(
  session: OracleSession,
  uri: string,
  text: string,
  version: number,
  uris: string[],
) {
  session.notify("textDocument/didOpen", {
    textDocument: { languageId: "vue", text, uri, version },
  });
  uris.push(uri);
}

export function change(session: OracleSession, uri: string, text: string, version: number) {
  session.notify("textDocument/didChange", {
    contentChanges: [{ text }],
    textDocument: { uri, version },
  });
}

export async function waitForDiagnostics(
  session: OracleSession,
  uri: string,
  version: number,
  timeoutMs: number,
): Promise<PublishDiagnosticsParams> {
  try {
    return (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (value) => diagnosticPayload(value, uri, version) != null,
      timeoutMs,
    )) as PublishDiagnosticsParams;
  } catch (error) {
    throw new Error(`Waiting for diagnostics for ${uri} version ${version}: ${String(error)}`);
  }
}

export async function requestCompletion(
  session: OracleSession,
  uri: string,
  position: LspRange["start"],
  timeoutMs: number,
) {
  return completionLabels(
    (await session.request(
      "textDocument/completion",
      textDocumentPosition(uri, position),
      timeoutMs,
    )) as CompletionResponse,
  );
}

/** Every edit of a rename, each checked to write `newText`. */
export function renameEditLocations(rename: WorkspaceEdit | null, newText: string): Location[] {
  return Object.entries(rename?.changes ?? {}).flatMap(([uri, edits]) =>
    edits.map((edit) => {
      assert.equal(edit.newText, newText, `rename text in ${uri}`);
      return { range: edit.range, uri };
    }),
  );
}

export function assertOracleFilesAreInCorpus(
  project: FixtureProject,
  files: string[],
  oracle: LspAuthoredOracle,
): void {
  for (const file of [
    oracle.templateBinding.file,
    oracle.componentBoundary.importerFile,
    oracle.componentBoundary.componentFile,
  ]) {
    assert.ok(
      files.includes(file),
      `${project.id} authored LSP oracle file is outside vueGlobs: ${file}`,
    );
  }
}
