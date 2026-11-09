/**
 * Replay the codename removal (#6832; maintainer decisions 2026-11-03 and 2026-11-09).
 *
 * - Program name: every path and line outside docs/davinci/ loses it. A whole word becomes
 *   `levels` (the level crates' home), an embedded one becomes `level`.
 * - Level codenames: each becomes its level name (see `levelNames`) in paths and lines. Markdown
 *   under docs/ and lines that say "codename" (the alias form) keep them; other data under docs/
 *   names current code and follows.
 *   A level codename counts only where a word part starts, so `historicalComplete` and
 *   `portfolio` are not codenames.
 *
 * Case is preserved. On a conflict, take main and re-run.
 *   node tools/support/levels/rename-codenames.ts --check|--move|--rewrite
 * Replay: `--move` and commit (move-only), then `--rewrite` (reformats what it touched) and
 * commit; `--check` is then empty.
 */
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const NAME = "davinci";
const kept = "docs/" + NAME;
const self = "tools/support/levels/rename-codenames.ts";
// The completion gate must spell the names it bans; it moves but keeps its bytes.
const gate = /^tools\/commands\/[^/]+\/completion-gates\.ts$/u;
const capture =
  /\.(?:log|jsonl|stderr|stdout)(?:\.txt)?$|(?:^|\/)(?:stderr|stdout)\.txt$|(?:^|\/)(?:fix-history|capture-[^/]+)\/|(?:^|\/)(?:original-[^/]*|observed-[^/]*\.json|[^/]*\.observation\.json)$/u;
/**
 * Captured output that a live parser or comparison reads in tests is a test input and follows the
 * rename: observer logs, the generated L3 reference graphs and the dump CLI's expected stderr.
 */
const parsedCapture =
  /^(?:tests\/tooling\/fixtures\/canonical-observer-logs|tests\/fixtures\/l3-reference|crates\/vize\/tests\/fixtures\/dump_cli)\//u;
/** Records that pin git trees at past revisions name the paths those trees had. */
const pinnedTrees = /^editors\/vscode\/test\/css-hover-source-custody\.json$/u;
/** Captured output of a past run is history and keeps its bytes. */
export const keepsBytes = (file: string): boolean =>
  pinnedTrees.test(file) || (capture.test(file) && !parsedCapture.test(file));

interface LevelName {
  /** Lowercase spelling for snake, kebab and plain words. */
  readonly lower: string;
  /** Kebab spelling, when it differs from `lower`. */
  readonly kebab?: string;
  /** Spelling as a capitalized word part. */
  readonly pascal: string;
  /** Spelling as a whole lowercase word, when it differs from `lower`. */
  readonly word?: string;
}

/** Each level codename and the name that replaces it. */
export const levelNames: Readonly<Record<string, LevelName>> = {
  sinopia: { lower: "l1", pascal: "L1" },
  ricalco: { lower: "l1_to_l2", kebab: "l1-to-l2", pascal: "L1ToL2" },
  disegno: { lower: "l2", pascal: "L2" },
  impeto: { lower: "l3", pascal: "L3" },
  stesura: { lower: "l4", pascal: "L4" },
  folio: { lower: "dump", pascal: "Dump" },
  spolvero: { lower: "stage", pascal: "Stage", word: "stages" },
};
const levelPattern = new RegExp(Object.keys(levelNames).join("|"), "giu");
const anyPattern = new RegExp([NAME, ...Object.keys(levelNames)].join("|"), "giu");

const wordChar = /[A-Za-z0-9_-]/u;
// The file kinds a codename-bearing path can end in.
const fileExtension =
  /^\.(?:md|rs|ts|tsx|mjs|js|json|toml|tsv|vue|snap|txt|ya?ml|sh|folio|dump)(?![A-Za-z0-9_-])/u;
