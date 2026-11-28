// The temporary-storage inventory: every product source line that creates or
// resolves storage under the system temp directory must be listed, with its
// purpose and cleanup owner, in `temp-producers.tsv`.
//
//   node tools/ci/commands/temp-producers.ts --check    fail on unlisted or changed sites
//   node tools/ci/commands/temp-producers.ts --list     print the sites found per file
//
// A site is a non-comment line of product code that names a temp-dir API:
// `std::env::temp_dir`, the `tempfile` crate, `os.tmpdir()` or `mkdtemp`.
// Test code is not product code: `tests/` directories, `*_tests.rs`-style
// files, `#[cfg(test)]` items and the modules they declare are skipped.
// Each inventory row records the exact site count of one file, so a new site
// fails `--check` until it is listed, and a removed one until its row shrinks.

import fs from "node:fs";
import path from "node:path";

import { isEntrypoint } from "../../support/command-runtime.ts";
import { gitTrackedFiles } from "../../support/git-tracked-files.ts";

export const inventoryPath = "tools/ci/commands/temp-producers.tsv";

export const inventoryColumns = [
  "path",
  "sites",
  "purpose",
  "frequency",
  "size",
  "lifetime",
  "permissions",
  "owner",
] as const;

export interface InventoryRow {
  readonly path: string;
  readonly sites: number;
  readonly fields: Readonly<Record<(typeof inventoryColumns)[number], string>>;
}

export interface Finding {
  readonly path: string;
  readonly listed: number;
  readonly found: number;
}

