import { Buffer } from "node:buffer";
import { createHash } from "node:crypto";
import { isAbsolute } from "node:path";
import { validateScriptScope, type ScriptScopeProject } from "./tool-matrix-inputs.ts";

type Rec = Readonly<Record<string, unknown>>;

export interface FileManifest {
  readonly fileCount: number;
  readonly files: string[];
  readonly sha256: string;
}

export interface TypecheckerCoverage {
  readonly schema: "vize.fixtureTypecheckerCoverage";
  readonly version: 2;
  readonly requested: FileManifest;
  readonly transitiveAuthored: FileManifest;
  readonly transitiveDependencies: FileManifest;
  readonly checked: FileManifest;
}

const outputKeys: readonly string[] = [
  "errorCount",
  "fileCount",
  "files",
  "programs",
  "warningCount",
];
const fileKeys: readonly string[] = ["diagnostics", "file"];
const programKeys: readonly string[] = ["compilerOptions", "files", "root", "tsconfig"];
const typecheckSourcePattern = /\.(?:vue|ts|tsx|mts|cts|js|jsx|mjs|cjs)$/;

export function validateTypecheckerOutput(
  project: ScriptScopeProject,
  output: unknown,
  exitCode: number | null,
  expectedFiles: readonly string[] | null = null,
  authoredFiles: readonly string[] | null = expectedFiles,
  scriptScope: unknown = null,
): TypecheckerCoverage {
  requireRecord(output, "envelope");
  if (scriptScope != null)
    validateScriptScope(project, output, expectedFiles, authoredFiles ?? [], scriptScope);
  const expectedOutputKeys = Array.isArray(output.programs)
    ? outputKeys
    : outputKeys.filter((key) => key !== "programs");
  requireExactKeys(output, expectedOutputKeys, "envelope");
  const [outputErrorCount, outputWarningCount, fileCount] = (
    ["errorCount", "warningCount", "fileCount"] as const
  ).map((field) => {
    const value = output[field];
    if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
      return invalid(`${field} must be a non-negative safe integer`);
    }
    return value;
  }) as [number, number, number];
  const files: unknown = output.files;
  if (!Array.isArray(files)) return invalid("files must be an array");
  const fileEntries = files as readonly unknown[];
  if (fileCount > fileEntries.length) {
    invalid(`fileCount ${fileCount} exceeds ${fileEntries.length} file entries`);
  }
  if (project.expectedVueFileCount === 0 && fileCount !== 0 && scriptScope == null) {
    invalid(`expected zero checked files, received ${fileCount}`);
  }
  if (project.expectedVueFileCount !== 0 && fileCount === 0) {
    invalid("non-empty fixture checked zero Vue files");
  }

  const seenFiles = new Set<string>();
  const entryPaths: string[] = [];
  let errorCount = 0;
  let warningCount = 0;
  for (const [index, file] of fileEntries.entries()) {
    requireRecord(file, `files[${index}]`);
    requireExactKeys(file, fileKeys, `files[${index}]`);
    const path = file.file;
    requireRelativePath(path, `files[${index}].file`);
    if (seenFiles.has(path)) invalid(`duplicate file entry: ${path}`);
    seenFiles.add(path);
    entryPaths.push(path);
    const diagnostics: unknown = file.diagnostics;
    if (!Array.isArray(diagnostics)) {
      return invalid(`files[${index}].diagnostics must be an array`);
    }
    if (index < fileCount && expectedFiles == null && !path.endsWith(".vue")) {
      invalid(`checked file is not a Vue SFC: ${path}`);
    }
    if (index < fileCount && !typecheckSourcePattern.test(path)) {
      invalid(`checked file has an unsupported typecheck extension: ${path}`);
    }
    if (index >= fileCount && diagnostics.length === 0) {
      invalid(`project-level file entry has no diagnostics: ${path}`);
    }
    for (const [diagnosticIndex, diagnostic] of (diagnostics as readonly unknown[]).entries()) {
      if (typeof diagnostic !== "string" || diagnostic.length === 0) {
        invalid(`files[${index}].diagnostics[${diagnosticIndex}] must be a non-empty string`);
      }
      if (diagnostic.startsWith("error:")) errorCount += 1;
      else if (diagnostic.startsWith("warning:")) warningCount += 1;
      else if (diagnostic.startsWith("hint:")) validateHintDiagnostic(diagnostic, path);
      else invalid(`diagnostic has no error or warning prefix: ${path}`);
    }
  }
  const programs: readonly unknown[] = Array.isArray(output.programs) ? output.programs : [];
  for (const [index, program] of programs.entries()) {
    requireRecord(program, `programs[${index}]`);
    const expectedKeys =
      typeof program.tsconfig === "string"
        ? programKeys
        : programKeys.filter((key) => key !== "compilerOptions" && key !== "tsconfig");
    requireExactKeys(program, expectedKeys, `programs[${index}]`);
    if (typeof program.root !== "string" || program.root.length === 0) {
      invalid(`programs[${index}].root must be a non-empty string`);
    }
    if ("tsconfig" in program && program.tsconfig === "") {
      invalid(`programs[${index}].tsconfig must be a non-empty string`);
    }
    if ("compilerOptions" in program) {
      requireRecord(program.compilerOptions, `programs[${index}].compilerOptions`);
    }
    const programFiles: unknown = program.files;
    if (!Array.isArray(programFiles)) return invalid(`programs[${index}].files must be an array`);
    for (const [fileIndex, file] of (programFiles as readonly unknown[]).entries()) {
      requireProgramPath(file, `programs[${index}].files[${fileIndex}]`);
    }
  }

  const checkedFiles = entryPaths.slice(0, fileCount);
  const sortedFiles = checkedFiles
    .map((file) => ({ file, buf: Buffer.from(file) }))
    .sort((left, right) => Buffer.compare(left.buf, right.buf))
    .map(({ file }) => file);
  if (JSON.stringify(checkedFiles) !== JSON.stringify(sortedFiles)) {
    invalid("checked file entries are not sorted");
  }
  let requested: readonly string[] = checkedFiles;
  let transitiveAuthoredFiles: string[] = [];
  let transitiveDependencyFiles: string[] = [];
  if (expectedFiles != null) {
    validateManifestInput(expectedFiles, "requested fixture inputs", isTypecheckSource);
    validateManifestInput(authoredFiles, "authored fixture sources", isTypecheckSource);
    const authored: readonly string[] = authoredFiles ?? [];
    const checkedSet = new Set(checkedFiles);
    const expectedSet = new Set(expectedFiles);
    const missing = expectedFiles.filter((file) => !checkedSet.has(file));
    if (missing.length > 0) {
      invalid(`checked files are missing requested fixture inputs: [${missing.join(", ")}]`);
    }
    const authoredSet = new Set(authored);
    const missingAuthoredInputs = expectedFiles.filter((file) => !authoredSet.has(file));
    if (missingAuthoredInputs.length > 0) {
      invalid(
        `requested fixture inputs are not authored sources: [${missingAuthoredInputs.join(", ")}]`,
      );
    }
    const transitiveFiles = checkedFiles.filter((file) => !expectedSet.has(file));
    transitiveAuthoredFiles = transitiveFiles.filter((file) => !isDependencySource(file));
    transitiveDependencyFiles = transitiveFiles.filter(isDependencySource);
    const unclassified = transitiveAuthoredFiles.filter((file) => !authoredSet.has(file));
    if (unclassified.length > 0) {
      invalid(
        `checked transitive files are not authored fixture sources: [${unclassified.join(", ")}]`,
      );
    }
    requested = expectedFiles;
  }
  if (outputErrorCount !== errorCount) {
    invalid(`errorCount ${outputErrorCount} does not match ${errorCount} diagnostics`);
  }
  if (outputWarningCount !== warningCount) {
    invalid(`warningCount ${outputWarningCount} does not match ${warningCount} diagnostics`);
  }
  const expectedExitCode = errorCount > 0 ? 1 : 0;
  if (exitCode !== expectedExitCode) {
    invalid(`exit code ${String(exitCode)} does not match expected ${expectedExitCode}`);
  }

  return {
    schema: "vize.fixtureTypecheckerCoverage",
    version: 2,
    requested: createManifest(requested),
    transitiveAuthored: createManifest(transitiveAuthoredFiles),
    transitiveDependencies: createManifest(transitiveDependencyFiles),
    checked: createManifest(checkedFiles),
  };
}

