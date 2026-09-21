// The error-severity witness exemption inventory (P4-6a): parse the committed
// ledger, derive it mechanically from the Rust sources, and compare it with
// the base revision so it can only shrink.
//
// Contract (`davinci-road/plan/phase-4-tasks-later.md#p4-6a`): a producer that
// predates the witness SDK reports errors through
// `Diagnostic::legacy_error(&EXEMPTION, ..)`, where `EXEMPTION` is a named
// `static` declared with `Exemption::new("<producer>", "<code>")`. A row is
// one declaration; `exempt` counts the construction sites (`&EXEMPTION`
// references in the producer crate's `src/`) that report under it.

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

export const inventoryRelative = "davinci-road/plan/witness-exemptions.tsv";
export const inventoryHeader = "producer\tcode\texempt";

export type ExemptionRow = { producer: string; code: string; exempt: number };
export type Derived = { rows: ExemptionRow[]; issues: string[] };

const name = /^[a-z0-9_./-]+$/u;
const declaration =
  /^\s*(?:pub(?:\([^)]*\))?\s+)?static\s+(?<ident>[A-Z][A-Z0-9_]*)\s*:\s*Exemption\s*=\s*Exemption::new\("(?<producer>[^"]*)",\s*"(?<code>[^"]*)"\);\s*$/u;

const key = (row: { producer: string; code: string }): string => `${row.producer}\t${row.code}`;

/**
 * A contract table: one const constructor builds each row's exemption
 * (`Exemption::new(PRODUCER, name)`, the only non-static `Exemption::new` the
 * scan accepts, exactly once), and every `row!(name, Tier, DOMAIN, Error)`
 * line is one exempt rule — one construction site each.
 */
export interface ContractTable {
  producer: string;
  constructor: string;
  rows: string;
}

export const contractTables: readonly ContractTable[] = [
  {
    producer: "vize_patina",
    constructor: "crates/vize_patina/src/rule_contracts.rs",
    rows: "crates/vize_patina/src/rule_contracts/table.rs",
  },
];

const tableRow =
  /^\s*row!\("(?<code>[^"]+)",\s*(?:Exact|Sound|Complete|Heuristic),\s*[A-Z0-9_]+,\s*(?<severity>Error|Warning)\),\s*$/u;

function compareRows(left: ExemptionRow, right: ExemptionRow): number {
  if (left.producer !== right.producer) return left.producer < right.producer ? -1 : 1;
  if (left.code === right.code) return 0;
  return left.code < right.code ? -1 : 1;
}

export function parseInventory(source: string): ExemptionRow[] {
  const lines = source.split("\n");
  if (lines.at(-1) !== "") throw new Error("the inventory must end with one newline");
  lines.pop();
  if (lines.shift() !== inventoryHeader) throw new Error("the inventory header drifted");
  const rows = lines.map((line, index) => {
    const fields = line.split("\t");
    if (fields.length !== 3) throw new Error(`line ${index + 2}: expected 3 fields`);
    const [producer, code, exempt] = fields;
    if (!name.test(producer) || !name.test(code)) {
      throw new Error(`line ${index + 2}: names must be [a-z0-9_./-]`);
    }
    if (!/^[1-9]\d*$/u.test(exempt)) {
      throw new Error(`line ${index + 2}: exempt must be a positive count`);
    }
    return { producer, code, exempt: Number(exempt) };
  });
  for (let index = 1; index < rows.length; index += 1) {
    if (compareRows(rows[index - 1], rows[index]) >= 0) {
      throw new Error(`line ${index + 2}: rows must be sorted and unique`);
    }
  }
  return rows;
}

export function formatInventory(rows: readonly ExemptionRow[]): string {
  const body = rows.map((row) => `${row.producer}\t${row.code}\t${row.exempt}\n`).join("");
  return `${inventoryHeader}\n${body}`;
}

function rustFiles(dir: string): string[] {
  if (!fs.existsSync(dir)) return [];
  const files: string[] = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const absolute = path.join(dir, entry.name);
    if (entry.isDirectory()) files.push(...rustFiles(absolute));
    else if (entry.isFile() && entry.name.endsWith(".rs")) files.push(absolute);
  }
  return files.sort();
}

/** Every non-comment line of `file`, with its 1-based number. */
function codeLines(file: string): Array<{ line: number; text: string }> {
  return fs
    .readFileSync(file, "utf8")
    .split("\n")
    .map((text, index) => ({ line: index + 1, text }))
    .filter(({ text }) => !text.trimStart().startsWith("//"));
}

/**
 * Derive the inventory from `crates/<crate>/src/**` under `root`: every
 * `Exemption::new` must be a named-static declaration whose producer is its
 * crate, and each declaration's row counts its `&NAME` construction sites.
 */
