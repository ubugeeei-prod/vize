import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { resolveVizeLaunchCommand } from "./lsp/launch.ts";
import { hasCompleteAuthoredFeatureEvidence } from "./real-project-lsp-report-authored.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const reportName = "lsp-lifecycle-summary.json";

export type FixtureProject = {
  coverage: string[];
  expectedVueFileCount: number | null;
  fixturePath: string;
  id: string;
  lspAuthoredOracle?: LspAuthoredOracle;
  revision: string;
  vueGlobs: string[];
};

export type LspAuthoredOracle = {
  componentBoundary: {
    componentFile: string;
    completionItemCount: number;
    completionItems: Array<{ label: string; rank: number }>;
    dependencyEdit: {
      anchor: string;
      completionLabel: string;
      replacement: string;
    };
    importerFile: string;
    tagAnchor: string;
    tagName: string;
  };
  fileLifecycle: {
    copiedFile: string;
    copiedImportSpecifier: string;
    markerInsertionAnchor: string;
    markerSymbol: string;
    originalImportSpecifier: string;
    requireDeletedImportDiagnostic?: boolean;
    renamedFile: string;
    renamedImportSpecifier: string;
  };
  templateBinding: {
    declarationAnchor: string;
    file: string;
    hoverContains: string[];
    renameTo: string;
    sharesComponentImporter?: boolean;
    symbol: string;
    usageAnchor: string;
  };
};

export type DiagnosticEvidence = {
  count: number;
  sha256: string;
};

export type LspResponseEvidence = {
  count: number;
  durationMs: number;
  sha256: string;
};

export type AuthoredAnchorEvidence = {
  count: number;
  sha256: string;
};

export type LspReportEvidence = {
  commitSha: string;
  runtime: { name: "node"; version: string };
  vizeBinary: {
    command: string[];
    path: string | null;
    sha256: string | null;
  };
};

export type AuthoredFileLifecycleEvidence = {
  copiedFile: string;
  createdDefinition: LspResponseEvidence;
  createdWorkspaceSymbols: LspResponseEvidence;
  deletedDefinition: LspResponseEvidence;
  deletedDocumentSymbols: LspResponseEvidence;
  deletedImporterDiagnostics: DiagnosticEvidence;
  deletedWorkspaceSymbols: LspResponseEvidence;
  repairedDiagnostics: DiagnosticEvidence;
  renameEdit: LspResponseEvidence;
  renamedDefinition: LspResponseEvidence;
  renamedFile: string;
  renamedWorkspaceSymbols: LspResponseEvidence;
  restoredDefinition: LspResponseEvidence;
  staleCopiedDocumentSymbols: LspResponseEvidence;
};

export type AuthoredLspEvidence = {
  authoredAnchors: AuthoredAnchorEvidence;
  completion: LspResponseEvidence;
  componentDefinition: LspResponseEvidence;
  componentFile: string;
  definition: LspResponseEvidence;
  dependencyCompletion: {
    baselineContainsProbe: boolean;
    changedContainsProbe: boolean;
    repairedContainsProbe: boolean;
  };
  fileLifecycle: AuthoredFileLifecycleEvidence;
  hover: LspResponseEvidence;
  importerFile: string;
  prepareRename: LspResponseEvidence;
  references: LspResponseEvidence;
  rename: LspResponseEvidence;
  templateBindingFile: string;
};

export type AuthoredLspExerciseEvidence = AuthoredLspEvidence & {
  templateBindingDiagnostics: DiagnosticEvidence;
};

export type LspProjectEvidence = {
  actualFile: string | null;
  actualFileDiagnostics: DiagnosticEvidence | null;
  authoredFeatures: AuthoredLspEvidence | null;
  brokenProbeDiagnostics: DiagnosticEvidence | null;
  durationMs: number;
  failure?: string;
  fixedProbeDiagnostics: DiagnosticEvidence | null;
  id: string;
  repairedProbeDiagnostics: DiagnosticEvidence | null;
  revision: string;
  status: "failed" | "ok";
  vueFileCount: number;
};

export function selectProjects(projects: FixtureProject[], environment: NodeJS.ProcessEnv) {
  const shardIndex = environment.FIXTURE_SHARD_INDEX;
  const shardCount = environment.FIXTURE_SHARD_COUNT;
  if ((shardIndex == null) !== (shardCount == null)) {
    throw new Error("FIXTURE_SHARD_INDEX and FIXTURE_SHARD_COUNT must be set together");
  }
  if (shardIndex != null && shardCount != null) {
    const index = nonNegativeInteger(shardIndex, "FIXTURE_SHARD_INDEX");
    const count = positiveInteger(shardCount, "FIXTURE_SHARD_COUNT");
    assert.ok(index < count, "FIXTURE_SHARD_INDEX must be less than FIXTURE_SHARD_COUNT");
    const selected = projects.filter((_, projectIndex) => projectIndex % count === index);
    assert.ok(selected.length > 0, `fixture shard ${index}/${count} selected no projects`);
    return { enforced: true, projects: selected, shardCount: count, shardIndex: index };
  }
  return {
    enforced: false,
    projects: projects.filter((project) => isHydrated(path.resolve(root, project.fixturePath))),
    shardCount: null,
    shardIndex: null,
  };
}

