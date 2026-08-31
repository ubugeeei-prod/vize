import path from "node:path";

export const UI_SOURCE_INSTALL_PLAN_SCHEMA_VERSION = 1;

export type UiSourceInstallMode = "dry-run";
export type UiSourceInstallActionOperation = "create" | "overwrite" | "skip" | "conflict";
export type UiSourceInstallActionReason =
  | "destination-current"
  | "destination-different"
  | "destination-missing"
  | "duplicate-destination"
  | "invalid-path"
  | "path-traversal";
export type UiSourceInstallDiagnosticCode =
  | "duplicate-destination"
  | "invalid-path"
  | "path-traversal"
  | "unsupported-mode"
  | "unknown-family";

export interface UiSourceInstallKnownFile {
  readonly familyName: string;
  readonly sourcePath: string;
  readonly destinationPath: string;
  readonly sourceDigest: string;
  readonly destinationDigest?: string | null;
}

export interface UiSourceInstallPlanInput {
  readonly mode: UiSourceInstallMode;
  readonly requestedFamilies: readonly string[];
  readonly destinationRoot: string;
  readonly sourceFiles: readonly UiSourceInstallKnownFile[];
}

export interface UiSourceInstallAction {
  readonly operation: UiSourceInstallActionOperation;
  readonly reason: UiSourceInstallActionReason;
  readonly familyName: string;
  readonly sourcePath: string;
  readonly destinationPath: string;
  readonly targetPath: string | null;
  readonly sourceDigest: string;
  readonly destinationDigest: string | null;
}

export interface UiSourceInstallDiagnostic {
  readonly code: UiSourceInstallDiagnosticCode;
  readonly familyName: string | null;
  readonly path: string | null;
  readonly message: string;
}

export interface UiSourceInstallPlan {
  readonly schemaVersion: typeof UI_SOURCE_INSTALL_PLAN_SCHEMA_VERSION;
  readonly mode: UiSourceInstallMode;
  readonly ok: boolean;
  readonly destinationRoot: string;
  readonly requestedFamilies: readonly string[];
  readonly actions: readonly UiSourceInstallAction[];
  readonly diagnostics: readonly UiSourceInstallDiagnostic[];
}

interface NormalizedPath {
  readonly path: string | null;
  readonly code: "invalid-path" | "path-traversal" | null;
}

interface SelectedFile {
  readonly familyName: string;
  readonly sourcePath: string;
  readonly destinationPath: string;
  readonly targetPath: string | null;
  readonly sourceDigest: string;
  readonly destinationDigest: string | null;
  readonly diagnostics: readonly UiSourceInstallDiagnostic[];
}

const WINDOWS_ABSOLUTE_PATH = /^[A-Za-z]:\//u;

export function createUiSourceInstallDryRunPlan(
  input: UiSourceInstallPlanInput,
): UiSourceInstallPlan {
  const destinationRoot = normalizeDestinationRoot(input.destinationRoot);
  const requestedFamilies = sortedUnique(input.requestedFamilies.map(normalizeName));
  const diagnostics: UiSourceInstallDiagnostic[] = [];

  if ((input as { readonly mode?: unknown }).mode !== "dry-run") {
    diagnostics.push({
      code: "unsupported-mode",
      familyName: null,
      path: null,
      message: 'Unsupported UI source install mode; expected "dry-run"',
    });
    return createPlan(destinationRoot, requestedFamilies, [], diagnostics);
  }

  const knownFamilies = new Set(
    input.sourceFiles.map((file) => normalizeName(file.familyName)).filter(Boolean),
  );
  const unknownFamilies = requestedFamilies.filter((family) => !knownFamilies.has(family));
  if (unknownFamilies.length > 0) {
    diagnostics.push(...unknownFamilies.map(createUnknownFamilyDiagnostic));
    return createPlan(destinationRoot, requestedFamilies, [], diagnostics);
  }

  const selectedFiles = input.sourceFiles
    .map(normalizeKnownFile(destinationRoot))
    .filter((file) => requestedFamilies.includes(file.familyName));

  const destinationCounts = countDestinationPaths(selectedFiles);
  const actions = selectedFiles.map((file) =>
    createAction(file, destinationCounts.get(file.destinationPath) ?? 0),
  );

  diagnostics.push(...selectedFiles.flatMap((file) => file.diagnostics));
  diagnostics.push(...duplicateDestinationDiagnostics(selectedFiles, destinationCounts));

  return createPlan(destinationRoot, requestedFamilies, actions, diagnostics);
}

