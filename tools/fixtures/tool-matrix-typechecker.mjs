import { Buffer } from "node:buffer";
import { isAbsolute } from "node:path";

const outputKeys = ["errorCount", "fileCount", "files", "warningCount"];
const fileKeys = ["diagnostics", "file"];

export function validateTypecheckerOutput(project, output, exitCode, expectedFiles = null) {
  requireRecord(output, "envelope");
  requireExactKeys(output, outputKeys, "envelope");
  for (const field of ["errorCount", "warningCount", "fileCount"]) {
    if (!Number.isSafeInteger(output[field]) || output[field] < 0) {
      invalid(`${field} must be a non-negative safe integer`);
    }
  }
  if (!Array.isArray(output.files)) invalid("files must be an array");
  if (output.fileCount > output.files.length) {
    invalid(`fileCount ${output.fileCount} exceeds ${output.files.length} file entries`);
  }
  if (project.expectedVueFileCount === 0 && output.fileCount !== 0) {
    invalid(`expected zero checked files, received ${output.fileCount}`);
  }
  if (project.expectedVueFileCount !== 0 && output.fileCount === 0) {
    invalid("non-empty fixture checked zero Vue files");
  }
  if (expectedFiles != null && output.fileCount !== expectedFiles.length) {
    invalid(
      `checked file count ${output.fileCount} does not match ${expectedFiles.length} fixture inputs`,
    );
  }

  const seenFiles = new Set();
  let errorCount = 0;
  let warningCount = 0;
  for (const [index, file] of output.files.entries()) {
    requireRecord(file, `files[${index}]`);
    requireExactKeys(file, fileKeys, `files[${index}]`);
    requireRelativePath(file.file, `files[${index}].file`);
    if (seenFiles.has(file.file)) invalid(`duplicate file entry: ${file.file}`);
    seenFiles.add(file.file);
    if (!Array.isArray(file.diagnostics)) {
      invalid(`files[${index}].diagnostics must be an array`);
    }
    if (index < output.fileCount && !file.file.endsWith(".vue")) {
      invalid(`checked file is not a Vue SFC: ${file.file}`);
    }
    if (index >= output.fileCount && file.diagnostics.length === 0) {
      invalid(`project-level file entry has no diagnostics: ${file.file}`);
    }
    for (const [diagnosticIndex, diagnostic] of file.diagnostics.entries()) {
      if (typeof diagnostic !== "string" || diagnostic.length === 0) {
        invalid(`files[${index}].diagnostics[${diagnosticIndex}] must be a non-empty string`);
      }
      if (diagnostic.startsWith("error:")) errorCount += 1;
      else if (diagnostic.startsWith("warning:")) warningCount += 1;
      else invalid(`diagnostic has no error or warning prefix: ${file.file}`);
    }
  }

  const checkedFiles = output.files.slice(0, output.fileCount).map((file) => file.file);
  const sortedFiles = checkedFiles
    .map((file) => ({ file, buf: Buffer.from(file) }))
    .sort((left, right) => Buffer.compare(left.buf, right.buf))
    .map(({ file }) => file);
  if (JSON.stringify(checkedFiles) !== JSON.stringify(sortedFiles)) {
    invalid("checked file entries are not sorted");
  }
  if (expectedFiles != null && JSON.stringify(checkedFiles) !== JSON.stringify(expectedFiles)) {
    const checkedSet = new Set(checkedFiles);
    const expectedSet = new Set(expectedFiles);
    const missing = expectedFiles.filter((file) => !checkedSet.has(file));
    const unexpected = checkedFiles.filter((file) => !expectedSet.has(file));
    invalid(
      `checked files do not match fixture inputs: missing [${missing.join(", ")}], unexpected [${unexpected.join(", ")}]`,
    );
  }
  if (output.errorCount !== errorCount) {
    invalid(`errorCount ${output.errorCount} does not match ${errorCount} diagnostics`);
  }
  if (output.warningCount !== warningCount) {
    invalid(`warningCount ${output.warningCount} does not match ${warningCount} diagnostics`);
  }
  const expectedExitCode = errorCount > 0 ? 1 : 0;
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
  const actual = Object.keys(value).sort();
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    invalid(`${label} keys must be ${expected.join(", ")}; received ${actual.join(", ")}`);
  }
}

function requireRelativePath(value, label) {
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

function invalid(message) {
  throw new Error(`invalid typechecker JSON output: ${message}`);
}
