import { completionLabels } from "./lsp/assertions.ts";
import type { LspRange, PublishDiagnosticsParams } from "./lsp/protocol.ts";
import {
  diagnosticPayload,
  textDocumentPosition,
  type CompletionResponse,
  type OracleSession,
} from "./real-project-lsp-authored-utils.ts";

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
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (value) => diagnosticPayload(value, uri, version) != null,
    timeoutMs,
  )) as PublishDiagnosticsParams;
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
