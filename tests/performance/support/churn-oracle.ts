import assert from "node:assert/strict";

import type {
  LspDiagnostic,
  PublishDiagnosticsParams,
} from "../../tooling/support/lsp/protocol.ts";
import type { LspSession } from "../../tooling/support/lsp/session.ts";
import { hasInjectedMismatch, normalizeDiagnostics } from "./lsp-oracle.ts";

/**
 * One `textDocument/publishDiagnostics` notification observed passively on the
 * session, in arrival order. `payload` is the order-independent normalized
 * diagnostic fingerprint (message multiset plus ranges) used for determinism
 * comparisons; `version` is `null` for the spec-allowed versionless publishes
 * the server sends to clear diagnostics on `didClose`.
 */
export type PublishRecord = {
  uri: string;
  version: number | null;
  payload: string[];
  mismatch: boolean;
};

/**
 * Records every diagnostics publish without consuming the notifications that
 * `waitForDiagnostics` callers race for, so the suite can audit the complete
 * publish stream (duplicates, stale republishes, ordering) after the fact.
 */
export function recordPublishes(session: LspSession): PublishRecord[] {
  const records: PublishRecord[] = [];
  session.notificationObservers.push((method, params) => {
    if (method !== "textDocument/publishDiagnostics") return;
    const publish = params as PublishDiagnosticsParams;
    records.push({
      uri: publish.uri,
      version: publish.version ?? null,
      payload: normalizeDiagnostics(publish.diagnostics),
      mismatch: hasInjectedMismatch(publish.diagnostics),
    });
  });
  return records;
}

/**
 * Groups a stream slice into per-document payload sequences. Publish order is
 * only guaranteed per document (each document's publishes are causally ordered
 * by its recompute pipeline), so cross-document interleaving is deliberately
 * not part of the determinism fingerprint.
 */
export function perUriPayloadSequences(records: PublishRecord[]): Map<string, string[]> {
  const sequences = new Map<string, string[]>();
  for (const record of records) {
    if (record.version == null) continue;
    const key = JSON.stringify(record.payload);
    sequences.set(record.uri, [...(sequences.get(record.uri) ?? []), key]);
  }
  return sequences;
}

/** Two stream slices must publish identical per-document payload sequences. */
export function assertSameStream(
  actual: PublishRecord[],
  expected: PublishRecord[],
  label: string,
): void {
  assert.equal(actual.length, expected.length, `${label}: publish counts diverged`);
  assert.deepEqual(
    Object.fromEntries(perUriPayloadSequences(actual)),
    Object.fromEntries(perUriPayloadSequences(expected)),
    `${label}: per-document diagnostic payload sequences diverged`,
  );
}

/**
 * Versioned publishes must never move a document's version backwards: a
 * republish carrying an older version is a stale overlay escaping the server.
 * Versionless publishes are only legal as empty `didClose` clears at the very
 * end of the stream.
 */
export function assertStreamOrdering(records: PublishRecord[], closedUris: string[]): void {
  const lastVersion = new Map<string, number>();
  records.forEach((record, index) => {
    if (record.version == null) return;
    const previous = lastVersion.get(record.uri);
    assert.ok(
      previous == null || record.version >= previous,
      `publish ${index} for ${record.uri} moved version ${previous} -> ${record.version}`,
    );
    lastVersion.set(record.uri, record.version);
  });
  const versionless = records.filter((record) => record.version == null);
  assert.equal(
    versionless.length,
    closedUris.length,
    "expected exactly one versionless clear per closed document",
  );
  assert.deepEqual(versionless.map((record) => record.uri).sort(), [...closedUris].sort());
  for (const record of versionless) {
    assert.deepEqual(record.payload, [], `didClose clear for ${record.uri} must be empty`);
  }
  const firstVersionless = records.findIndex((record) => record.version == null);
  if (firstVersionless >= 0) {
    assert.ok(
      records.slice(firstVersionless).every((record) => record.version == null),
      "versionless didClose clears must terminate the stream",
    );
  }
}

/**
 * Audits the rapid-edit supersession window: the server may skip publishing
 * superseded intermediate versions entirely (cancellation), but whatever it
 * does publish must carry the payload matching that version's sent content,
 * at most once per version, and the window must end converged on the final
 * version's diagnostics with no other document publishing at all.
 */
export function assertCancellationWindow(
  window: PublishRecord[],
  uri: string,
  expectedByVersion: Map<number, string[]>,
  finalVersion: number,
): void {
  assert.ok(window.length >= 1, "the supersession window must publish at least the final version");
  const seen = new Set<number>();
  for (const record of window) {
    assert.equal(record.uri, uri, "no other document may publish during the supersession window");
    assert.ok(record.version != null, "supersession window publishes must be versioned");
    const expected = expectedByVersion.get(record.version);
    assert.ok(expected != null, `unexpected version ${record.version} in supersession window`);
    assert.ok(!seen.has(record.version), `version ${record.version} published twice`);
    seen.add(record.version);
    assert.deepEqual(
      record.payload,
      expected,
      `version ${record.version} published diagnostics not matching its content`,
    );
  }
  const last = window[window.length - 1];
  assert.equal(last.version, finalVersion, "the supersession window must end on the final version");
}

/** Exact hint fingerprint of the upstream dependency's own baseline publish. */
export function assertDependencyBaseline(diagnostics: LspDiagnostic[], label: string): void {
  assert.equal(diagnostics.length, 1, `${label}: ${JSON.stringify(diagnostics)}`);
  const [hint] = diagnostics;
  assert.equal(String(hint.code).replace(/^TS/, ""), "6133", label);
  assert.equal(hint.severity, 4, label);
  assert.equal(hint.source, "vize/types", label);
}
