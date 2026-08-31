// Shared plumbing for the Davinci P0-13 FP/FN pilot tools
// (seed-defects.mjs, suppression-telemetry.mjs): repository locations, the
// corpus-shard definition, `vize lint` invocation, and the text-coordinate
// utilities both tools need to talk about diagnostic locations exactly.
//
// Coordinate conventions (must match vize_patina/src/output/shared.rs):
//   - lines are 1-based, split on "\n";
//   - columns are 1-based and count Unicode code points, not UTF-16 units
//     and not bytes;
//   - spans in manifests/reports are JS string indices (UTF-16 units) into
//     the exact file text they are recorded against — they are an internal
//     identity key, always paired with the line/column form.

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
  "..",
);

// The P0-13 pilot corpus shard: three small MIT-licensed projects from the
// pinned corpus (tests/_fixtures/vue-ecosystem-fixtures.json). Hydrate with:
//   git submodule update --init --depth 1 -- tests/_fixtures/_git/<id>
export const CORPUS_SHARD = ["splitpanes", "layoutit-grid", "cssgridgenerator"];

export function shardProjectDir(id) {
  return path.join(repoRoot, "tests", "_fixtures", "_git", id);
}

/** List every .vue file under `dir`, as sorted /-separated relative paths. */
export function listVueFiles(dir) {
  const found = [];
  const walk = (current) => {
    for (const entry of fs
      .readdirSync(current, { withFileTypes: true })
      .sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0))) {
      if (entry.name === "node_modules" || entry.name.startsWith(".")) continue;
      const target = path.join(current, entry.name);
      if (entry.isDirectory()) walk(target);
      else if (entry.isFile() && entry.name.endsWith(".vue")) found.push(target);
    }
  };
  walk(dir);
  return found
    .map((file) => path.relative(dir, file).split(path.sep).join("/"))
    .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
}

// --- vize CLI ---------------------------------------------------------------

/**
 * Resolve the vize CLI. Preference order: $VIZE_BIN, the ci/release/debug
 * build outputs, `vize` on PATH, then `cargo run -q -p vize --` as the
 * last-resort fallback (matching tests/tooling/cli-lint-contract.test.ts).
 */
export function resolveVizeCli() {
  const candidates = [];
  if (process.env.VIZE_BIN) candidates.push(process.env.VIZE_BIN);
  candidates.push(
    path.join(repoRoot, "target", "ci", "vize"),
    path.join(repoRoot, "target", "release", "vize"),
    path.join(repoRoot, "target", "debug", "vize"),
    "vize",
  );
  for (const candidate of candidates) {
    const probe = spawnSync(candidate, ["--version"], { cwd: repoRoot, encoding: "utf8" });
    if (probe.status === 0) return { command: candidate, prefix: [] };
  }
  return { command: "cargo", prefix: ["run", "-q", "-p", "vize", "--"] };
}

/**
 * Run `vize lint --no-config --format json` over `files` (relative paths)
 * with `cwd` as the working directory, and parse the JSON result. Exit
 * status 0 (clean/warnings) and 1 (errors) both carry a valid report;
 * anything else throws with the captured stderr.
 */
export function runVizeLintJson(cli, cwd, files) {
  const result = spawnSync(
    cli.command,
    [...cli.prefix, "lint", "--no-config", "--format", "json", ...files],
    { cwd, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  if (result.error) throw result.error;
  if (result.status !== 0 && result.status !== 1) {
    throw new Error(
      `vize lint exited with status ${result.status}:\n${result.stderr}\n${result.stdout}`,
    );
  }
  let parsed;
  try {
    parsed = JSON.parse(result.stdout);
  } catch {
    throw new Error(`vize lint emitted non-JSON output:\n${result.stdout}\n${result.stderr}`);
  }
  if (!Array.isArray(parsed)) throw new Error("vize lint JSON output is not an array");
  return parsed;
}

/** Flatten a vize lint JSON report into location-identity diagnostic rows. */
export function flattenLintJson(report) {
  const rows = [];
  for (const fileResult of report) {
    for (const message of fileResult.messages) {
      rows.push({
        path: fileResult.file.split(path.sep).join("/"),
        ruleId: message.ruleId,
        severity: message.severity,
        line: message.line,
        column: message.column,
        endLine: message.endLine,
        endColumn: message.endColumn,
      });
    }
  }
  return sortDiagnostics(rows);
}

export function sortDiagnostics(rows) {
  return rows.sort(
    (a, b) =>
      compareStrings(a.path, b.path) ||
      a.line - b.line ||
      a.column - b.column ||
      a.endLine - b.endLine ||
      a.endColumn - b.endColumn ||
      compareStrings(a.ruleId, b.ruleId) ||
      a.severity - b.severity,
  );
}

export function diagnosticKey(row) {
  return [
    row.path,
    row.ruleId,
    row.severity,
    row.line,
    row.column,
    row.endLine,
    row.endColumn,
  ].join("|");
}

function compareStrings(a, b) {
  return a < b ? -1 : a > b ? 1 : 0;
}

// --- text coordinates -------------------------------------------------------

/** JS-index offsets of every line start ("\n"-separated, 1-based lines). */
export function lineStartsOf(text) {
  const starts = [0];
  for (let i = 0; i < text.length; i += 1) {
    if (text[i] === "\n") starts.push(i + 1);
  }
  return starts;
}

/** Convert a JS string index into {line, column} (code-point columns). */
export function indexToLineCol(text, lineStarts, index) {
  let low = 0;
  let high = lineStarts.length - 1;
  while (low < high) {
    const mid = (low + high + 1) >> 1;
    if (lineStarts[mid] <= index) low = mid;
    else high = mid - 1;
  }
  const column = codePointLength(text.slice(lineStarts[low], index)) + 1;
  return { line: low + 1, column };
}

/** Convert a 1-based {line, column} (code-point columns) to a JS index. */
export function lineColToIndex(text, lineStarts, line, column) {
  const start = lineStarts[line - 1];
  if (start == null) return null;
  const end = line < lineStarts.length ? lineStarts[line] : text.length;
  const lineText = text.slice(start, end);
  let remaining = column - 1;
  let jsOffset = 0;
  for (const codePoint of lineText) {
    if (remaining === 0) break;
    remaining -= 1;
    jsOffset += codePoint.length;
  }
  if (remaining > 0) return null;
  return start + jsOffset;
}

function codePointLength(slice) {
  let count = 0;
  for (const _ of slice) count += 1;
  return count;
}

// --- deterministic JSON -----------------------------------------------------

/** Write `value` as pretty JSON with a trailing newline (byte-stable). */
export function writeJson(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

export function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}
