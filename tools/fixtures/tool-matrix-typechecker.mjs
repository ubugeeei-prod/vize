import { Buffer } from "node:buffer";
import { createHash } from "node:crypto";
import { isAbsolute } from "node:path";

const outputKeys = ["errorCount", "fileCount", "files", "programs", "warningCount"];
const fileKeys = ["diagnostics", "file"];
const programKeys = ["files", "root", "tsconfig"];
const typecheckSourcePattern = /\.(?:vue|ts|tsx|mts|cts|js|jsx|mjs|cjs)$/;

export function validateTypecheckerOutput(
  project,
  output,
  exitCode,
  expectedFiles = null,
  authoredFiles = expectedFiles,
) {
  requireRecord(output, "envelope");
  requireExactKeys(output, outputKeys, "envelope");
  for (const field of ["errorCount", "warningCount", "fileCount"]) {
    if (!Number.isSafeInteger(output[field]) || output[field] < 0) {
      invalid(`${field} must be a non-negative safe integer`);
    }
  }
  if (!Array.isArray(output.files)) invalid("files must be an array");
  if (!Array.isArray(output.programs)) invalid("programs must be an array");
  if (output.fileCount > output.files.length) {
    invalid(`fileCount ${output.fileCount} exceeds ${output.files.length} file entries`);
  }
  if (project.expectedVueFileCount === 0 && output.fileCount !== 0) {
    invalid(`expected zero checked files, received ${output.fileCount}`);
  }
  if (project.expectedVueFileCount !== 0 && output.fileCount === 0) {
    invalid("non-empty fixture checked zero Vue files");
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
    if (index < output.fileCount && expectedFiles == null && !file.file.endsWith(".vue")) {
      invalid(`checked file is not a Vue SFC: ${file.file}`);
    }
    if (index < output.fileCount && !typecheckSourcePattern.test(file.file)) {
      invalid(`checked file has an unsupported typecheck extension: ${file.file}`);
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
  for (const [index, program] of output.programs.entries()) {
    requireRecord(program, `programs[${index}]`);
    const expectedKeys =
      typeof program.tsconfig === "string"
        ? programKeys
        : programKeys.filter((key) => key !== "tsconfig");
    requireExactKeys(program, expectedKeys, `programs[${index}]`);
    if (typeof program.root !== "string" || program.root.length === 0) {
      invalid(`programs[${index}].root must be a non-empty string`);
    }
    if ("tsconfig" in program && program.tsconfig.length === 0) {
      invalid(`programs[${index}].tsconfig must be a non-empty string`);
    }
    if (!Array.isArray(program.files)) invalid(`programs[${index}].files must be an array`);
    for (const [fileIndex, file] of program.files.entries()) {
      requireRelativePath(file, `programs[${index}].files[${fileIndex}]`);
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
  let requestedFiles = checkedFiles;
  let transitiveAuthoredFiles = [];
  if (expectedFiles != null) {
    validateManifestInput(expectedFiles, "requested fixture inputs", isVueSfc);
    validateManifestInput(authoredFiles, "authored fixture sources", isTypecheckSource);
    const checkedSet = new Set(checkedFiles);
    const expectedSet = new Set(expectedFiles);
    const missing = expectedFiles.filter((file) => !checkedSet.has(file));
    if (missing.length > 0) {
      invalid(`checked files are missing requested fixture inputs: [${missing.join(", ")}]`);
    }
    const authoredSet = new Set(authoredFiles);
    const missingAuthoredInputs = expectedFiles.filter((file) => !authoredSet.has(file));
    if (missingAuthoredInputs.length > 0) {
      invalid(
        `requested fixture inputs are not authored sources: [${missingAuthoredInputs.join(", ")}]`,
      );
    }
    transitiveAuthoredFiles = checkedFiles.filter((file) => !expectedSet.has(file));
    const unclassified = transitiveAuthoredFiles.filter((file) => !authoredSet.has(file));
    if (unclassified.length > 0) {
      invalid(
        `checked transitive files are not authored fixture sources: [${unclassified.join(", ")}]`,
      );
    }
    requestedFiles = expectedFiles;
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

  return {
    schema: "vize.fixtureTypecheckerCoverage",
    version: 1,
    requested: createManifest(requestedFiles),
    transitiveAuthored: createManifest(transitiveAuthoredFiles),
    checked: createManifest(checkedFiles),
  };
}

export function summarizeTypecheckerCoverage(coverage) {
  requireRecord(coverage, "typechecker coverage");
  requireExactKeys(
    coverage,
    ["checked", "requested", "schema", "transitiveAuthored", "version"],
    "typechecker coverage",
  );
  if (coverage.schema !== "vize.fixtureTypecheckerCoverage" || coverage.version !== 1) {
    invalid("typechecker coverage schema is unsupported");
  }
  for (const key of ["requested", "transitiveAuthored", "checked"]) {
    validateManifest(
      coverage[key],
      `typechecker coverage.${key}`,
      key === "requested" ? isVueSfc : isTypecheckSource,
    );
  }
  const combined = [...coverage.requested.files, ...coverage.transitiveAuthored.files].sort(
    byteSort,
  );
  if (JSON.stringify(combined) !== JSON.stringify(coverage.checked.files)) {
    invalid("typechecker coverage classes do not partition checked files");
  }
  return {
    requestedFileCount: coverage.requested.fileCount,
    requestedSha256: coverage.requested.sha256,
    transitiveAuthoredFileCount: coverage.transitiveAuthored.fileCount,
    transitiveAuthoredSha256: coverage.transitiveAuthored.sha256,
    checkedFileCount: coverage.checked.fileCount,
    checkedSha256: coverage.checked.sha256,
  };
}

function createManifest(files) {
  const digest = createHash("sha256");
  for (const file of files) digest.update(file).update("\0");
  return { fileCount: files.length, files: [...files], sha256: digest.digest("hex") };
}

function validateManifestInput(files, label, acceptsFile) {
  if (!Array.isArray(files)) invalid(`${label} must be an array`);
  const seen = new Set();
  for (const [index, file] of files.entries()) {
    requireRelativePath(file, `${label}[${index}]`);
    if (!acceptsFile(file)) invalid(`${label}[${index}] has an unsupported source extension`);
    if (seen.has(file)) invalid(`${label} contains duplicate file: ${file}`);
    seen.add(file);
  }
  if (JSON.stringify(files) !== JSON.stringify([...files].sort(byteSort))) {
    invalid(`${label} are not sorted`);
  }
}

function validateManifest(manifest, label, acceptsFile) {
  requireRecord(manifest, label);
  requireExactKeys(manifest, ["fileCount", "files", "sha256"], label);
  validateManifestInput(manifest.files, `${label}.files`, acceptsFile);
  if (manifest.fileCount !== manifest.files.length) invalid(`${label}.fileCount is inconsistent`);
  if (manifest.sha256 !== createManifest(manifest.files).sha256) {
    invalid(`${label}.sha256 is inconsistent`);
  }
}

function byteSort(left, right) {
  return Buffer.compare(Buffer.from(left), Buffer.from(right));
}

function isVueSfc(file) {
  return file.endsWith(".vue");
}

function isTypecheckSource(file) {
  return typecheckSourcePattern.test(file);
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