export function summarizeTypecheckerCoverage(coverage: unknown) {
  requireRecord(coverage, "typechecker coverage");
  requireExactKeys(
    coverage,
    ["checked", "requested", "schema", "transitiveAuthored", "transitiveDependencies", "version"],
    "typechecker coverage",
  );
  if (coverage.schema !== "vize.fixtureTypecheckerCoverage" || coverage.version !== 2) {
    invalid("typechecker coverage schema is unsupported");
  }
  const requested = validateManifest(
    coverage.requested,
    "typechecker coverage.requested",
    isTypecheckSource,
  );
  const transitiveAuthored = validateManifest(
    coverage.transitiveAuthored,
    "typechecker coverage.transitiveAuthored",
    (file) => isTypecheckSource(file) && !isDependencySource(file),
  );
  const transitiveDependencies = validateManifest(
    coverage.transitiveDependencies,
    "typechecker coverage.transitiveDependencies",
    (file) => isTypecheckSource(file) && isDependencySource(file),
  );
  const checked = validateManifest(
    coverage.checked,
    "typechecker coverage.checked",
    isTypecheckSource,
  );
  const combined = [
    ...requested.files,
    ...transitiveAuthored.files,
    ...transitiveDependencies.files,
  ].sort(byteSort);
  if (JSON.stringify(combined) !== JSON.stringify(checked.files)) {
    invalid("typechecker coverage classes do not partition checked files");
  }
  return {
    requestedFileCount: requested.fileCount,
    requestedSha256: requested.sha256,
    transitiveAuthoredFileCount: transitiveAuthored.fileCount,
    transitiveAuthoredSha256: transitiveAuthored.sha256,
    transitiveDependencyFileCount: transitiveDependencies.fileCount,
    transitiveDependencySha256: transitiveDependencies.sha256,
    checkedFileCount: checked.fileCount,
    checkedSha256: checked.sha256,
  };
}