export function deriveInventory(
  root: string,
  tables: readonly ContractTable[] = contractTables,
): Derived {
  const rows: ExemptionRow[] = [];
  const issues: string[] = [];
  const constructors = new Map(tables.map((table) => [table.constructor, 0]));
  for (const table of tables) {
    const file = path.join(root, table.rows);
    if (!fs.existsSync(file)) {
      issues.push(`${table.rows}: contract table is missing`);
      continue;
    }
    for (const { line, text } of codeLines(file)) {
      if (!text.trimStart().startsWith("row!(")) continue;
      const match = tableRow.exec(text);
      if (!match) issues.push(`${table.rows}:${line}: a table row must be one row!(..) line`);
      else if (match.groups!.severity === "Error") {
        rows.push({ producer: table.producer, code: match.groups!.code, exempt: 1 });
      }
    }
  }
  const cratesDir = path.join(root, "crates");
  for (const crate of fs.readdirSync(cratesDir).sort()) {
    const files = rustFiles(path.join(cratesDir, crate, "src"));
    const declared: Array<ExemptionRow & { ident: string; at: string }> = [];
    for (const file of files) {
      const relative = path.relative(root, file);
      for (const { line, text } of codeLines(file)) {
        if (!text.includes("Exemption::new(")) continue;
        const at = `${relative}:${line}`;
        if (constructors.has(relative) && /Exemption::new\(PRODUCER, name\)/u.test(text)) {
          constructors.set(relative, constructors.get(relative)! + 1);
          continue;
        }
        const match = declaration.exec(text);
        if (!match) {
          issues.push(`${at}: an Exemption must be declared as one named static`);
          continue;
        }
        const { ident, producer, code } = match.groups!;
        if (producer !== crate) {
          issues.push(`${at}: producer ${producer} must be its crate ${crate}`);
          continue;
        }
        declared.push({ ident, producer, code, exempt: 0, at });
      }
    }
    for (const entry of declared) {
      const reference = new RegExp(`&\\s*(?:[A-Za-z_][A-Za-z0-9_]*::)*${entry.ident}\\b`, "gu");
      for (const file of files) {
        for (const { text } of codeLines(file)) {
          entry.exempt += [...text.matchAll(reference)].length;
        }
      }
      if (entry.exempt === 0) issues.push(`${entry.at}: ${entry.ident} is declared but unused`);
      else rows.push({ producer: entry.producer, code: entry.code, exempt: entry.exempt });
    }
  }
  for (const [constructor, count] of constructors) {
    if (count !== 1) issues.push(`${constructor}: expected one table constructor, found ${count}`);
  }
  rows.sort(compareRows);
  for (let index = 1; index < rows.length; index += 1) {
    if (compareRows(rows[index - 1], rows[index]) === 0) {
      issues.push(`${key(rows[index]).replace("\t", "/")}: declared more than once`);
    }
  }
  return { rows, issues };
}

/**
 * The ratchet: against `base`, no count rises and no producer already on the
 * unified channel gains a code. A producer absent from `base` is joining the
 * channel, and its legacy errors enter the inventory with it (the P4-6c
 * onboarding shape); `base === undefined` is the inventory's introduction.
 */
export function ratchetViolations(
  base: readonly ExemptionRow[] | undefined,
  head: readonly ExemptionRow[],
): string[] {
  if (base === undefined) return [];
  const before = new Map(base.map((row) => [key(row), row.exempt]));
  const producers = new Set(base.map((row) => row.producer));
  const violations: string[] = [];
  for (const row of head) {
    const label = `${row.producer}/${row.code}`;
    const previous = before.get(key(row));
    if (previous === undefined) {
      if (producers.has(row.producer)) {
        violations.push(`${label}: new exemption for a producer already on the channel`);
      }
    } else if (row.exempt > previous) {
      violations.push(`${label}: exempt rose from ${previous} to ${row.exempt}`);
    }
  }
  return violations;
}

function git(
  cwd: string,
  args: string[],
): { status: number | null; stdout: string; stderr: string } {
  return spawnSync("git", args, { cwd, encoding: "utf8" });
}

/** The inventory at `ref`, or `undefined` when it does not exist there. */
export function readBaseInventory(cwd: string, ref: string): ExemptionRow[] | undefined {
  const exists = git(cwd, ["cat-file", "-e", `${ref}:${inventoryRelative}`]);
  if (exists.status !== 0) {
    const resolved = git(cwd, ["rev-parse", "--verify", `${ref}^{commit}`]);
    if (resolved.status !== 0) throw new Error(`base revision ${ref} is not available`);
    return undefined;
  }
  const shown = git(cwd, ["show", `${ref}:${inventoryRelative}`]);
  if (shown.status !== 0) throw new Error(shown.stderr.trim());
  return parseInventory(shown.stdout);
}

/**
 * The revision to ratchet against: an explicit override, the pull request's
 * base (fetched when the checkout is shallow), or the merge base with
 * `origin/main`; `HEAD` itself when none exists, which compares vacuously.
 */
export function resolveBaseRef(cwd: string, env: NodeJS.ProcessEnv): string {
  if (env.WITNESS_EXEMPTIONS_BASE_REF) return env.WITNESS_EXEMPTIONS_BASE_REF;
  if (env.GITHUB_BASE_REF) {
    if (!env.GITHUB_EVENT_PATH) throw new Error("GITHUB_EVENT_PATH is required for pull requests");
    const event = JSON.parse(fs.readFileSync(env.GITHUB_EVENT_PATH, "utf8")) as {
      pull_request?: { base?: { sha?: unknown } };
    };
    const sha = event.pull_request?.base?.sha;
    if (typeof sha !== "string" || !/^[0-9a-f]{40}$/u.test(sha)) {
      throw new Error("pull_request.base.sha must be a full lowercase commit SHA");
    }
    const fetched = git(cwd, ["fetch", "--no-tags", "--depth=1", "origin", sha]);
    if (fetched.status !== 0) throw new Error(fetched.stderr.trim());
    return sha;
  }
  const mergeBase = git(cwd, ["merge-base", "HEAD", "origin/main"]);
  return mergeBase.status === 0 ? mergeBase.stdout.trim() : "HEAD";
}