export function createLspReport(
  selection: ReturnType<typeof selectProjects>,
  projects: LspProjectEvidence[],
  environment: NodeJS.ProcessEnv,
  dependencies: {
    resolveLaunchCommand?: (environment: NodeJS.ProcessEnv) => string[];
  } = {},
) {
  const evidence = createReportEvidence(environment, dependencies);
  const completeAuthoredProjects = projects.filter(hasCompleteAuthoredFeatureEvidence);
  return {
    schema: "vize.realProjectLspLifecycle",
    version: 5,
    commitSha: evidence.commitSha,
    evidence,
    shard: { count: selection.shardCount, index: selection.shardIndex },
    summary: {
      projectCount: projects.length,
      actualFileCount: projects.filter((project) => project.actualFile != null).length,
      authoredFeatureProjectCount: completeAuthoredProjects.length,
      authoredAnchorCount: completeAuthoredProjects.reduce(
        (total, project) => total + project.authoredFeatures!.authoredAnchors.count,
        0,
      ),
      missingAuthoredFeatureProjectIds: projects
        .filter((project) => !hasCompleteAuthoredFeatureEvidence(project))
        .map((project) => project.id),
      vueFileCount: projects.reduce((total, project) => total + project.vueFileCount, 0),
      failedProjectCount: projects.filter((project) => project.status === "failed").length,
    },
    projects,
  };
}

function createReportEvidence(
  environment: NodeJS.ProcessEnv,
  dependencies: {
    resolveLaunchCommand?: (environment: NodeJS.ProcessEnv) => string[];
  },
): LspReportEvidence {
  const command =
    dependencies.resolveLaunchCommand?.(environment) ??
    resolveVizeLaunchCommand(
      (candidate) =>
        spawnSync(candidate, ["--version"], {
          cwd: root,
          encoding: "utf8",
        }).status === 0,
      environment.VIZE_LSP_BIN,
    );
  const binaryPath = resolvedBinaryPath(command[0]);
  return {
    commitSha: resolveCommitSha(environment),
    runtime: { name: "node", version: process.versions.node },
    vizeBinary: {
      command,
      path: binaryPath == null ? null : displayBinaryPath(binaryPath),
      sha256: binaryPath == null ? null : sha256File(binaryPath),
    },
  };
}

function resolvedBinaryPath(command: string): string | null {
  if (command === "cargo") return null;
  const resolved = path.isAbsolute(command) ? command : path.resolve(root, command);
  return fs.existsSync(resolved) && fs.statSync(resolved).isFile() ? resolved : null;
}

function displayBinaryPath(binaryPath: string): string {
  const relative = path.relative(root, binaryPath);
  return relative.startsWith("..") ? binaryPath : relative;
}

function sha256File(binaryPath: string): string {
  return createHash("sha256").update(fs.readFileSync(binaryPath)).digest("hex");
}

export function writeLspReport(
  report: ReturnType<typeof createLspReport>,
  environment: NodeJS.ProcessEnv,
): void {
  const configured = environment.FIXTURE_REPORT_DIR;
  if (configured == null || configured.length === 0) return;
  const outputDir = path.resolve(root, configured);
  fs.mkdirSync(outputDir, { recursive: true });
  fs.writeFileSync(path.join(outputDir, reportName), `${JSON.stringify(report, null, 2)}\n`);
}

export function emptyLspEvidence(project: FixtureProject): LspProjectEvidence {
  return {
    actualFile: null,
    actualFileDiagnostics: null,
    authoredFeatures: null,
    brokenProbeDiagnostics: null,
    durationMs: 0,
    fixedProbeDiagnostics: null,
    id: project.id,
    repairedProbeDiagnostics: null,
    revision: project.revision,
    status: "failed",
    vueFileCount: 0,
  };
}

export function assertFixtureFileCount(project: FixtureProject, files: string[]): void {
  if (project.expectedVueFileCount != null) {
    assert.equal(
      files.length,
      project.expectedVueFileCount,
      `${project.id} matched ${files.length} Vue files, expected ${project.expectedVueFileCount}`,
    );
  }
  if (project.expectedVueFileCount !== 0) {
    assert.ok(files.length > 0, `${project.id} matched no files`);
  }
}

export function isHydrated(directory: string): boolean {
  return fs.existsSync(directory) && fs.readdirSync(directory).length > 0;
}

export function positiveInteger(value: string, name: string): number {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) throw new Error(`${name} must be positive`);
  return parsed;
}

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function nonNegativeInteger(value: string, name: string): number {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < 0) throw new Error(`${name} must be non-negative`);
  return parsed;
}

function resolveCommitSha(environment: NodeJS.ProcessEnv): string {
  const value = environment.GITHUB_SHA;
  if (value != null) return requireCommitSha(value, "GITHUB_SHA");
  const result = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
  assert.equal(result.status, 0, "git rev-parse HEAD must succeed");
  return requireCommitSha(result.stdout.trim(), "git rev-parse HEAD");
}

function requireCommitSha(value: string, source: string): string {
  assert.match(value, /^[0-9a-f]{40}$/, `${source} must be a full lowercase commit SHA`);
  return value;
}
