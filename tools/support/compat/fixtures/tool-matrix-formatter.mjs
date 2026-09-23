import { spawnSync } from "node:child_process";
import { Buffer } from "node:buffer";
import { createHash } from "node:crypto";
import { readFileSync, statSync } from "node:fs";
import { isAbsolute, resolve } from "node:path";

import { collectInputPaths } from "./tool-matrix-inputs.mjs";

const noFilesMessage =
  "No .vue, .js, .mjs, .cjs, .ts, .mts, .cts, .jsx, .tsx, .json, .jsonc, .yaml, .yml, .md, or .markdown files found matching the patterns";

export function snapshotFormatterInputs(cwd, patterns) {
  const digest = createHash("sha256");
  for (const inputPath of collectInputPaths(cwd, patterns)) {
    const absolute = resolve(cwd, inputPath);
    const metadata = statSync(absolute, { bigint: true });
    digest.update(inputPath.replaceAll("\\", "/"));
    digest.update("\0");
    digest.update(String(metadata.mode));
    digest.update("\0");
    digest.update(String(metadata.mtimeNs));
    digest.update("\0");
    digest.update(readFileSync(absolute));
    digest.update("\0");
  }
  const status = spawnSync(
    "git",
    [
      "status",
      "--porcelain=v1",
      "-z",
      "--untracked-files=all",
      "--ignore-submodules=none",
      "--",
      ".",
    ],
    { cwd, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  if (status.error != null || status.status !== 0) {
    throw new Error("failed to snapshot formatter working tree");
  }
  digest.update(status.stdout);
  return digest.digest("hex");
}

export function validateFormatterOutput(
  project,
  stdout,
  stderr,
  exitCode,
  before,
  after,
  expectedFiles = null,
) {
  if (stdout !== "") invalid("stdout must be empty");
  if (before !== after) invalid("formatter check modified its working tree or input metadata");
  const normalizedStderr = stderr.replaceAll("\r\n", "\n");
  if (!normalizedStderr.endsWith("\n")) invalid("stderr must end with a newline");

  if (project.expectedVueFileCount === 0) {
    if (normalizedStderr !== `${noFilesMessage}\n`) {
      invalid("zero-file fixture emitted an unexpected report");
    }
    if (exitCode !== 1) invalid(`zero-file exit code ${exitCode} does not match expected 1`);
    return createFormatterChangeEvidence(0, []);
  }

  const lines = normalizedStderr.slice(0, -1).split("\n");
  const found = parseCount(lines[0], /^Found (\d+) file\(s\)$/, "found count");
  if (found === 0) invalid("non-empty fixture formatted zero files");
  if (expectedFiles != null && found !== expectedFiles.length) {
    invalid(`found count ${found} does not match ${expectedFiles.length} inputs`);
  }
  const changedPaths = [];
  let index = 1;
  while (lines[index]?.startsWith("Would reformat: ")) {
    const candidate = lines[index].slice("Would reformat: ".length);
    changedPaths.push(normalizeFormatterPath(candidate));
    index += 1;
  }
  if (lines[index] !== "") invalid("missing blank line before formatter summary");
  index += 1;
  const checked = parseCount(lines[index], /^Checked (\d+) file\(s\)$/, "checked count");
  index += 1;

  let changed = 0;
  const changedMatch = /^  (\d+) file\(s\) would be reformatted$/.exec(lines[index] ?? "");
  if (changedMatch != null) {
    changed = safeCount(changedMatch[1], "changed count");
    index += 1;
  }
  let unchanged = 0;
  const unchangedMatch = /^  (\d+) file\(s\) already formatted$/.exec(lines[index] ?? "");
  if (unchangedMatch != null) {
    unchanged = safeCount(unchangedMatch[1], "unchanged count");
    index += 1;
  }
  if (index !== lines.length) invalid("formatter report contains unexpected lines");
  if (new Set(changedPaths).size !== changedPaths.length) {
    invalid("formatter report contains duplicate changed paths");
  }
  if (expectedFiles != null) {
    const expectedSet = new Set(expectedFiles);
    const unexpected = changedPaths.filter((file) => !expectedSet.has(file));
    if (unexpected.length > 0) {
      invalid(`changed files are not fixture inputs: ${unexpected.join(", ")}`);
    }
  }
  if (changed !== changedPaths.length) {
    invalid(`changed count ${changed} does not match ${changedPaths.length} paths`);
  }
  if (found !== checked || checked !== changed + unchanged) {
    invalid(
      `file counts do not reconcile: found ${found}, checked ${checked}, changed ${changed}, unchanged ${unchanged}`,
    );
  }
  const expectedExitCode = changed > 0 ? 1 : 0;
  if (exitCode !== expectedExitCode) {
    invalid(`exit code ${exitCode} does not match expected ${expectedExitCode}`);
  }
  return createFormatterChangeEvidence(checked, changedPaths);
}

/** Build the canonical evidence shared by formatter check and write validation. */
export function createFormatterChangeEvidence(checkedFileCount, changedPaths) {
  const digest = createHash("sha256");
  for (const inputPath of [...changedPaths].sort(byteOrder)) {
    digest.update(inputPath);
    digest.update("\0");
  }
  const evidence = {
    checkedFileCount,
    changedFileCount: changedPaths.length,
    unchangedFileCount: checkedFileCount - changedPaths.length,
    changedPathsSha256: digest.digest("hex"),
  };
  validateFormatterChangeEvidence(evidence);
  return evidence;
}

/** Validate persisted formatter evidence before comparing separate CLI runs. */
export function validateFormatterChangeEvidence(evidence, label = "formatter change evidence") {
  if (evidence == null || typeof evidence !== "object" || Array.isArray(evidence)) {
    throw new Error(`invalid ${label}`);
  }
  const keys = ["changedFileCount", "changedPathsSha256", "checkedFileCount", "unchangedFileCount"];
  if (JSON.stringify(Object.keys(evidence).sort()) !== JSON.stringify(keys)) {
    throw new Error(`invalid ${label} keys`);
  }
  for (const field of ["checkedFileCount", "changedFileCount", "unchangedFileCount"]) {
    if (!Number.isSafeInteger(evidence[field]) || evidence[field] < 0) {
      throw new Error(`invalid ${label} ${field}`);
    }
  }
  if (evidence.checkedFileCount !== evidence.changedFileCount + evidence.unchangedFileCount) {
    throw new Error(`${label} counts do not reconcile`);
  }
  if (!/^[0-9a-f]{64}$/.test(evidence.changedPathsSha256)) {
    throw new Error(`invalid ${label} changedPathsSha256`);
  }
}

function byteOrder(left, right) {
  return Buffer.compare(Buffer.from(left), Buffer.from(right));
}

function parseCount(line, pattern, label) {
  const match = pattern.exec(line ?? "");
  if (match == null) invalid(`missing ${label}`);
  return safeCount(match[1], label);
}

function safeCount(value, label) {
  const count = Number(value);
  if (!Number.isSafeInteger(count) || count < 0) invalid(`${label} is not a safe integer`);
  return count;
}

function normalizeFormatterPath(value) {
  const bare = value.startsWith("./") ? value.slice(2) : value;
  if (
    bare.length === 0 ||
    isAbsolute(bare) ||
    /^[A-Za-z]:[\\/]/.test(bare) ||
    bare.includes("\\") ||
    bare.split("/").some((segment) => segment === "" || segment === "." || segment === "..")
  ) {
    invalid("changed file must be a normalized relative path");
  }
  if (!bare.endsWith(".vue")) invalid(`changed file is not a Vue SFC: ${value}`);
  return bare;
}

function invalid(message) {
  throw new Error(`invalid formatter check output: ${message}`);
}
