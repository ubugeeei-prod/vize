import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath, pathToFileURL } from "node:url";

import { offsetToPosition } from "./lsp/assertions.ts";
import type { LspDiagnostic, LspRange, PublishDiagnosticsParams } from "./lsp/protocol.ts";
import type { DiagnosticEvidence, LspResponseEvidence } from "./real-project-lsp-report.ts";

export type OracleSession = {
  notify(method: string, params: unknown): void;
  request(method: string, params: unknown, timeoutMs?: number): Promise<unknown>;
  waitForNotification(
    method: string,
    predicate?: (params: unknown) => boolean,
    timeoutMs?: number,
  ): Promise<unknown>;
};
export type Location = { range: LspRange; uri: string };
export type Hover = { contents?: unknown; range?: LspRange } | null;
export type CompletionResponse =
  | Array<{ label: string }>
  | { items?: Array<{ label: string }> }
  | null;

export function readOracleDocument(workspaceDir: string, relativeFile: string) {
  assert.equal(path.isAbsolute(relativeFile), false, "authored oracle files must be relative");
  const absolute = path.resolve(workspaceDir, relativeFile);
  const relative = path.relative(workspaceDir, absolute);
  assert.ok(
    relative.length > 0 && relative !== ".." && !relative.startsWith(`..${path.sep}`),
    `authored oracle file escapes the fixture: ${relativeFile}`,
  );
  assert.ok(
    fs.statSync(absolute, { throwIfNoEntry: false })?.isFile() === true,
    `authored oracle file is missing: ${relativeFile}`,
  );
  return { source: fs.readFileSync(absolute, "utf8"), uri: pathToFileURL(absolute).href };
}

export function anchoredSymbolRange(
  source: string,
  anchor: string,
  symbol: string,
  label: string,
): LspRange {
  const startOffset =
    uniqueAnchorOffset(source, anchor, label) + uniqueAnchorOffset(anchor, symbol, label);
  return {
    start: offsetToPosition(source, startOffset),
    end: offsetToPosition(source, startOffset + symbol.length),
  };
}

export function uniqueAnchorOffset(source: string, anchor: string, label: string): number {
  assert.ok(anchor.length > 0, `${label} anchor must not be empty`);
  const offset = source.indexOf(anchor);
  assert.ok(offset >= 0, `${label} anchor is missing: ${JSON.stringify(anchor)}`);
  assert.equal(
    source.lastIndexOf(anchor),
    offset,
    `${label} anchor must occur exactly once: ${JSON.stringify(anchor)}`,
  );
  return offset;
}

export function replaceUniqueAnchor(
  source: string,
  anchor: string,
  replacement: string,
  label: string,
): string {
  const offset = uniqueAnchorOffset(source, anchor, label);
  assert.notEqual(replacement, anchor, `${label} dependency edit must change the source`);
  return `${source.slice(0, offset)}${replacement}${source.slice(offset + anchor.length)}`;
}

export function textDocumentPosition(uri: string, position: LspRange["start"]) {
  return { position, textDocument: { uri } };
}

export function locations(response: unknown, message: string): Location[] {
  assert.ok(response, message);
  const values = Array.isArray(response) ? response : [response];
  assert.ok(values.length > 0, message);
  for (const value of values) {
    assert.ok(
      typeof value === "object" && value != null && "uri" in value && "range" in value,
      message,
    );
  }
  return values as Location[];
}

export function sortLocations(values: Location[]): Location[] {
  return [...values].sort((left, right) => compareKeys(locationKey(left), locationKey(right)));
}

export function sortTextEdits<T extends { newText: string; range: LspRange }>(values: T[]): T[] {
  return [...values].sort((left, right) => compareKeys(textEditKey(left), textEditKey(right)));
}

export function assertRangeInDocument(range: LspRange, source: string, label: string): void {
  const lines = source.split("\n");
  for (const [edge, position] of [
    ["start", range.start],
    ["end", range.end],
  ] as const) {
    assert.ok(
      position.line >= 0 && position.line < lines.length,
      `${label} ${edge} line is invalid`,
    );
    assert.ok(
      position.character >= 0 && position.character <= (lines[position.line]?.length ?? -1),
      `${label} ${edge} character is invalid`,
    );
  }
}