const productRoots = ["crates/", "npm/"];
const excludedSegments = new Set([
  "node_modules",
  "target",
  "dist",
  "fixtures",
  "_fixtures",
  "__fixtures__",
  "examples",
  "benches",
  "testdata",
  "scripts",
  "test-support",
  "test-programs",
]);
const excludedNames = /\.(?:fixtures?|bench)\.[cm]?[jt]sx?$/u;
const rustSite = /\btempfile::|\btemp_dir\s*\(|\bNamedTempFile\b|\bTempDir\b|\btempdir\s*\(/u;
const scriptSite = /\btmpdir\s*\(|\bmkdtemp(?:Sync)?\s*\(/u;
const scriptExtension = /\.(?:[cm]?[jt]s|[jt]sx)$/u;
const testAttribute = /^\s*#\[cfg\((?:all\()?test\b/u;
const testAttributePrefix = /^\s*#\[cfg\([^\]]*\]/u;
const testModule =
  /#\[cfg\((?:all\()?test\b[^\]]*\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;/gu;

/** The test-path rule of `readability/counters.ts`, which needs the `vite` package. */
export function isTestPath(file: string): boolean {
  return (
    file.startsWith("tests/") ||
    /\/(?:tests?|__tests__|e2e|benches)\//u.test(file) ||
    /(?:^|[/_.-])tests?(?:[/_.-]|$)|\.spec\.[cm]?[jt]sx?$/u.test(file.split("/").at(-1) ?? "")
  );
}

/** Files and directories a Rust file declares as `#[cfg(test)] mod name;`. */
export function rustTestModules(file: string, source: string): string[] {
  const directory = file.slice(0, file.lastIndexOf("/") + 1);
  const stem = file.slice(directory.length, -3);
  const base = ["mod", "lib", "main"].includes(stem) ? directory : `${directory}${stem}/`;
  return [...source.matchAll(testModule)].flatMap((match) => [
    `${base}${match[1]}.rs`,
    `${base}${match[1]}/`,
  ]);
}

function byCodePoint(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

export function inScope(file: string): boolean {
  if (!productRoots.some((root) => file.startsWith(root))) return false;
  if (!file.endsWith(".rs") && !scriptExtension.test(file)) return false;
  if (file.endsWith("/build.rs") || file.endsWith(".d.ts")) return false;
  if (isTestPath(file) || excludedNames.test(file)) return false;
  return !file
    .split("/")
    .slice(0, -1)
    .some((segment) => excludedSegments.has(segment));
}

/** Drops string literals and `//` comments so braces and names are code only. */
function codeOf(line: string): string {
  return line
    .replace(/"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])'/gu, '""')
    .replace(/\/\/.*$/u, "")
    .trim();
}

function delta(code: string, open: string, close: string): number {
  let balance = 0;
  for (const char of code) {
    if (char === open) balance++;
    else if (char === close) balance--;
  }
  return balance;
}

/**
 * The lines of a Rust source that belong to product code: `#[cfg(test)]`
 * items (an inline `mod tests { ... }`, a test-only `fn` or `use`) are dropped.
 */
export function rustProductLines(source: string): string[] {
  const lines = source.split("\n");
  const kept: string[] = [];
  for (let index = 0; index < lines.length; index++) {
    if (!testAttribute.test(lines[index])) {
      kept.push(lines[index]);
      continue;
    }
    // The item may follow its attribute on the same line.
    let line = lines[index].replace(testAttributePrefix, "");
    let depth = 0;
    let attribute = 0;
    let opened = false;
    for (;;) {
      const code = codeOf(line);
      if (!opened && (attribute > 0 || code.startsWith("#["))) {
        // Another attribute on the item, possibly over several lines.
        attribute += delta(code, "[", "]");
      } else if (opened || code !== "") {
        depth += delta(code, "{", "}");
        if (code.includes("{")) opened = true;
        if (opened ? depth <= 0 : /[;,]$/u.test(code)) break;
      }
      if (++index >= lines.length) break;
      line = lines[index];
    }
  }
  return kept;
}

function isComment(line: string): boolean {
  return /^\s*(?:\/\/|\/\*|\*)/u.test(line);
}

export function countSites(file: string, source: string): number {
  const rust = file.endsWith(".rs");
  const lines = rust ? rustProductLines(source) : source.split("\n");
  const site = rust ? rustSite : scriptSite;
  return lines.filter((line) => !isComment(line) && site.test(codeOf(line))).length;
}

export function scanTree(root: string, files?: readonly string[]): Map<string, number> {
  const sources = new Map<string, string>();
  const testFiles = new Set<string>();
  const testDirectories: string[] = [];
  for (const file of files ?? gitTrackedFiles(root)) {
    if (!inScope(file)) continue;
    const absolute = path.join(root, file);
    if (!fs.lstatSync(absolute, { throwIfNoEntry: false })?.isFile()) continue;
    const source = fs.readFileSync(absolute, "utf8");
    sources.set(file, source);
    if (file.endsWith(".rs"))
      for (const module of rustTestModules(file, source))
        if (module.endsWith("/")) testDirectories.push(module);
        else testFiles.add(module);
  }
  const sites = new Map<string, number>();
  for (const [file, source] of sources) {
    if (testFiles.has(file) || testDirectories.some((prefix) => file.startsWith(prefix))) continue;
    const count = countSites(file, source);
    if (count > 0) sites.set(file, count);
  }
  return new Map([...sites].sort(([left], [right]) => byCodePoint(left, right)));
}

export function parseInventory(text: string): InventoryRow[] {
  const rows: InventoryRow[] = [];
  const seen = new Set<string>();
  const lines = text.split("\n").filter((line) => line !== "" && !line.startsWith("#"));
  const [header, ...body] = lines;
  if (header !== inventoryColumns.join("\t"))
    throw new Error(`inventory header must be: ${inventoryColumns.join(" ")}`);
  for (const line of body) {
    const cells = line.split("\t");
    if (cells.length !== inventoryColumns.length || cells.some((cell) => cell.trim() === ""))
      throw new Error(`inventory row needs ${inventoryColumns.length} non-empty cells: ${line}`);
    const sites = Number(cells[1]);
    if (!Number.isInteger(sites) || sites < 1)
      throw new Error(`inventory sites must be a positive integer: ${line}`);
    if (seen.has(cells[0])) throw new Error(`duplicate inventory row: ${cells[0]}`);
    seen.add(cells[0]);
    const fields = Object.fromEntries(
      inventoryColumns.map((column, at) => [column, cells[at]]),
    ) as InventoryRow["fields"];
    rows.push({ path: cells[0], sites, fields });
  }
  const order = rows.map((row) => row.path);
  const sorted = [...order].sort(byCodePoint);
  if (order.some((entry, at) => entry !== sorted[at]))
    throw new Error("inventory rows must be sorted by path");
  return rows;
}

export function compare(
  found: ReadonlyMap<string, number>,
  rows: readonly InventoryRow[],
): Finding[] {
  const listed = new Map(rows.map((row) => [row.path, row.sites]));
  const paths = [...new Set([...found.keys(), ...listed.keys()])].sort(byCodePoint);
  return paths
    .map((file) => ({ path: file, listed: listed.get(file) ?? 0, found: found.get(file) ?? 0 }))
    .filter((finding) => finding.listed !== finding.found);
}

export function describe(finding: Finding): string {
  if (finding.listed === 0)
    return `${finding.path}: ${finding.found} unlisted temp site(s); add a row to ${inventoryPath}`;
  if (finding.found === 0)
    return `${finding.path}: no temp sites remain; delete its row from ${inventoryPath}`;
  return `${finding.path}: ${finding.found} temp site(s), inventory lists ${finding.listed}; update the row`;
}

export function main(argv: readonly string[], root = process.cwd()): number {
  const found = scanTree(root);
  if (argv.includes("--list")) {
    for (const [file, count] of found) process.stdout.write(`${file}\t${count}\n`);
    return 0;
  }
  const rows = parseInventory(fs.readFileSync(path.join(root, inventoryPath), "utf8"));
  const findings = compare(found, rows);
  for (const finding of findings) process.stderr.write(`${describe(finding)}\n`);
  return findings.length === 0 ? 0 : 1;
}

if (isEntrypoint(import.meta.url)) process.exitCode = main(process.argv.slice(2));
