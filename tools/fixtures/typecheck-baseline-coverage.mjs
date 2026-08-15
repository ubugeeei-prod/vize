import { Buffer } from "node:buffer";
import { createHash } from "node:crypto";
import { isAbsolute, relative } from "node:path";

import { isSupportVueFile, normalizePath } from "./typecheck-divergence-input.mjs";

/**
 * Prove that the diagnostic comparator ran over the same Vue corpus on both
 * sides. `vue-tsc --listFilesOnly` prints absolute program paths;
 * Vize's matrix artifact already carries its sorted checked-file list. That
 * list also carries the transitive authored TS/TSX sources Vize pulled into the
 * program, so only its `.vue` entries are comparable with the baseline set.
 */
export function evaluateVueProgramCoverage(vizeReport, vueTscOutput, cwd) {
  const vizeVueFiles = vizeReport.files
    .slice(0, vizeReport.fileCount)
    // The compiled program paths below are cwd-relative and slash-normalized, so
    // the Vize side has to go through the same normalization contract or an
    // absolute or backslash-separated report path can never match it.
    .map((entry, index) => normalizePath(entry?.file, cwd, `Vize files[${index}].file`))
    .filter((file) => file.endsWith(".vue"))
    .sort(byteOrder);
  const {
    authoredFiles: baselineVueFiles,
    dependencyFiles: baselineDependencyVueFiles,
    supportFiles: baselineSupportVueFiles,
  } = collectVueTscProgramFiles(vueTscOutput, cwd, new Set(vizeVueFiles));
  const vizeSet = new Set(vizeVueFiles);
  const baselineSet = new Set(baselineVueFiles);
  const missingVueFiles = vizeVueFiles.filter((file) => !baselineSet.has(file));
  const unexpectedVueFiles = baselineVueFiles.filter((file) => !vizeSet.has(file));
  const sharedVueFileCount = vizeVueFiles.length - missingVueFiles.length;
  const unusableReason =
    missingVueFiles.length === 0 && unexpectedVueFiles.length === 0
      ? null
      : `vue-tsc checked ${baselineVueFiles.length} Vue files while Vize checked ` +
        `${vizeVueFiles.length} (missing ${missingVueFiles.length}, unexpected ` +
        `${unexpectedVueFiles.length})`;
  return {
    baselineVueFileCount: baselineVueFiles.length,
    baselineVueFilesSha256: fileListHash(baselineVueFiles),
    ignoredDependencyVueFileCount: baselineDependencyVueFiles.length,
    ignoredDependencyVueFilesSha256: fileListHash(baselineDependencyVueFiles),
    ignoredSupportVueFileCount: baselineSupportVueFiles.length,
    ignoredSupportVueFilesSha256: fileListHash(baselineSupportVueFiles),
    missingVueFiles,
    sharedVueFileCount,
    unexpectedVueFiles,
    unusableReason,
    verdict: unusableReason == null ? "usable" : "unusable",
    vizeVueFileCount: vizeVueFiles.length,
    vizeVueFilesSha256: fileListHash(vizeVueFiles),
  };
}

function collectVueTscProgramFiles(output, cwd, comparableVueFiles) {
  const files = new Set();
  const dependencyFiles = new Set();
  const supportFiles = new Set();
  for (const rawLine of output.replaceAll("\r\n", "\n").split("\n")) {
    const line = rawLine.trimEnd();
    // `--listFilesOnly` paths are absolute. Requiring that shape keeps diagnostic
    // messages which happen to end in ".vue" out of the coverage evidence.
    if (!line.endsWith(".vue") || !isAbsolute(line)) continue;
    const file = relative(cwd, line).replaceAll("\\", "/");
    if (
      file.length === 0 ||
      isAbsolute(file) ||
      /^[A-Za-z]:\//.test(file) ||
      file.split("/").some((segment) => segment === "" || segment === "." || segment === "..")
    ) {
      continue;
    }
    if (file.split("/").includes("node_modules")) {
      dependencyFiles.add(file);
    } else if (!comparableVueFiles.has(file) && isSupportVueFile(file)) {
      supportFiles.add(file);
    } else {
      files.add(file);
    }
  }
  return {
    authoredFiles: [...files].sort(byteOrder),
    dependencyFiles: [...dependencyFiles].sort(byteOrder),
    supportFiles: [...supportFiles].sort(byteOrder),
  };
}

function fileListHash(files) {
  return createHash("sha256")
    .update(files.map((file) => `${file}\n`).join(""))
    .digest("hex");
}

function byteOrder(left, right) {
  return Buffer.compare(Buffer.from(left), Buffer.from(right));
}
