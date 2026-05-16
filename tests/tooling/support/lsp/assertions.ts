import assert from "node:assert/strict";
import type { LspSession } from "./session.ts";
import type { LspDiagnostic, PublishDiagnosticsParams } from "./protocol.ts";

export function isDiagnosticsForUri(
  params: unknown,
  uri: string,
): params is PublishDiagnosticsParams {
  return (
    typeof params === "object" &&
    params != null &&
    "uri" in params &&
    (params as { uri?: unknown }).uri === uri &&
    "diagnostics" in params &&
    Array.isArray((params as { diagnostics?: unknown }).diagnostics)
  );
}

export function hasDiagnosticSource(params: unknown, uri: string, source: string): boolean {
  return (
    isDiagnosticsForUri(params, uri) &&
    params.diagnostics.some((diagnostic) => diagnostic.source === source)
  );
}

export function assertDiagnosticRange(diagnostic: LspDiagnostic): void {
  const { range } = diagnostic;
  assert.ok(range?.start, "diagnostic should include a start range");
  assert.ok(range?.end, "diagnostic should include an end range");
  assert.equal(typeof range.start.line, "number");
  assert.equal(typeof range.start.character, "number");
  assert.equal(typeof range.end.line, "number");
  assert.equal(typeof range.end.character, "number");
  assert.ok((range.end.line ?? 0) >= (range.start.line ?? 0));
}

/**
 * Verifies that editor-only requests return the inert value for documents the
 * server should not currently index.
 *
 * The same list is used for unopened and closed documents so newly added editor
 * capabilities do not accidentally retain stale virtual files after close.
 */
export async function assertEmptyEditorRequests(
  session: LspSession,
  uri: string,
  label: string,
): Promise<void> {
  const position = { line: 0, character: 0 };
  const range = { start: position, end: position };
  const requests: Array<[method: string, params: unknown]> = [
    [
      "textDocument/hover",
      {
        textDocument: { uri },
        position,
      },
    ],
    [
      "textDocument/definition",
      {
        textDocument: { uri },
        position,
      },
    ],
    [
      "textDocument/references",
      {
        textDocument: { uri },
        position,
        context: {
          includeDeclaration: true,
        },
      },
    ],
    [
      "textDocument/completion",
      {
        textDocument: { uri },
        position,
      },
    ],
    [
      "textDocument/documentSymbol",
      {
        textDocument: { uri },
      },
    ],
    [
      "textDocument/documentLink",
      {
        textDocument: { uri },
      },
    ],
    [
      "textDocument/semanticTokens/full",
      {
        textDocument: { uri },
      },
    ],
    [
      "textDocument/codeLens",
      {
        textDocument: { uri },
      },
    ],
    [
      "textDocument/inlayHint",
      {
        textDocument: { uri },
        range,
      },
    ],
    [
      "textDocument/foldingRange",
      {
        textDocument: { uri },
      },
    ],
    [
      "textDocument/codeAction",
      {
        textDocument: { uri },
        range,
        context: {
          diagnostics: [],
        },
      },
    ],
    [
      "textDocument/prepareRename",
      {
        textDocument: { uri },
        position,
      },
    ],
    [
      "textDocument/rename",
      {
        textDocument: { uri },
        position,
        newName: "renamedSymbol",
      },
    ],
    [
      "textDocument/formatting",
      {
        textDocument: { uri },
        options: {
          tabSize: 2,
          insertSpaces: true,
        },
      },
    ],
    [
      "textDocument/rangeFormatting",
      {
        textDocument: { uri },
        range,
        options: {
          tabSize: 2,
          insertSpaces: true,
        },
      },
    ],
  ];

  for (const [method, params] of requests) {
    const response = await session.request(method, params);
    assert.equal(response, null, `${method} should return null for ${label}`);
  }
}

export function assertNoDiagnosticSource(diagnostics: LspDiagnostic[], source: string): void {
  assert.equal(
    diagnostics.some((diagnostic) => diagnostic.source === source),
    false,
    `unexpected ${source} diagnostic: ${JSON.stringify(diagnostics)}`,
  );
}

export function offsetToPosition(
  source: string,
  offset: number,
): {
  line: number;
  character: number;
} {
  const lines = source.slice(0, offset).split("\n");
  return {
    line: lines.length - 1,
    character: lines.at(-1)?.length ?? 0,
  };
}

export function firstLocation(
  response:
    | Array<{ uri: string; range: { start: { line: number; character: number } } }>
    | { uri: string; range: { start: { line: number; character: number } } },
): { uri: string; range: { start: { line: number; character: number } } } {
  return Array.isArray(response) ? response[0] : response;
}

export function hoverToText(hover: { contents?: unknown } | null): string {
  assert.ok(hover?.contents);

  const contents = hover.contents;
  if (typeof contents === "string") {
    return contents;
  }
  if (Array.isArray(contents)) {
    return contents.map(markedStringToText).join("\n\n");
  }
  if (typeof contents === "object" && contents != null && "value" in contents) {
    const value = (contents as { value?: unknown }).value;
    return typeof value === "string" ? value : JSON.stringify(contents);
  }

  return JSON.stringify(contents);
}

function markedStringToText(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }
  if (typeof value === "object" && value != null && "value" in value) {
    const text = (value as { value?: unknown }).value;
    return typeof text === "string" ? text : JSON.stringify(value);
  }
  return JSON.stringify(value);
}

export function completionLabels(
  response: Array<{ label: string }> | { items?: Array<{ label: string }> } | null,
): string[] {
  if (response == null) {
    return [];
  }
  if (Array.isArray(response)) {
    return response.map((item) => item.label);
  }
  return (response.items ?? []).map((item) => item.label);
}
