import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";

import type { LspRange } from "./lsp/protocol.ts";
import {
  assertRangeInDocument,
  locations,
  replaceUniqueAnchor,
  textDocumentPosition,
  type OracleSession,
} from "./real-project-lsp-authored-utils.ts";
import {
  change,
  requestCompletion,
  waitForDiagnostics,
} from "./real-project-lsp-authored-oracle-session.ts";
import { reservedPath } from "./real-project-lsp-file-lifecycle-utils.ts";
import type { LspAuthoredOracle } from "./real-project-lsp-report.ts";

type Document = { source: string; uri: string };
type Overlay = {
  importer: Document;
  dependency: Document;
  changedDependencySource: string;
  tagRange: LspRange;
  completionPosition: LspRange["start"];
  baselineLabels: string[];
  changedLabels: string[];
  initialVersion: number;
};

/** Retarget the importer buffer while its original dependency remains unsaved. */
export async function exerciseAuthoredImporterOverlay(
  session: OracleSession,
  workspaceDir: string,
  oracle: LspAuthoredOracle,
  overlay: Overlay,
  timeoutMs: () => number,
): Promise<number> {
  const { importer, dependency, tagRange, completionPosition } = overlay;
  const lifecycle = oracle.fileLifecycle;
  const copiedPath = reservedPath(workspaceDir, lifecycle.copiedFile, "importer overlay");
  assert.equal(fs.existsSync(copiedPath), false, `${lifecycle.copiedFile} already exists`);
  const copiedUri = pathToFileURL(copiedPath).href;
  const copiedImporter = replaceUniqueAnchor(
    importer.source,
    lifecycle.originalImportSpecifier,
    lifecycle.copiedImportSpecifier,
    oracle.componentBoundary.importerFile,
  );
  const assertUnsaved = () => {
    for (const document of [importer, dependency])
      assert.equal(
        fs.readFileSync(fileURLToPath(document.uri), "utf8"),
        document.source,
        "importer and dependency overlay edits must leave authored disk bytes unchanged",
      );
  };
  assertUnsaved();
  let version = overlay.initialVersion;
  let copied = false;
  let retargeted = false;
  let failed = false;

  try {
    fs.writeFileSync(copiedPath, dependency.source, { encoding: "utf8", flag: "wx" });
    copied = true;
    session.notify("workspace/didCreateFiles", { files: [{ uri: copiedUri }] });
    retargeted = true;
    change(session, importer.uri, copiedImporter, ++version);
    await waitForDiagnostics(session, importer.uri, version, timeoutMs());
    await expectDefinition(
      session,
      importer.uri,
      tagRange,
      copiedUri,
      dependency.source,
      timeoutMs(),
    );
    const copiedLabels = await requestCompletion(
      session,
      importer.uri,
      completionPosition,
      timeoutMs(),
    );
    assert.deepEqual(
      copiedLabels,
      overlay.baselineLabels,
      "unsaved importer retarget must observe the unchanged copy's complete completion bank",
    );
    assertUnsaved();

    change(session, importer.uri, importer.source, ++version);
    await waitForDiagnostics(session, importer.uri, version, timeoutMs());
    await expectDefinition(
      session,
      importer.uri,
      tagRange,
      dependency.uri,
      overlay.changedDependencySource,
      timeoutMs(),
    );
    const restoredLabels = await requestCompletion(
      session,
      importer.uri,
      completionPosition,
      timeoutMs(),
    );
    assert.deepEqual(
      restoredLabels,
      overlay.changedLabels,
      "importer repair must restore the dirty dependency's complete completion bank",
    );
    assertUnsaved();
    retargeted = false;
    return version;
  } catch (error) {
    failed = true;
    throw error;
  } finally {
    let cleanupError: unknown;
    if (retargeted) {
      try {
        change(session, importer.uri, importer.source, ++version);
        await waitForDiagnostics(session, importer.uri, version, timeoutMs());
      } catch (error) {
        cleanupError = error;
      }
    }
    if (copied) {
      try {
        fs.rmSync(copiedPath);
        session.notify("workspace/didDeleteFiles", { files: [{ uri: copiedUri }] });
      } catch (error) {
        cleanupError ??= error;
      }
    }
    if (!failed && cleanupError != null) throw cleanupError;
  }
}

async function expectDefinition(
  session: OracleSession,
  importerUri: string,
  tagRange: LspRange,
  expectedUri: string,
  expectedSource: string,
  timeoutMs: number,
) {
  const response = await session.request(
    "textDocument/definition",
    textDocumentPosition(importerUri, tagRange.start),
    timeoutMs,
  );
  const resolved = locations(response, `importer overlay definition must resolve ${expectedUri}`);
  assert.equal(resolved.length, 1);
  assert.equal(resolved[0]?.uri, expectedUri);
  assertRangeInDocument(resolved[0]!.range, expectedSource, expectedUri);
}