export function assertMissingModuleDiagnostic(
  published: PublishDiagnosticsParams,
  source: string,
  specifier: string,
): void {
  const matching = published.diagnostics.filter(
    (diagnostic) =>
      String(diagnostic.code).replace(/^TS/, "") === "2307" &&
      diagnostic.message?.includes(specifier),
  );
  assert.equal(matching.length, 1, `deleted dependency must produce one TS2307: ${specifier}`);
  const offset = uniqueAnchorOffset(source, specifier, "deleted dependency import");
  assert.deepEqual(matching[0], {
    code: 2307,
    message: `Cannot find module '${specifier}' or its corresponding type declarations.`,
    range: {
      start: offsetToPosition(source, offset - 1),
      end: offsetToPosition(source, offset + specifier.length + 1),
    },
    severity: 1,
    source: "vize/types",
  });
}

export function assertRankedLabels(
  actual: string[],
  expected: Array<{ label: string; rank: number }>,
  file: string,
): void {
  for (const item of expected) {
    assert.equal(
      actual[item.rank],
      item.label,
      `${file} completion rank ${item.rank} drifted; got ${actual.join(", ")}`,
    );
  }
}

export function responseEvidence(
  response: unknown,
  count: number,
  workspaceDir: string,
  durationMs = 0,
): LspResponseEvidence {
  const normalized = normalizeResponse(response, workspaceDir);
  return {
    count,
    durationMs,
    sha256: createHash("sha256")
      .update(`${JSON.stringify(normalized)}\n`)
      .digest("hex"),
  };
}

export async function timedRequest<T>(
  session: OracleSession,
  method: string,
  params: unknown,
  timeoutMs: number,
): Promise<{ durationMs: number; response: T }> {
  const started = performance.now();
  const response = (await session.request(method, params, timeoutMs)) as T;
  return { durationMs: elapsedMs(started), response };
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

export function diagnosticEvidence(diagnostics: LspDiagnostic[]): DiagnosticEvidence {
  const normalized = normalizeDiagnostics(diagnostics);
  return {
    count: normalized.length,
    sha256: createHash("sha256")
      .update(`${normalized.join("\n")}\n`)
      .digest("hex"),
  };
}

export function diagnosticPayload(
  value: unknown,
  uri: string,
  version: number,
): PublishDiagnosticsParams | null {
  if (typeof value !== "object" || value == null) return null;
  const payload = value as Partial<PublishDiagnosticsParams>;
  return payload.uri === uri && payload.version === version && Array.isArray(payload.diagnostics)
    ? (payload as PublishDiagnosticsParams)
    : null;
}

function normalizeResponse(value: unknown, workspaceDir: string): unknown {
  if (Array.isArray(value)) return value.map((item) => normalizeResponse(item, workspaceDir));
  if (typeof value === "string" && value.startsWith("file:")) {
    const resolved = fileURLToPath(value);
    const relative = path.relative(workspaceDir, resolved);
    return relative !== ".." && !relative.startsWith(`..${path.sep}`)
      ? relative.split(path.sep).join("/")
      : `<outside-workspace>/${path.basename(resolved)}`;
  }
  if (typeof value !== "object" || value == null) return value;
  return Object.fromEntries(
    Object.entries(value)
      .sort(([left], [right]) => compareKeys(left, right))
      .map(([key, nested]) => [key, normalizeResponse(nested, workspaceDir)]),
  );
}

function compareKeys(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

function elapsedMs(started: number): number {
  return Number(Math.max(0, performance.now() - started).toFixed(3));
}

function locationKey(location: Location): string {
  return `${location.uri}:${rangeKey(location.range)}`;
}

function textEditKey(edit: { newText: string; range: LspRange }): string {
  return `${rangeKey(edit.range)}:${edit.newText}`;
}

function rangeKey(range: LspRange): string {
  return [range.start.line, range.start.character, range.end.line, range.end.character]
    .map((value) => String(value).padStart(10, "0"))
    .join(":");
}