function createPlan(
  destinationRoot: string,
  requestedFamilies: readonly string[],
  actions: readonly UiSourceInstallAction[],
  diagnostics: readonly UiSourceInstallDiagnostic[],
): UiSourceInstallPlan {
  const sortedActions = [...actions].sort(compareActions);
  const sortedDiagnostics = [...diagnostics].sort(compareDiagnostics);

  return {
    schemaVersion: UI_SOURCE_INSTALL_PLAN_SCHEMA_VERSION,
    mode: "dry-run",
    ok:
      sortedDiagnostics.length === 0 &&
      sortedActions.every((action) => action.operation !== "conflict"),
    destinationRoot,
    requestedFamilies,
    actions: sortedActions,
    diagnostics: sortedDiagnostics,
  };
}

function normalizeName(value: string): string {
  return value.trim();
}

function sortedUnique(values: readonly string[]): readonly string[] {
  return [...new Set(values.filter((value) => value.length > 0))].sort(compareText);
}

function normalizeDestinationRoot(value: string): string {
  const normalized = value.trim().replaceAll("\\", "/");
  if (normalized.length === 0) return ".";

  const compact = path.posix.normalize(normalized);
  return compact.length > 1 && compact.endsWith("/") ? compact.slice(0, -1) : compact;
}

function normalizeRelativePath(value: string): NormalizedPath {
  const normalizedSeparators = value.trim().replaceAll("\\", "/");
  if (normalizedSeparators.length === 0 || normalizedSeparators.includes("\0")) {
    return { path: null, code: "invalid-path" };
  }

  if (
    normalizedSeparators.startsWith("/") ||
    normalizedSeparators.startsWith("//") ||
    WINDOWS_ABSOLUTE_PATH.test(normalizedSeparators)
  ) {
    return { path: null, code: "invalid-path" };
  }

  if (normalizedSeparators.split("/").includes("..")) {
    return { path: null, code: "path-traversal" };
  }

  const compact = path.posix.normalize(normalizedSeparators);
  if (compact === "." || compact.startsWith("../")) {
    return { path: null, code: "path-traversal" };
  }

  return { path: compact, code: null };
}

function joinDestinationPath(destinationRoot: string, destinationPath: string): string {
  if (destinationRoot === ".") return destinationPath;
  if (destinationRoot === "/") return `/${destinationPath}`;
  return `${destinationRoot}/${destinationPath}`;
}

function normalizeKnownFile(destinationRoot: string) {
  return (file: UiSourceInstallKnownFile): SelectedFile => {
    const familyName = normalizeName(file.familyName);
    const source = normalizeRelativePath(file.sourcePath);
    const destination = normalizeRelativePath(file.destinationPath);
    const diagnostics = [
      createPathDiagnostic("source", familyName, file.sourcePath, source.code),
      createPathDiagnostic("destination", familyName, file.destinationPath, destination.code),
    ].filter((diagnostic): diagnostic is UiSourceInstallDiagnostic => diagnostic != null);

    return {
      familyName,
      sourcePath: source.path ?? file.sourcePath,
      destinationPath: destination.path ?? file.destinationPath,
      targetPath:
        destination.path == null ? null : joinDestinationPath(destinationRoot, destination.path),
      sourceDigest: file.sourceDigest,
      destinationDigest: file.destinationDigest ?? null,
      diagnostics,
    };
  };
}

