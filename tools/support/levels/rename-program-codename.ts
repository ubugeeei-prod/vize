/**
 * Replay the removal of the program codename (maintainer decision 2026-11-03): every path and
 * line outside docs/davinci/ loses it. A whole word becomes `levels` (the level crates' home),
 * an embedded one becomes `level`, case preserved. On a conflict, take main and re-run.
 *   node tools/support/levels/rename-program-codename.ts --check|--move|--rewrite
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
const self = "tools/support/levels/rename-program-codename.ts";
// The completion gate must spell the names it bans; it moves but keeps its bytes.
const gate = /^tools\/commands\/[^/]+\/completion-gates\.ts$/u;
/** Captured output of a past run is history and keeps its bytes. */
export const history =
  /\.(?:log|jsonl|stderr|stdout)$|(?:^|\/)(?:fix-history|capture-[^/]+)\/|(?:^|\/)(?:original-[^/]*|observed-[^/]*\.json|[^/]*\.observation\.json)$/u;
const wordChar = /[A-Za-z0-9_-]/u;
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

export function renameWord(value: string): string {
  if (value.toLowerCase() === NAME) return cased(value, "levels");
  const replaced = value.replace(new RegExp(NAME, "giu"), (match) => cased(match, "level"));
  // `davinci-level-x` must not become `level-level-x`.
  return replaced.replace(/(level)[-_]?level/giu, "$1");
}

/** Each maximal word around an occurrence; scanning by indexOf keeps long lines linear. */
function words(text: string, from: number, to: number): { start: number; end: number }[] {
  const lower = text.toLowerCase();
  const found: { start: number; end: number }[] = [];
  let at = lower.indexOf(NAME, from);
  while (at !== -1 && at < to) {
    let start = at,
      end = at + NAME.length;
    while (start > from && wordChar.test(text[start - 1])) start--;
    while (end < to && wordChar.test(text[end])) end++;
    // An ANSI colour (`\x1B[32m`) ends at its `m`; the word starts after it.
    const sgr = /^\d+(?:;\d+)*m/u.exec(text.slice(start, at));
    if (sgr && text[start - 1] === "[") start += sgr[0].length;
    found.push({ start, end });
    at = lower.indexOf(NAME, end);
  }
  return found;
}

function replaceWords(
  text: string,
  from: number,
  to: number,
  keep: (start: number, end: number) => boolean,
): string {
  let out = "",
    cursor = from;
  for (const { start, end } of words(text, from, to)) {
    out += text.slice(cursor, start);
    const value = text.slice(start, end);
    out += keep(start, end) ? value : renameWord(value);
    cursor = end;
  }
  return out + text.slice(cursor, to);
}

export function renamePath(relative: string): string {
  if (relative === kept || relative.startsWith(kept + "/")) return relative;
  return replaceWords(relative, 0, relative.length, () => false);
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

/**
 * Rewrites one file's text. Inside docs/davinci/ the prose keeps the program name: a Markdown
 * page changes only paths and code spans, which name current code; data files change fully.
 */
export function rewriteText(file: string, text: string): string {
  if (!text.toLowerCase().includes(NAME)) return text;
  const prose = file.startsWith(kept + "/") && file.endsWith(".md");
  const code = prose ? codeRanges(text) : [];
  const inCode = (at: number) => code.some(([start, end]) => at >= start && at < end);
  let out = "",
    cursor = 0;
  const free = (end: number) => {
    out += replaceWords(
      text,
      cursor,
      end,
      (start, stop) =>
        resolvesToKept(file, text, start) ||
        (prose && text[start - 1] !== "/" && text[stop] !== "/" && !inCode(start)),
    );
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
    .filter((file) => file !== self && !gate.test(file) && !history.test(file));
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
    throw new Error("usage: rename-program-codename.ts --check|--move|--rewrite");
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
  }
  return stale;
}

const generators = ["tools/support/compat/levels/v-on-corpus.ts"];

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
