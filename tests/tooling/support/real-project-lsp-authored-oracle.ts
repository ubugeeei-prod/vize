import assert from "node:assert/strict";

import { completionLabels, hoverToText, offsetToPosition } from "./lsp/assertions.ts";
import type { LspRange, WorkspaceEdit } from "./lsp/protocol.ts";
import {
  anchoredSymbolRange,
  assertRankedLabels,
  assertRangeInDocument,
  diagnosticEvidence,
  locations,
  normalizeDiagnostics,
  readOracleDocument,
  replaceUniqueAnchor,
  responseEvidence,
  sortLocations,
  sortTextEdits,
  textDocumentPosition,
  timedRequest,
  uniqueAnchorOffset,
  type CompletionResponse,
  type Hover,
  type OracleSession,
} from "./real-project-lsp-authored-utils.ts";
import {
  change,
  open,
  requestCompletion,
  waitForDiagnostics,
} from "./real-project-lsp-authored-oracle-session.ts";
import type {
  AuthoredLspExerciseEvidence,
  FixtureProject,
  LspAuthoredOracle,
} from "./real-project-lsp-report.ts";
import { authoredAnchorEvidence } from "./real-project-lsp-report-authored.ts";
import { exerciseAuthoredFileLifecycle } from "./real-project-lsp-file-lifecycle.ts";

const diagnosticsTimeoutMs = 120_000;

