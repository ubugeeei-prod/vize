import assert from "node:assert/strict";
import fs from "node:fs";
import { setTimeout as sleep } from "node:timers/promises";
import { pathToFileURL } from "node:url";

import { offsetToPosition } from "./lsp/assertions.ts";
import type { LspRange, PublishDiagnosticsParams, WorkspaceEdit } from "./lsp/protocol.ts";
import {
  assertMissingModuleDiagnostic,
  assertRangeInDocument,
  diagnosticEvidence,
  diagnosticPayload,
  hasMissingModuleDiagnostic,
  locations,
  replaceUniqueAnchor,
  responseEvidence,
  sortTextEdits,
  textDocumentPosition,
  uniqueAnchorOffset,
  type OracleSession,
} from "./real-project-lsp-authored-utils.ts";
import type {
  AuthoredFileLifecycleEvidence,
  LspAuthoredOracle,
} from "./real-project-lsp-report.ts";
import {
  normalizeLifecycleRepairDiagnostics,
  reservedPath,
} from "./real-project-lsp-file-lifecycle-utils.ts";

type OracleDocument = { source: string; uri: string };
type SymbolInformation = { location?: { range?: LspRange; uri?: string }; name?: string };
type DocumentEdit = { newText: string; range: LspRange };
type DocumentChange = { edits?: DocumentEdit[]; textDocument?: { uri?: string } };

