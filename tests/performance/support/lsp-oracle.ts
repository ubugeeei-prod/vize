import assert from "node:assert/strict";

import { isDiagnosticsForUri, offsetToPosition } from "../../tooling/support/lsp/assertions.ts";
import type {
  LspDiagnostic,
  PublishDiagnosticsParams,
} from "../../tooling/support/lsp/protocol.ts";
import type { LspSession } from "../../tooling/support/lsp/session.ts";
import type { IncrementalMetrics } from "./incremental-metrics.ts";

export const diagnosticsTimeoutMs = 120_000;

/**
 * Where an injected `string`-into-`number` mismatch must be reported:
 * `"declaration"` anchors at the injected `const <symbol>` name, while
 * `{ attributeName }` anchors at a component prop attribute, exactly where
 * vue-tsc reports child prop-type mismatches.
 */
export type MismatchAnchor = "declaration" | { attributeName: string };

export function replaceExactly(source: string, expected: string, replacement: string): string {
  const first = source.indexOf(expected);
  assert.notEqual(first, -1, `missing patch anchor: ${expected}`);
  assert.equal(
    source.indexOf(expected, first + expected.length),
    -1,
    "patch anchor must be unique",
  );
  return `${source.slice(0, first)}${replacement}${source.slice(first + expected.length)}`;
}

export async function changeVue(
  session: LspSession,
  metrics: IncrementalMetrics,
  change: { name: string; uri: string; version: number; source: string; expectError?: boolean },
): Promise<PublishDiagnosticsParams> {
  return metrics.measure(change.name, async () => {
    session.notify("textDocument/didChange", {
      textDocument: { uri: change.uri, version: change.version },
      contentChanges: [{ text: change.source }],
    });
    return waitForDiagnostics(session, change.uri, change.version, change.expectError);
  });
}

export async function waitForDiagnostics(
  session: LspSession,
  uri: string,
  version: number,
  expectError?: boolean,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => {
      if (!isDiagnosticsForUri(params, uri) || params.version !== version) return false;
      return expectError == null || hasInjectedMismatch(params.diagnostics) === expectError;
    },
    diagnosticsTimeoutMs,
  )) as PublishDiagnosticsParams;
}

export function hasInjectedMismatch(diagnostics: LspDiagnostic[]): boolean {
  return diagnostics.some(
    (diagnostic) =>
      String(diagnostic.code).replace(/^TS/, "") === "2322" &&
      /string.*not assignable.*number/i.test(diagnostic.message ?? ""),
  );
}

export function assertSingleInjectedMismatch(
  diagnostics: LspDiagnostic[],
  baseline: string[],
  source: string,
  symbol: string,
  anchor: MismatchAnchor = "declaration",
): void {
  const injected = diagnostics.filter(
    (diagnostic) =>
      String(diagnostic.code).replace(/^TS/, "") === "2322" &&
      /string.*not assignable.*number/i.test(diagnostic.message ?? ""),
  );
  assert.equal(injected.length, 1, JSON.stringify(diagnostics));
  const [diagnostic] = injected;
  assert.equal(diagnostic.source, "vize/types");
  assert.equal(diagnostic.severity, 1);
  if (anchor === "declaration") {
    const declarationOffset = source.indexOf(`const ${symbol}`);
    assert.notEqual(declarationOffset, -1);
    const start = offsetToPosition(source, declarationOffset + "const ".length);
    const end = { line: start.line, character: start.character + symbol.length };
    assert.deepEqual(diagnostic.range?.start, start);
    assert.deepEqual(diagnostic.range?.end, end);
  } else {
    const attributeOffset = source.indexOf(`:${anchor.attributeName}="String(${symbol})"`);
    assert.notEqual(attributeOffset, -1);
    const start = offsetToPosition(source, attributeOffset + ":".length);
    const end = { line: start.line, character: start.character + anchor.attributeName.length };
    assert.deepEqual(diagnostic.range?.start, start);
    assert.deepEqual(diagnostic.range?.end, end);
  }
  assert.deepEqual(
    normalizeDiagnostics(diagnostics.filter((item) => item !== diagnostic)),
    baseline,
  );
}

export function normalizeDiagnostics(diagnostics: LspDiagnostic[]): string[] {
  return diagnostics
    .map((diagnostic) =>
      JSON.stringify({
        code: diagnostic.code,
        message: diagnostic.message,
        range: diagnostic.range,
        severity: diagnostic.severity,
        source: diagnostic.source,
      }),
    )
    .sort();
}

export function positionInsideTemplateSymbol(
  source: string,
  symbol: string,
  prefix: string,
): { line: number; character: number } {
  const templateOffset = source.indexOf(`{{ ${symbol} }}`);
  assert.notEqual(templateOffset, -1);
  return offsetToPosition(source, templateOffset + "{{ ".length + prefix.length);
}