export async function exerciseAuthoredLspOracle(
  session: OracleSession,
  workspaceDir: string,
  oracle: LspAuthoredOracle,
  timeoutMs: () => number = () => diagnosticsTimeoutMs,
): Promise<AuthoredLspExerciseEvidence> {
  const openUris: string[] = [];
  const binding = oracle.templateBinding;
  const boundary = oracle.componentBoundary;
  const bindingDocument = readOracleDocument(workspaceDir, binding.file);
  const childDocument = readOracleDocument(workspaceDir, boundary.componentFile);
  const importerDocument = readOracleDocument(workspaceDir, boundary.importerFile);
  const usageRange = anchoredSymbolRange(
    bindingDocument.source,
    binding.usageAnchor,
    binding.symbol,
    `${binding.file} template usage`,
  );
  const declarationRange = anchoredSymbolRange(
    bindingDocument.source,
    binding.declarationAnchor,
    binding.symbol,
    `${binding.file} declaration`,
  );
  const tagRange = anchoredSymbolRange(
    importerDocument.source,
    boundary.tagAnchor,
    boundary.tagName,
    `${boundary.importerFile} component tag`,
  );
  const completionPosition = offsetToPosition(
    importerDocument.source,
    uniqueAnchorOffset(importerDocument.source, boundary.tagAnchor, boundary.importerFile) +
      boundary.tagAnchor.length,
  );
  const changedChildSource = replaceUniqueAnchor(
    childDocument.source,
    boundary.dependencyEdit.anchor,
    boundary.dependencyEdit.replacement,
    boundary.componentFile,
  );

  try {
    open(session, bindingDocument.uri, bindingDocument.source, 1, openUris);
    const bindingDiagnostics = await waitForDiagnostics(
      session,
      bindingDocument.uri,
      1,
      timeoutMs(),
    );
    const hoverRequest = await timedRequest<Hover>(
      session,
      "textDocument/hover",
      textDocumentPosition(bindingDocument.uri, usageRange.start),
      timeoutMs(),
    );
    const hover = hoverRequest.response;
    assert.ok(hover, `hover must resolve the authored ${binding.symbol} binding`);
    assert.deepEqual(
      hover.range,
      usageRange,
      "hover range must stay on the authored template token",
    );
    const hoverText = hoverToText(hover);
    for (const expected of binding.hoverContains) {
      assert.ok(hoverText.includes(expected), `hover for ${binding.symbol} is missing ${expected}`);
    }

    const definitionRequest = await timedRequest<unknown>(
      session,
      "textDocument/definition",
      textDocumentPosition(bindingDocument.uri, usageRange.start),
      timeoutMs(),
    );
    const definition = locations(
      definitionRequest.response,
      `definition must resolve the authored ${binding.symbol} binding`,
    );
    const referencesRequest = await timedRequest<unknown>(
      session,
      "textDocument/references",
      {
        ...textDocumentPosition(bindingDocument.uri, usageRange.start),
        context: { includeDeclaration: true },
      },
      timeoutMs(),
    );
    const references = locations(
      referencesRequest.response,
      `references must resolve the authored ${binding.symbol} binding`,
    );
    assert.deepEqual(definition, [{ range: declarationRange, uri: bindingDocument.uri }]);
    assert.deepEqual(
      sortLocations(references),
      sortLocations([
        { range: usageRange, uri: bindingDocument.uri },
        { range: declarationRange, uri: bindingDocument.uri },
      ]),
      "references must cover exactly the authored template use and declaration",
    );

    const prepareRenameRequest = await timedRequest<LspRange | null>(
      session,
      "textDocument/prepareRename",
      textDocumentPosition(bindingDocument.uri, usageRange.start),
      timeoutMs(),
    );
    const prepareRename = prepareRenameRequest.response;
    assert.deepEqual(prepareRename, usageRange, "prepareRename must select the template token");
    const renameRequest = await timedRequest<WorkspaceEdit | null>(
      session,
      "textDocument/rename",
      {
        ...textDocumentPosition(bindingDocument.uri, usageRange.start),
        newName: binding.renameTo,
      },
      timeoutMs(),
    );
    const rename = renameRequest.response;
    const renameEdits = rename?.changes?.[bindingDocument.uri];
    assert.ok(renameEdits, `rename must edit the authored ${binding.symbol} binding`);
    assert.deepEqual(
      sortTextEdits(renameEdits),
      sortTextEdits([
        { newText: binding.renameTo, range: usageRange },
        { newText: binding.renameTo, range: declarationRange },
      ]),
      "rename must edit exactly the authored template use and declaration",
    );
    assert.deepEqual(Object.keys(rename?.changes ?? {}), [bindingDocument.uri]);

    open(session, childDocument.uri, childDocument.source, 1, openUris);
    const fixedChildDiagnostics = await waitForDiagnostics(
      session,
      childDocument.uri,
      1,
      timeoutMs(),
    );
    open(session, importerDocument.uri, importerDocument.source, 1, openUris);
    const initialImporterDiagnostics = await waitForDiagnostics(
      session,
      importerDocument.uri,
      1,
      timeoutMs(),
    );
    const componentDefinitionRequest = await timedRequest<unknown>(
      session,
      "textDocument/definition",
      textDocumentPosition(importerDocument.uri, tagRange.start),
      timeoutMs(),
    );
    const componentDefinition = locations(
      componentDefinitionRequest.response,
      `definition must cross the authored ${boundary.tagName} component boundary`,
    );
    assert.equal(componentDefinition.length, 1);
    assert.equal(componentDefinition[0]?.uri, childDocument.uri);
    assertRangeInDocument(componentDefinition[0]!.range, childDocument.source, boundary.tagName);

    const completionRequest = await timedRequest<CompletionResponse>(
      session,
      "textDocument/completion",
      textDocumentPosition(importerDocument.uri, completionPosition),
      timeoutMs(),
    );
    const baselineLabels = completionLabels(completionRequest.response);
    assert.equal(
      baselineLabels.length,
      boundary.completionItemCount,
      `${boundary.importerFile} completion set size drifted`,
    );
    assertRankedLabels(baselineLabels, boundary.completionItems, boundary.importerFile);
    const probe = boundary.dependencyEdit.completionLabel;
    const baselineContainsProbe = baselineLabels.includes(probe);
    assert.equal(baselineContainsProbe, false, "probe must start absent");

    const importerSettledVersion = (initialImporterDiagnostics.version ?? 1) + 1;
    change(session, importerDocument.uri, importerDocument.source, importerSettledVersion);
    await waitForDiagnostics(session, importerDocument.uri, importerSettledVersion, timeoutMs());

    // Component-boundary requests can settle the backend project view after
    // the child file's first diagnostics. Re-baseline the unchanged dependency
    // immediately before editing it so repair compares against that same view.
    const dependencyBaselineVersion = (fixedChildDiagnostics.version ?? 1) + 1;
    change(session, childDocument.uri, childDocument.source, dependencyBaselineVersion);
    const dependencyBaselineDiagnostics = await waitForDiagnostics(
      session,
      childDocument.uri,
      dependencyBaselineVersion,
      timeoutMs(),
    );

    const changedChildVersion = dependencyBaselineVersion + 1;
    change(session, childDocument.uri, changedChildSource, changedChildVersion);
    await waitForDiagnostics(session, childDocument.uri, changedChildVersion, timeoutMs());
    const changedLabels = await requestCompletion(
      session,
      importerDocument.uri,
      completionPosition,
      timeoutMs(),
    );
    const changedContainsProbe = changedLabels.includes(probe);
    assert.ok(changedContainsProbe, "completion must observe an unsaved dependency edit");

    const repairedChildVersion = changedChildVersion + 1;
    change(session, childDocument.uri, childDocument.source, repairedChildVersion);
    const repaired = await waitForDiagnostics(
      session,
      childDocument.uri,
      repairedChildVersion,
      timeoutMs(),
    );
    assert.deepEqual(
      normalizeDiagnostics(repaired.diagnostics),
      normalizeDiagnostics(dependencyBaselineDiagnostics.diagnostics),
      "dependency repair must restore diagnostics exactly",
    );
    const repairedLabels = await requestCompletion(
      session,
      importerDocument.uri,
      completionPosition,
      timeoutMs(),
    );
    assert.deepEqual(repairedLabels, baselineLabels, "dependency repair must restore completions");
    const repairedContainsProbe = repairedLabels.includes(probe);
    assert.equal(
      repairedContainsProbe,
      false,
      "dependency repair must remove the probe completion",
    );
    // Type diagnostics can arrive after the importer opened with an initial
    // lint-only publish. Recompute the unchanged importer immediately before
    // the create/rename/delete lifecycle so the final repair is compared
    // against the server's settled baseline for that exact source.
    const lifecycleImporterVersion = importerSettledVersion + 1;
    change(session, importerDocument.uri, importerDocument.source, lifecycleImporterVersion);
    const lifecycleImporterDiagnostics = await waitForDiagnostics(
      session,
      importerDocument.uri,
      lifecycleImporterVersion,
      timeoutMs(),
    );
    const fileLifecycle = await exerciseAuthoredFileLifecycle(
      session,
      workspaceDir,
      oracle,
      importerDocument,
      childDocument,
      tagRange,
      lifecycleImporterDiagnostics,
      timeoutMs,
    );
    const evidence = (request: { durationMs: number }, response: unknown, count: number) =>
      responseEvidence(response, count, workspaceDir, request.durationMs);

    return {
      authoredAnchors: authoredAnchorEvidence(oracle),
      completion: evidence(completionRequest, baselineLabels, baselineLabels.length),
      componentDefinition: evidence(componentDefinitionRequest, componentDefinition, 1),
      componentFile: boundary.componentFile,
      definition: evidence(definitionRequest, definition, definition.length),
      dependencyCompletion: {
        baselineContainsProbe,
        changedContainsProbe,
        repairedContainsProbe,
      },
      fileLifecycle,
      hover: evidence(hoverRequest, hover, 1),
      importerFile: boundary.importerFile,
      prepareRename: evidence(prepareRenameRequest, prepareRename, 1),
      references: evidence(referencesRequest, sortLocations(references), references.length),
      rename: evidence(renameRequest, sortTextEdits(renameEdits), renameEdits.length),
      templateBindingDiagnostics: diagnosticEvidence(bindingDiagnostics.diagnostics),
      templateBindingFile: binding.file,
    };
  } finally {
    for (const uri of openUris.reverse()) {
      try {
        session.notify("textDocument/didClose", { textDocument: { uri } });
      } catch {
        // Preserve the primary oracle failure instead of the cleanup error.
      }
    }
  }
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
