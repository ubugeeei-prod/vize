import { isAbsolute } from "node:path";

const fileKeys = ["errorCount", "file", "messages", "warningCount"];
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

export function validateLinterOutput(project, output, exitCode, expectedFiles = null) {
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

  const seenFiles = new Set();
  let totalErrors = 0;
  for (const [fileIndex, file] of output.entries()) {
    requireRecord(file, `files[${fileIndex}]`);
    requireExactKeys(file, fileKeys, `files[${fileIndex}]`);
    requireNormalizedPath(file.file, `files[${fileIndex}].file`);
    if (!file.file.endsWith(".vue")) invalid(`checked file is not a Vue SFC: ${file.file}`);
    if (seenFiles.has(file.file)) invalid(`duplicate file entry: ${file.file}`);
    seenFiles.add(file.file);
    for (const field of ["errorCount", "warningCount"]) {
      requireNonNegativeInteger(file[field], `files[${fileIndex}].${field}`);
    }
    if (!Array.isArray(file.messages)) invalid(`files[${fileIndex}].messages must be an array`);

    let errorCount = 0;
    let warningCount = 0;
    for (const [messageIndex, message] of file.messages.entries()) {
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
      for (const field of ["line", "column", "endLine", "endColumn"]) {
        requirePositiveInteger(message[field], `${label}.${field}`);
      }
      if (
        message.endLine < message.line ||
        (message.endLine === message.line && message.endColumn < message.column)
      ) {
        invalid(`${label} has an inverted source range`);
      }
    }
    if (file.errorCount !== errorCount) {
      invalid(`${file.file} errorCount ${file.errorCount} does not match ${errorCount} messages`);
    }
    if (file.warningCount !== warningCount) {
      invalid(
        `${file.file} warningCount ${file.warningCount} does not match ${warningCount} messages`,
      );
    }
    totalErrors += errorCount;
  }

  const checkedFiles = output.map((file) => file.file);
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

  const expectedExitCode = totalErrors > 0 ? 1 : 0;
  if (exitCode !== expectedExitCode) {
    invalid(`exit code ${exitCode} does not match expected ${expectedExitCode}`);
  }
}

function requireRecord(value, label) {
  if (value == null || typeof value !== "object" || Array.isArray(value)) {
    invalid(`${label} must be an object`);
  }
}

function requireExactKeys(value, expected, label) {
  const actual = Object.keys(value).sort((left, right) => left.localeCompare(right));
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    invalid(`${label} keys must be ${expected.join(", ")}; received ${actual.join(", ")}`);
  }
}

function requireNormalizedPath(value, label) {
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

function requireNonEmptyString(value, label) {
  if (typeof value !== "string" || value.length === 0) invalid(`${label} must be non-empty`);
}

function requireNonNegativeInteger(value, label) {
  if (!Number.isSafeInteger(value) || value < 0) {
    invalid(`${label} must be a non-negative safe integer`);
  }
}

function requirePositiveInteger(value, label) {
  if (!Number.isSafeInteger(value) || value < 1) {
    invalid(`${label} must be a positive safe integer`);
  }
}

function invalid(message) {
  throw new Error(`invalid linter JSON output: ${message}`);
}
