import assert from "node:assert/strict";
import path from "node:path";

import type { LspDiagnostic, PublishDiagnosticsParams } from "./lsp/protocol.ts";
import {
  diagnosticPayload,
  normalizeDiagnostics,
  type OracleSession,
} from "./real-project-lsp-authored-utils.ts";
import type { LspAuthoredOracle } from "./real-project-lsp-report.ts";

export function normalizeLifecycleRepairDiagnostics(
  published: PublishDiagnosticsParams,
  lifecycle: LspAuthoredOracle["fileLifecycle"],
): string[] {
  // Real-project backends can resolve unrelated package aliases while this
  // lifecycle exercises editor features. Keep lifecycle-owned imports exact.
  return normalizeDiagnostics(
    published.diagnostics.filter(
      (diagnostic) => !isUnownedMissingModuleDiagnostic(diagnostic, lifecycle),
    ),
  );
}

export function reservedPath(workspaceDir: string, relativeFile: string, label: string): string {
  assert.equal(path.isAbsolute(relativeFile), false, `${label} lifecycle file must be relative`);
  const absolute = path.resolve(workspaceDir, relativeFile);
  const relative = path.relative(workspaceDir, absolute);
  assert.ok(
    relative.length > 0 && relative !== ".." && !relative.startsWith(`..${path.sep}`),
    `${label} lifecycle file escapes the fixture: ${relativeFile}`,
  );
  return absolute;
}

function isUnownedMissingModuleDiagnostic(
  diagnostic: LspDiagnostic,
  lifecycle: LspAuthoredOracle["fileLifecycle"],
): boolean {
  if (String(diagnostic.code).replace(/^TS/, "") !== "2307") {
    return false;
  }
  const message = diagnostic.message ?? "";
  if (!message.startsWith("Cannot find module '")) {
    return false;
  }
  return ![
    lifecycle.originalImportSpecifier,
    lifecycle.copiedImportSpecifier,
    lifecycle.renamedImportSpecifier,
  ].some((specifier) => message.includes(specifier));
}

export async function waitForDiagnostics(
  session: OracleSession,
  uri: string,
  version: number,
  timeoutMs: number,
  predicate?: (diagnostics: PublishDiagnosticsParams) => boolean,
): Promise<PublishDiagnosticsParams> {
  const observed: Array<Array<{ code: unknown; message: string }>> = [];
  try {
    return (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (value) => {
        const diagnostics = diagnosticPayload(value, uri, version);
        if (diagnostics == null) return false;
        observed.push(diagnostics.diagnostics.map(({ code, message }) => ({ code, message })));
        return predicate == null || predicate(diagnostics);
      },
      timeoutMs,
    )) as PublishDiagnosticsParams;
  } catch (error) {
    const summary = JSON.stringify(observed.slice(-4));
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(`Diagnostics observed for ${uri} version ${version}: ${summary}\n${detail}`);
  }
}