export async function exerciseAuthoredFileLifecycle(
  session: OracleSession,
  workspaceDir: string,
  oracle: LspAuthoredOracle,
  importer: OracleDocument,
  dependency: OracleDocument,
  tagRange: LspRange,
  baselineDiagnostics: PublishDiagnosticsParams,
  timeoutMs: () => number,
): Promise<AuthoredFileLifecycleEvidence> {
  const lifecycle = oracle.fileLifecycle;
  const copiedPath = reservedPath(workspaceDir, lifecycle.copiedFile, "copied");
  const renamedPath = reservedPath(workspaceDir, lifecycle.renamedFile, "renamed");
  assert.equal(fs.existsSync(copiedPath), false, `${lifecycle.copiedFile} already exists`);
  assert.equal(fs.existsSync(renamedPath), false, `${lifecycle.renamedFile} already exists`);
  uniqueAnchorOffset(
    importer.source,
    lifecycle.originalImportSpecifier,
    oracle.componentBoundary.importerFile,
  );
  const copiedImporterSource = replaceUniqueAnchor(
    importer.source,
    lifecycle.originalImportSpecifier,
    lifecycle.copiedImportSpecifier,
    oracle.componentBoundary.importerFile,
  );
  const renamedImporterSource = replaceUniqueAnchor(
    copiedImporterSource,
    lifecycle.copiedImportSpecifier,
    lifecycle.renamedImportSpecifier,
    oracle.componentBoundary.importerFile,
  );
  const copiedSource = replaceUniqueAnchor(
    dependency.source,
    lifecycle.markerInsertionAnchor,
    `${lifecycle.markerInsertionAnchor}const ${lifecycle.markerSymbol} = 1;\n`,
    oracle.componentBoundary.componentFile,
  );
  const copiedUri = pathToFileURL(copiedPath).href;
  const renamedUri = pathToFileURL(renamedPath).href;
  const initialVersion = baselineDiagnostics.version ?? 1;
  let currentVersion = initialVersion;
  let importerRepaired = false;

  try {
    fs.writeFileSync(copiedPath, copiedSource, { encoding: "utf8", flag: "wx" });
    session.notify("workspace/didCreateFiles", { files: [{ uri: copiedUri }] });
    const createdSymbols = await expectWorkspaceSymbol(
      session,
      lifecycle.markerSymbol,
      copiedUri,
      timeoutMs(),
    );

    currentVersion += 1;
    changeDocument(session, importer.uri, copiedImporterSource, currentVersion);
    await waitForDiagnostics(session, importer.uri, currentVersion, timeoutMs());
    const createdDefinition = await expectDefinition(
      session,
      importer.uri,
      tagRange,
      copiedUri,
      copiedSource,
      timeoutMs(),
    );

    const renameEdit = (await session.request(
      "workspace/willRenameFiles",
      { files: [{ newUri: renamedUri, oldUri: copiedUri }] },
      timeoutMs(),
    )) as WorkspaceEdit | null;
    assertRenameEdit(renameEdit, importer.uri, copiedImporterSource, lifecycle);
    fs.renameSync(copiedPath, renamedPath);
    session.notify("workspace/didRenameFiles", {
      files: [{ newUri: renamedUri, oldUri: copiedUri }],
    });

    currentVersion += 1;
    changeDocument(session, importer.uri, renamedImporterSource, currentVersion);
    await waitForDiagnostics(session, importer.uri, currentVersion, timeoutMs());
    const renamedDefinition = await expectDefinition(
      session,
      importer.uri,
      tagRange,
      renamedUri,
      copiedSource,
      timeoutMs(),
    );
    const renamedSymbols = await expectWorkspaceSymbol(
      session,
      lifecycle.markerSymbol,
      renamedUri,
      timeoutMs(),
    );
    const staleCopiedSymbols = await expectNullDocumentSymbols(session, copiedUri, timeoutMs());

    fs.rmSync(renamedPath);
    session.notify("workspace/didDeleteFiles", { files: [{ uri: renamedUri }] });
    const renamedImport = lifecycle.renamedImportSpecifier;
    const deletedDiagnostics = await waitForDiagnostics(
      session,
      importer.uri,
      currentVersion,
      timeoutMs(),
      (lifecycle.requireDeletedImportDiagnostic ?? true)
        ? (diagnostics) => hasMissingModuleDiagnostic(diagnostics, renamedImport)
        : undefined,
    );
    if (lifecycle.requireDeletedImportDiagnostic ?? true) {
      assertMissingModuleDiagnostic(deletedDiagnostics, renamedImporterSource, renamedImport);
    }
    const deletedDefinition = await expectNullResponse(
      session,
      "textDocument/definition",
      textDocumentPosition(importer.uri, tagRange.start),
      "deleted dependency definition must disappear",
      timeoutMs(),
    );
    const deletedWorkspaceSymbols = await expectNullResponse(
      session,
      "workspace/symbol",
      { query: lifecycle.markerSymbol },
      "deleted dependency workspace symbol must disappear",
      timeoutMs(),
    );
    const deletedDocumentSymbols = await expectNullDocumentSymbols(
      session,
      renamedUri,
      timeoutMs(),
    );

    currentVersion += 1;
    changeDocument(session, importer.uri, importer.source, currentVersion);
    const repaired = await waitForDiagnostics(session, importer.uri, currentVersion, timeoutMs());
    assert.deepEqual(
      normalizeLifecycleRepairDiagnostics(repaired, lifecycle),
      normalizeLifecycleRepairDiagnostics(baselineDiagnostics, lifecycle),
      "file lifecycle repair must restore owned importer diagnostics exactly",
    );
    const restoredDefinition = await expectDefinition(
      session,
      importer.uri,
      tagRange,
      dependency.uri,
      dependency.source,
      timeoutMs(),
    );
    importerRepaired = true;

    return {
      copiedFile: lifecycle.copiedFile,
      createdDefinition: responseEvidence(createdDefinition, 1, workspaceDir),
      createdWorkspaceSymbols: responseEvidence(createdSymbols, 1, workspaceDir),
      deletedDefinition: responseEvidence(deletedDefinition, 0, workspaceDir),
      deletedDocumentSymbols: responseEvidence(deletedDocumentSymbols, 0, workspaceDir),
      deletedImporterDiagnostics: diagnosticEvidence(deletedDiagnostics.diagnostics),
      deletedWorkspaceSymbols: responseEvidence(deletedWorkspaceSymbols, 0, workspaceDir),
      repairedDiagnostics: diagnosticEvidence(repaired.diagnostics),
      renameEdit: responseEvidence(renameEdit, 1, workspaceDir),
      renamedDefinition: responseEvidence(renamedDefinition, 1, workspaceDir),
      renamedFile: lifecycle.renamedFile,
      renamedWorkspaceSymbols: responseEvidence(renamedSymbols, 1, workspaceDir),
      restoredDefinition: responseEvidence(restoredDefinition, 1, workspaceDir),
      staleCopiedDocumentSymbols: responseEvidence(staleCopiedSymbols, 0, workspaceDir),
    };
  } finally {
    if (!importerRepaired && currentVersion > initialVersion) {
      try {
        changeDocument(session, importer.uri, importer.source, currentVersion + 1);
      } catch {
        // Preserve the primary lifecycle failure.
      }
    }
    for (const [file, uri] of [
      [copiedPath, copiedUri],
      [renamedPath, renamedUri],
    ] as const) {
      if (!fs.existsSync(file)) continue;
      fs.rmSync(file, { force: true });
      try {
        session.notify("workspace/didDeleteFiles", { files: [{ uri }] });
      } catch {
        // Preserve the primary lifecycle failure.
      }
    }
  }
}