const lowerChar = /[a-z]/u;
const letter = /[A-Za-z]/u;
const protectedSpans = new RegExp(
  [
    `${kept}(?![A-Za-z0-9_-])`,
    `docs%2F${NAME}(?![A-Za-z0-9_-])`,
    String.raw`docs\\\/${NAME}(?![A-Za-z0-9_-])`,
    // Links pinned to a commit or branch other than main show that tree as it was.
    String.raw`https?://github\.com/ubugeeei-prod/vize/(?:blob|tree|raw|commit|compare)/(?!main/)[^\s)"'\x60\]>]+`,
  ].join("|"),
  "gu",
);

const cased = (sample: string, lower: string) =>
  sample === sample.toUpperCase()
    ? lower.toUpperCase()
    : sample[0] === sample[0].toUpperCase()
      ? lower[0].toUpperCase() + lower.slice(1)
      : lower;

/** A level codename at `at` starts a word part: not inside `historicalComplete`. */
function startsPart(text: string, at: number): boolean {
  const before = text[at - 1];
  if (before === undefined || !letter.test(before)) return true;
  return lowerChar.test(before) && !lowerChar.test(text[at]);
}

/** Level codenames found in `text`, at word-part starts. */
export function levelHits(text: string): string[] {
  const hits: string[] = [];
  for (const match of text.matchAll(levelPattern)) {
    if (startsPart(text, match.index)) hits.push(match[0].toLowerCase());
  }
  return hits;
}

function levelReplacement(word: string, at: number, match: string): string {
  const name = levelNames[match.toLowerCase()];
  const whole = match.length === word.length;
  if (match === match.toUpperCase())
    return (name.kebab && word.includes("-") ? name.kebab : name.lower).toUpperCase();
  if (match[0] === match[0].toUpperCase()) return name.pascal;
  if (whole && name.word) return name.word;
  const kebab = word[at - 1] === "-" || word[at + match.length] === "-";
  return kebab && name.kebab ? name.kebab : name.lower;
}

export interface Scope {
  readonly program: boolean;
  readonly levels: boolean;
}
const everything: Scope = { program: true, levels: true };

export function renameWord(value: string, scope: Scope = everything): string {
  let out = value;
  if (scope.levels) {
    out = out.replace(levelPattern, (match, at: number) =>
      startsPart(out, at) ? levelReplacement(out, at, match) : match,
    );
  }
  if (!scope.program) return out;
  if (out.toLowerCase() === NAME) return cased(out, "levels");
  const replaced = out.replace(new RegExp(NAME, "giu"), (match) => cased(match, "level"));
  // `davinci-level-x` must not become `level-level-x`.
  return replaced.replace(/(level)[-_]?level/giu, "$1");
}

/** Each maximal word around an occurrence; scanning by index keeps long lines linear. */
function words(text: string, from: number, to: number): { start: number; end: number }[] {
  const found: { start: number; end: number }[] = [];
  anyPattern.lastIndex = from;
  let match: RegExpExecArray | null;
  while ((match = anyPattern.exec(text)) !== null && match.index < to) {
    const at = match.index;
    let start = at,
      end = at + match[0].length;
    while (start > from && wordChar.test(text[start - 1])) start--;
    while (end < to && wordChar.test(text[end])) end++;
    // An ANSI colour (`\x1B[32m`) ends at its `m`; the word starts after it.
    const sgr = /^\d+(?:;\d+)*m/u.exec(text.slice(start, at));
    if (sgr && text[start - 1] === "[") start += sgr[0].length;
    found.push({ start, end });
    anyPattern.lastIndex = end;
  }
  return found;
}

function replaceWords(
  text: string,
  from: number,
  to: number,
  scope: (start: number, end: number) => Scope,
): string {
  let out = "",
    cursor = from;
  for (const { start, end } of words(text, from, to)) {
    out += text.slice(cursor, start);
    out += renameWord(text.slice(start, end), scope(start, end));
    cursor = end;
  }
  return out + text.slice(cursor, to);
}