function createManifest(files: readonly string[]): FileManifest {
  const digest = createHash("sha256");
  for (const file of files) digest.update(file).update("\0");
  return { fileCount: files.length, files: [...files], sha256: digest.digest("hex") };
}

function validateManifestInput(
  files: unknown,
  label: string,
  acceptsFile: (file: string) => boolean,
): asserts files is string[] {
  if (!Array.isArray(files)) return invalid(`${label} must be an array`);
  const seen = new Set<string>();
  for (const [index, file] of (files as readonly unknown[]).entries()) {
    requireRelativePath(file, `${label}[${index}]`);
    if (!acceptsFile(file)) invalid(`${label}[${index}] has an unsupported source extension`);
    if (seen.has(file)) invalid(`${label} contains duplicate file: ${file}`);
    seen.add(file);
  }
  if (JSON.stringify(files) !== JSON.stringify([...files].sort(byteSort))) {
    invalid(`${label} are not sorted`);
  }
}

function validateManifest(
  manifest: unknown,
  label: string,
  acceptsFile: (file: string) => boolean,
): FileManifest {
  requireRecord(manifest, label);
  requireExactKeys(manifest, ["fileCount", "files", "sha256"], label);
  const files = manifest.files;
  validateManifestInput(files, `${label}.files`, acceptsFile);
  if (manifest.fileCount !== files.length) invalid(`${label}.fileCount is inconsistent`);
  const expected = createManifest(files);
  if (manifest.sha256 !== expected.sha256) {
    invalid(`${label}.sha256 is inconsistent`);
  }
  return expected;
}

function byteSort(left: string, right: string): number {
  return Buffer.compare(Buffer.from(left), Buffer.from(right));
}

function isTypecheckSource(file: string): boolean {
  return typecheckSourcePattern.test(file);
}

function isDependencySource(file: string): boolean {
  return file.split("/").includes("node_modules");
}

function requireRecord(value: unknown, label: string): asserts value is Rec {
  if (value == null || typeof value !== "object" || Array.isArray(value)) {
    invalid(`${label} must be an object`);
  }
}

function requireExactKeys(value: Rec, expected: readonly string[], label: string): void {
  const actual = Object.keys(value).sort();
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    invalid(`${label} keys must be ${expected.join(", ")}; received ${actual.join(", ")}`);
  }
}

function requireRelativePath(value: unknown, label: string): asserts value is string {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    isAbsolute(value) ||
    /^[A-Za-z]:[\\/]/.test(value) ||
    value.includes("\\") ||
    value.startsWith("./") ||
    value.split("/").some((segment) => segment === "" || segment === "." || segment === "..")
  ) {
    invalid(`${label} must be a normalized relative path`);
  }
}

function requireProgramPath(value: unknown, label: string): void {
  if (typeof value !== "string" || value.length === 0 || /[\\\0]/.test(value)) {
    return invalid(`${label} must be a normalized program path`);
  }
  let components = value;
  if (/^[A-Za-z]:/.test(value)) {
    if (value[2] !== "/") invalid(`${label} must be a normalized program path`);
    components = value.slice(3);
  } else if (value.startsWith("//")) {
    components = value.slice(2);
    if (components.split("/").length < 2) invalid(`${label} must be a normalized program path`);
  } else if (value.startsWith("/")) {
    components = value.slice(1);
  }
  if (
    components.split("/").some((segment) => segment === "" || segment === "." || segment === "..")
  ) {
    invalid(`${label} must be a normalized program path`);
  }
}

function invalid(message: string): never {
  throw new Error(`invalid typechecker JSON output: ${message}`);
}

function validateHintDiagnostic(diagnostic: string, file: string): void {
  const match = /^hint:(\d+):(\d+) \[TS(\d+)\] ([\s\S]+)$/.exec(diagnostic);
  if (
    match == null ||
    match.slice(1, 4).some((field) => {
      const value = Number(field);
      return !Number.isSafeInteger(value) || value < 1;
    }) ||
    (match[4] ?? "").trim().length === 0
  ) {
    invalid(`hint must have positive safe location/code and non-empty message: ${file}`);
  }
}