function countDestinationPaths(files: readonly SelectedFile[]): Map<string, number> {
  const counts = new Map<string, number>();
  for (const file of files) {
    if (file.targetPath == null) continue;
    counts.set(file.destinationPath, (counts.get(file.destinationPath) ?? 0) + 1);
  }
  return counts;
}

function createAction(file: SelectedFile, destinationCount: number): UiSourceInstallAction {
  const conflictReason = firstConflictReason(file, destinationCount);
  if (conflictReason != null) return action(file, "conflict", conflictReason);
  if (file.destinationDigest == null) return action(file, "create", "destination-missing");
  if (file.destinationDigest === file.sourceDigest)
    return action(file, "skip", "destination-current");
  return action(file, "overwrite", "destination-different");
}

function action(
  file: SelectedFile,
  operation: UiSourceInstallActionOperation,
  reason: UiSourceInstallActionReason,
): UiSourceInstallAction {
  return {
    operation,
    reason,
    familyName: file.familyName,
    sourcePath: file.sourcePath,
    destinationPath: file.destinationPath,
    targetPath: file.targetPath,
    sourceDigest: file.sourceDigest,
    destinationDigest: file.destinationDigest,
  };
}

function firstConflictReason(
  file: SelectedFile,
  destinationCount: number,
): UiSourceInstallActionReason | null {
  if (file.diagnostics.some((diagnostic) => diagnostic.code === "path-traversal")) {
    return "path-traversal";
  }
  if (file.diagnostics.length > 0) return "invalid-path";
  return destinationCount > 1 ? "duplicate-destination" : null;
}

function createUnknownFamilyDiagnostic(familyName: string): UiSourceInstallDiagnostic {
  return {
    code: "unknown-family",
    familyName,
    path: null,
    message: `Unknown UI source family "${familyName}"`,
  };
}

function createPathDiagnostic(
  label: "destination" | "source",
  familyName: string,
  filePath: string,
  code: "invalid-path" | "path-traversal" | null,
): UiSourceInstallDiagnostic | null {
  if (code == null) return null;

  const detail =
    code === "path-traversal"
      ? "path traversal segments are not allowed"
      : "paths must be non-empty relative file paths";
  return {
    code,
    familyName,
    path: filePath,
    message: `Rejecting ${label} path "${filePath}" for ${familyName}: ${detail}`,
  };
}

function duplicateDestinationDiagnostics(
  files: readonly SelectedFile[],
  counts: ReadonlyMap<string, number>,
): readonly UiSourceInstallDiagnostic[] {
  return files.flatMap((file): readonly UiSourceInstallDiagnostic[] => {
    if (file.targetPath == null || (counts.get(file.destinationPath) ?? 0) <= 1) return [];

    return [
      {
        code: "duplicate-destination",
        familyName: file.familyName,
        path: file.destinationPath,
        message: `Destination path "${file.destinationPath}" is planned more than once`,
      },
    ];
  });
}

function compareActions(left: UiSourceInstallAction, right: UiSourceInstallAction): number {
  return (
    compareNullable(left.targetPath, right.targetPath) ||
    compareText(left.destinationPath, right.destinationPath) ||
    compareText(left.sourcePath, right.sourcePath) ||
    compareText(left.familyName, right.familyName) ||
    compareText(left.operation, right.operation)
  );
}

function compareDiagnostics(
  left: UiSourceInstallDiagnostic,
  right: UiSourceInstallDiagnostic,
): number {
  return (
    compareNullable(left.path, right.path) ||
    compareNullable(left.familyName, right.familyName) ||
    compareText(left.code, right.code) ||
    compareText(left.message, right.message)
  );
}

function compareNullable(left: string | null, right: string | null): number {
  if (left === right) return 0;
  if (left === null) return 1;
  if (right === null) return -1;
  return compareText(left, right);
}

function compareText(left: string, right: string): number {
  if (left === right) return 0;
  return left < right ? -1 : 1;
}