/** Captured output moves with its directory; only its bytes are history. */
export function renamePath(relative: string, limit: Scope = everything): string {
  const program = limit.program && !(relative === kept || relative.startsWith(kept + "/"));
  return replaceWords(relative, 0, relative.length, () => ({ program, levels: limit.levels }));
}

function resolvesToKept(file: string, text: string, start: number): boolean {
  const prefix = /(?:\.\.?\/)+$/u.exec(text.slice(Math.max(0, start - 64), start))?.[0];
  if (!prefix) return false;
  return path.posix.normalize(path.posix.join(path.posix.dirname(file), prefix + NAME)) === kept;
}

/** Fenced blocks and inline code spans of a Markdown text: there names mean current code. */
function codeRanges(text: string): [number, number][] {
  const ranges: [number, number][] = [];
  for (const match of text.matchAll(/^(`{3,}|~{3,})[^\n]*\n[\s\S]*?^\1[^\n]*$|`[^`\n]+`/gmu)) {
    ranges.push([match.index, match.index + match[0].length]);
  }
  return ranges;
}

/** Start offsets of the lines that say "codename": the alias form keeps level codenames. */
function aliasLines(text: string): [number, number][] {
  const ranges: [number, number][] = [];
  const lower = text.toLowerCase();
  for (let at = lower.indexOf("codename"); at !== -1; at = lower.indexOf("codename", at + 1)) {
    const start = lower.lastIndexOf("\n", at) + 1;
    const newline = lower.indexOf("\n", at);
    const end = newline === -1 ? lower.length : newline;
    ranges.push([start, end]);
    at = end;
  }
  return ranges;
}

/**
 * Rewrites one file's text. Markdown prose keeps the program name inside docs/davinci/; paths and
 * code spans name current code and follow. In Markdown under docs/ the level codenames stay as
 * aliases and historical names (published packages, old wire headers); only file paths follow
 * their moves. Other data under docs/ names current code and follows fully.
 */
export function rewriteText(file: string, text: string, limit: Scope = everything): string {
  anyPattern.lastIndex = 0;
  if (!anyPattern.test(text)) return text;
  const markdown = file.endsWith(".md");
  const keptProse = markdown && file.startsWith(kept + "/");
  const docs = markdown && file.startsWith("docs/");
  const code = keptProse ? codeRanges(text) : [];
  const alias = aliasLines(text);
  const within = (ranges: [number, number][], at: number) =>
    ranges.some(([start, end]) => at >= start && at < end);
  // A word next to a path separator, before a file extension or as one (`x-baseline.folio`)
  // names a file.
  const pathAt = (start: number, stop: number) =>
    text[start - 1] === "/" ||
    text[stop] === "/" ||
    fileExtension.test(text.slice(stop, stop + 8)) ||
    (text[start - 1] === "." &&
      wordChar.test(text[start - 2] ?? "") &&
      Object.hasOwn(levelNames, text.slice(start, stop).toLowerCase()));
  const proseAt = (start: number, stop: number) => !pathAt(start, stop) && !within(code, start);
  let out = "",
    cursor = 0;
  const free = (end: number) => {
    out += replaceWords(text, cursor, end, (start, stop) => ({
      program:
        limit.program && !resolvesToKept(file, text, start) && !(keptProse && proseAt(start, stop)),
      levels: limit.levels && !within(alias, start) && (!docs || pathAt(start, stop)),
    }));
  };
  for (const span of text.matchAll(protectedSpans)) {
    free(span.index);
    out += span[0];
    cursor = span.index + span[0].length;
  }
  free(text.length);
  return out;
}

interface Tracked {
  readonly path: string;
  readonly mode: string;
}

function tracked(cwd: string): Tracked[] {
  return execFileSync("git", ["ls-files", "-s", "-z"], {
    cwd,
    encoding: "utf8",
    maxBuffer: 1 << 28,
  })
    .split("\0")
    .filter(Boolean)
    .map((line) => {
      const [meta, file] = line.split("\t");
      return { path: file, mode: meta.split(" ")[0] };
    });
}

export function plannedMoves(paths: readonly string[]): Map<string, string> {
  const moves = new Map<string, string>();
  const targets = new Map<string, string>();
  const existing = new Set(paths);
  for (const file of paths) {
    const target = renamePath(file);
    if (target === file) continue;
    const clash = targets.get(target) ?? (existing.has(target) ? target : undefined);
    if (clash) throw new Error(`${file} and ${clash} both map to ${target}`);
    targets.set(target, file);
    moves.set(file, target);
  }
  return moves;
}

const textFile = (bytes: Buffer) => !bytes.subarray(0, 8000).includes(0);

function contentTargets(cwd: string): string[] {
  return tracked(cwd)
    .filter(({ mode }) => mode === "100644" || mode === "100755")
    .map(({ path: file }) => file)
    .filter((file) => file !== self && !gate.test(file) && !keepsBytes(file));
}

export function run(mode: string, cwd = root): string[] {
  if (mode === "--move") {
    const moves = plannedMoves(tracked(cwd).map(({ path: file }) => file));
    for (const [from, to] of moves) {
      fs.mkdirSync(path.join(cwd, path.dirname(to)), { recursive: true });
      execFileSync("git", ["mv", "-k", from, to], { cwd });
    }
    return [...moves.keys()];
  }
  if (mode !== "--check" && mode !== "--rewrite") {
    throw new Error("usage: rename-codenames.ts --check|--move|--rewrite");
  }
  const stale: string[] = [];
  const moves = plannedMoves(tracked(cwd).map(({ path: file }) => file));
  stale.push(...moves.keys());
  for (const file of contentTargets(cwd)) {
    const full = path.join(cwd, file);
    const bytes = fs.readFileSync(full);
    if (!textFile(bytes)) continue;
    const text = bytes.toString("utf8");
    const next = rewriteText(file, text);
    if (next === text) continue;
    stale.push(file);
    if (mode === "--rewrite") fs.writeFileSync(full, next);
  }
  if (mode === "--rewrite") {
    format(
      cwd,
      stale.filter((file) => !moves.has(file)),
    );
    // Ledgers whose rows or file names derive from moved paths are regenerated, not edited.
    for (const generator of generators) {
      execFileSync(process.execPath, [generator, "--write"], { cwd, stdio: "ignore" });
    }
    // Renamed packages move in the lockfile's name order; let cargo re-sort it.
    execFileSync("cargo", ["metadata", "--format-version", "1", "--offline"], {
      cwd,
      stdio: "ignore",
    });
  }
  return stale;
}

const generators = [
  "tools/support/compat/levels/v-on-corpus.ts",
  "tools/support/compat/levels/croquis-consumers.ts",
  "tools/support/compat/levels/sourcelocation-inventory.ts",
  "tools/support/compat/levels/consumer-migration-surfaces.ts",
];

/** Renamed spellings change line widths; reflow only the rewritten files. */
function format(cwd: string, files: string[]): void {
  const chunks = (list: string[]) =>
    Array.from({ length: Math.ceil(list.length / 200) }, (_, i) =>
      list.slice(i * 200, i * 200 + 200),
    );
  const rust = files.filter((file) => file.endsWith(".rs"));
  for (const chunk of chunks(rust)) {
    execFileSync(process.env.RUSTFMT ?? "rustfmt", ["--edition", "2024", ...chunk], { cwd });
  }
  const web = files.filter((file) => /\.(?:css|json|md|[cm]?[jt]sx?|vue|ya?ml)$/u.test(file));
  for (const chunk of chunks(web)) {
    execFileSync(path.join(cwd, "node_modules/.bin/vp"), ["fmt", ...chunk], {
      cwd,
      stdio: "ignore",
    });
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const mode = process.argv[2] ?? "";
  const stale = run(mode);
  process.stdout.write(`${stale.length} paths ${mode === "--check" ? "still named" : "changed"}\n`);
  if (mode === "--check" && stale.length > 0) process.exitCode = 1;
}
