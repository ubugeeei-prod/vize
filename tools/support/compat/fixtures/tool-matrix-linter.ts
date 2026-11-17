import { isAbsolute } from "node:path";

const fileKeys: readonly string[] = ["errorCount", "file", "messages", "warningCount"];
const messageKeys = [
  "column",
  "endColumn",
  "endLine",
  "line",
  "message",
  "ruleDocsPath",
  "ruleId",
  "severity",
];
const messageKeysWithHelp = [
  "column",
  "endColumn",
  "endLine",
  "help",
  "line",
  "message",
  "ruleDocsPath",
  "ruleId",
  "severity",
];

type Rec = Readonly<Record<string, unknown>>;

const lintExtensions = ".vue, .html, .htm, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, .tsx, or .tsrx";

export function zeroFileLinterStderr(patterns: readonly string[]): string {
  return `No ${lintExtensions} files found matching patterns: [${patterns
    .map((pattern) => JSON.stringify(pattern))
    .join(", ")}]\n`;
}

export function validateLinterOutput(
  project: { readonly expectedVueFileCount?: number },
  output: unknown,
  exitCode: number | null,
  expectedFiles: readonly string[] | null = null,
): void {
  if (!Array.isArray(output)) invalid("envelope must be an array");
  if (project.expectedVueFileCount === 0 && output.length !== 0) {
    invalid(`expected zero checked files, received ${output.length}`);
  }
  if (project.expectedVueFileCount !== 0 && output.length === 0) {
    invalid("non-empty fixture linted zero Vue files");
  }
  if (expectedFiles != null && output.length !== expectedFiles.length) {
    invalid(`checked file count ${output.length} does not match ${expectedFiles.length} inputs`);
  }

  const seenFiles = new Set<string>();
  const checkedFiles: string[] = [];
  let totalErrors = 0;
  for (const [fileIndex, file] of output.entries()) {
    requireRecord(file, `files[${fileIndex}]`);
    requireExactKeys(file, fileKeys, `files[${fileIndex}]`);
    const path = file.file;
    requireNormalizedPath(path, `files[${fileIndex}].file`);
    checkedFiles.push(path);
    if (!path.endsWith(".vue")) invalid(`checked file is not a Vue SFC: ${path}`);
    if (seenFiles.has(path)) invalid(`duplicate file entry: ${path}`);
    seenFiles.add(path);
    for (const field of ["errorCount", "warningCount"]) {
      requireNonNegativeInteger(file[field], `files[${fileIndex}].${field}`);
    }
    const messages: unknown = file.messages;
    if (!Array.isArray(messages)) return invalid(`files[${fileIndex}].messages must be an array`);

    let errorCount = 0;
    let warningCount = 0;
    for (const [messageIndex, message] of (messages as readonly unknown[]).entries()) {
      const label = `files[${fileIndex}].messages[${messageIndex}]`;
      requireRecord(message, label);
      const expectedKeys = "help" in message ? messageKeysWithHelp : messageKeys;
      requireExactKeys(message, expectedKeys, label);
      for (const field of ["ruleId", "ruleDocsPath", "message"]) {
        requireNonEmptyString(message[field], `${label}.${field}`);
      }
      requireNormalizedPath(message.ruleDocsPath, `${label}.ruleDocsPath`);
      if ("help" in message) requireNonEmptyString(message.help, `${label}.help`);
      if (message.severity === 2) errorCount += 1;
      else if (message.severity === 1) warningCount += 1;
      else invalid(`${label}.severity must be 1 or 2`);
      const [line, column, endLine, endColumn] = (
        ["line", "column", "endLine", "endColumn"] as const
      ).map((field) => requirePositiveInteger(message[field], `${label}.${field}`)) as [
        number,
        number,
        number,
        number,
      ];
      if (endLine < line || (endLine === line && endColumn < column)) {
        invalid(`${label} has an inverted source range`);
      }
    }
    if (file.errorCount !== errorCount) {
      invalid(
        `${path} errorCount ${String(file.errorCount)} does not match ${errorCount} messages`,
      );
    }
    if (file.warningCount !== warningCount) {
      invalid(
        `${path} warningCount ${String(file.warningCount)} does not match ${warningCount} messages`,
      );
    }
    totalErrors += errorCount;
  }

  if (expectedFiles != null) {
    const checkedSet = new Set(checkedFiles);
    const expectedSet = new Set(expectedFiles);
    const missing = expectedFiles.filter((file) => !checkedSet.has(file));
    const unexpected = checkedFiles.filter((file) => !expectedSet.has(file));
    if (missing.length > 0 || unexpected.length > 0) {
      invalid(
        `checked files do not match inputs: missing [${missing.join(", ")}], unexpected [${unexpected.join(", ")}]`,
      );
    }
  }

  const expectedExitCode = project.expectedVueFileCount === 0 ? 2 : totalErrors > 0 ? 1 : 0;
  if (exitCode !== expectedExitCode) {
    invalid(`exit code ${String(exitCode)} does not match expected ${expectedExitCode}`);
  }
}

function requireRecord(value: unknown, label: string): asserts value is Rec {
  if (value == null || typeof value !== "object" || Array.isArray(value)) {
    invalid(`${label} must be an object`);
  }
}

function requireExactKeys(value: Rec, expected: readonly string[], label: string): void {
  const actual = Object.keys(value).sort((left, right) => left.localeCompare(right));
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    invalid(`${label} keys must be ${expected.join(", ")}; received ${actual.join(", ")}`);
  }
}

function requireNormalizedPath(value: unknown, label: string): asserts value is string {
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

function requireNonEmptyString(value: unknown, label: string): void {
  if (typeof value !== "string" || value.length === 0) invalid(`${label} must be non-empty`);
}

function requireNonNegativeInteger(value: unknown, label: string): void {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    invalid(`${label} must be a non-negative safe integer`);
  }
}

function requirePositiveInteger(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 1) {
    return invalid(`${label} must be a positive safe integer`);
  }
  return value;
}

function invalid(message: string): never {
  throw new Error(`invalid linter JSON output: ${message}`);
}