async function expectDefinition(
  session: OracleSession,
  importerUri: string,
  tagRange: LspRange,
  expectedUri: string,
  expectedSource: string,
  timeoutMs: number,
): Promise<unknown> {
  const response = await session.request(
    "textDocument/definition",
    textDocumentPosition(importerUri, tagRange.start),
    timeoutMs,
  );
  const resolved = locations(response, `definition must resolve ${expectedUri}`);
  assert.equal(resolved.length, 1);
  assert.equal(resolved[0]?.uri, expectedUri);
  assertRangeInDocument(resolved[0]!.range, expectedSource, expectedUri);
  return response;
}

async function expectWorkspaceSymbol(
  session: OracleSession,
  marker: string,
  expectedUri: string,
  timeoutMs: number,
): Promise<unknown> {
  const response = await session.request("workspace/symbol", { query: marker }, timeoutMs);
  assert.ok(Array.isArray(response), `workspace symbol must resolve ${marker}`);
  const symbols = response as SymbolInformation[];
  assert.equal(symbols.length, 1, JSON.stringify(response));
  assert.equal(symbols[0]?.name, marker);
  assert.equal(symbols[0]?.location?.uri, expectedUri);
  return response;
}

async function expectNullDocumentSymbols(
  session: OracleSession,
  uri: string,
  timeoutMs: number,
): Promise<null> {
  return expectNullResponse(
    session,
    "textDocument/documentSymbol",
    { textDocument: { uri } },
    `stale document symbols must disappear for ${uri}`,
    timeoutMs,
  );
}

async function expectNullResponse(
  session: OracleSession,
  method: string,
  params: unknown,
  message: string,
  timeoutMs: number,
): Promise<null> {
  const deadline = Date.now() + timeoutMs;
  let response: unknown = undefined;
  do {
    const remainingMs = Math.max(1, deadline - Date.now());
    response = await session.request(method, params, Math.min(remainingMs, 5000));
    if (response === null) return null;
    await sleep(Math.min(100, Math.max(1, deadline - Date.now())));
  } while (Date.now() < deadline);
  assert.equal(response, null, message);
  return null;
}

function assertRenameEdit(
  edit: WorkspaceEdit | null,
  importerUri: string,
  importerSource: string,
  lifecycle: LspAuthoredOracle["fileLifecycle"],
): void {
  assert.ok(edit, "willRenameFiles must edit the authored importer");
  const documentChanges = (edit.documentChanges ?? []) as DocumentChange[];
  const edits = [
    ...(edit.changes?.[importerUri] ?? []),
    ...documentChanges.flatMap((change) =>
      change.textDocument?.uri === importerUri ? (change.edits ?? []) : [],
    ),
  ];
  const offset = uniqueAnchorOffset(
    importerSource,
    lifecycle.copiedImportSpecifier,
    "copied dependency import",
  );
  assert.deepEqual(
    sortTextEdits(edits),
    [
      {
        newText: lifecycle.renamedImportSpecifier,
        range: {
          start: offsetToPosition(importerSource, offset),
          end: offsetToPosition(importerSource, offset + lifecycle.copiedImportSpecifier.length),
        },
      },
    ],
    "willRenameFiles must rewrite exactly the copied dependency specifier",
  );
}

function changeDocument(session: OracleSession, uri: string, text: string, version: number): void {
  session.notify("textDocument/didChange", {
    contentChanges: [{ text }],
    textDocument: { uri, version },
  });
}

async function waitForDiagnostics(
  session: OracleSession,
  uri: string,
  version: number,
  timeoutMs: number,
  predicate?: (diagnostics: PublishDiagnosticsParams) => boolean,
): Promise<PublishDiagnosticsParams> {
  return (await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (value) => {
      const diagnostics = diagnosticPayload(value, uri, version);
      return diagnostics != null && (predicate == null || predicate(diagnostics));
    },
    timeoutMs,
  )) as PublishDiagnosticsParams;
}
