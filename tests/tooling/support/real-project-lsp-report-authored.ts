import { createHash } from "node:crypto";

import type {
  LspAuthoredOracle,
  LspProjectEvidence,
  LspResponseEvidence,
} from "./real-project-lsp-report.ts";

export function authoredAnchorEvidence(oracle: LspAuthoredOracle) {
  const anchors = [
    {
      anchor: oracle.templateBinding.usageAnchor,
      file: oracle.templateBinding.file,
      kind: "template-binding-usage",
      symbol: oracle.templateBinding.symbol,
    },
    {
      anchor: oracle.templateBinding.declarationAnchor,
      file: oracle.templateBinding.file,
      kind: "template-binding-declaration",
      symbol: oracle.templateBinding.symbol,
    },
    {
      anchor: oracle.componentBoundary.tagAnchor,
      file: oracle.componentBoundary.importerFile,
      kind: "component-boundary-tag",
      symbol: oracle.componentBoundary.tagName,
    },
    {
      anchor: oracle.componentBoundary.dependencyEdit.anchor,
      file: oracle.componentBoundary.componentFile,
      kind: "dependency-edit",
      symbol: oracle.componentBoundary.dependencyEdit.completionLabel,
    },
    {
      anchor: oracle.fileLifecycle.originalImportSpecifier,
      file: oracle.componentBoundary.importerFile,
      kind: "file-lifecycle-import",
      symbol: oracle.componentBoundary.tagName,
    },
    {
      anchor: oracle.fileLifecycle.markerInsertionAnchor,
      file: oracle.componentBoundary.componentFile,
      kind: "file-lifecycle-marker-insertion",
      symbol: oracle.fileLifecycle.markerSymbol,
    },
  ];
  return {
    count: anchors.length,
    sha256: createHash("sha256")
      .update(`${JSON.stringify(anchors)}\n`)
      .digest("hex"),
  };
}

export function hasCompleteAuthoredFeatureEvidence(project: LspProjectEvidence): boolean {
  const authored = project.authoredFeatures;
  if (authored == null) return false;
  return (
    authored.authoredAnchors != null &&
    authored.authoredAnchors.count > 0 &&
    isSha256(authored.authoredAnchors.sha256) &&
    authored.templateBindingFile.length > 0 &&
    authored.importerFile.length > 0 &&
    authored.componentFile.length > 0 &&
    hasPositiveResponse(authored.completion) &&
    hasPositiveResponse(authored.componentDefinition) &&
    hasPositiveResponse(authored.definition) &&
    hasPositiveResponse(authored.hover) &&
    hasPositiveResponse(authored.prepareRename) &&
    hasPositiveResponse(authored.references) &&
    hasPositiveResponse(authored.rename) &&
    authored.dependencyCompletion.baselineContainsProbe === false &&
    authored.dependencyCompletion.changedContainsProbe === true &&
    authored.dependencyCompletion.repairedContainsProbe === false &&
    hasPositiveResponse(authored.fileLifecycle.createdDefinition) &&
    hasPositiveResponse(authored.fileLifecycle.createdWorkspaceSymbols) &&
    hasPositiveResponse(authored.fileLifecycle.renameEdit) &&
    hasPositiveResponse(authored.fileLifecycle.renamedDefinition) &&
    hasPositiveResponse(authored.fileLifecycle.renamedWorkspaceSymbols) &&
    hasPositiveResponse(authored.fileLifecycle.restoredDefinition) &&
    hasZeroResponse(authored.fileLifecycle.deletedDefinition) &&
    hasZeroResponse(authored.fileLifecycle.deletedDocumentSymbols) &&
    hasZeroResponse(authored.fileLifecycle.deletedWorkspaceSymbols) &&
    hasZeroResponse(authored.fileLifecycle.staleCopiedDocumentSymbols)
  );
}

function hasPositiveResponse(evidence: LspResponseEvidence | undefined): boolean {
  return (
    evidence != null &&
    evidence.count > 0 &&
    Number.isFinite(evidence.durationMs) &&
    evidence.durationMs >= 0 &&
    isSha256(evidence.sha256)
  );
}

function hasZeroResponse(evidence: LspResponseEvidence | undefined): boolean {
  return (
    evidence != null &&
    evidence.count === 0 &&
    Number.isFinite(evidence.durationMs) &&
    evidence.durationMs >= 0 &&
    isSha256(evidence.sha256)
  );
}

function isSha256(value: string | undefined): boolean {
  return value != null && /^[0-9a-f]{64}$/.test(value);
}
