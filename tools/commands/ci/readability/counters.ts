// Per-file readability counters for the shrink-only ratchet
// (`readability-ratchet.ts`). Each counter is a heuristic from the readability
// inventory, tuned so the same bytes always give the same count; the ratchet
// only needs counts that cannot rise unnoticed, not a verdict on each line.

import { lexRust } from "../../../support/readability/rust-tokens.ts";
import { maskScript, vueScriptSource } from "../../../support/readability/script-comments.ts";

export const counterNames = [
  "lines",
  "long-functions",
  "issue-comments",
  "issue-names",
  "restating-comments",
  "mixed-imports",
  "hardcoded-limits",
] as const;

export type CounterName = (typeof counterNames)[number];

/**
 * Counters whose rise fails `check`. These carry rules already decided: the
 * 500-line file limit, and no issue numbers in comments, file names or test
 * names (maintainer decision, 2026-11-24). The other counters are recorded and
 * reported, and join this set when the house source style is adopted.
 */
export const enforcedCounters: ReadonlySet<CounterName> = new Set([
  "lines",
  "issue-comments",
  "issue-names",
]);

export type Counts = Record<CounterName, number>;
export type Language = "rust" | "script" | "vue";

/** A file over this many lines is oversized (the `source-file-lengths` limit). */
export const fileLineLimit = 500;
/** A function over this many lines is long (the house style's limit for new code). */
export const functionLineLimit = 120;

const issueReference = /(?<![\w&])#\d{3,6}\b|\b(?:issue|PR|pull)\s*#?\d{3,6}\b|\bgh-\d{3,}\b/iu;
const liveTodo = /\b(?:TODO|FIXME)\(#\d+\)/u;
const keepMarker =
  /^\s*(?:SAFETY|INVARIANT|Invariant|PANIC|TODO|FIXME|XXX|HACK|eslint|oxlint|@ts-|prettier|rustfmt|clippy|#region|#endregion|shellcheck|@vite-ignore|istanbul|c8|v8)/u;
const issueFileName = /[-_]\d{3,6}(?=[-_.]|$)/u;
const rustIssueTest = /(?:^|_)(?:issue_?|pr_?|gh_?)?\d{3,6}(?:_|$)/u;
const scriptTestTitle = /\b(?:test|it|describe)(?:\.\w+)?\s*\(\s*(["'`])((?:\\.|(?!\1).)*)\1/gu;
const stopWords = new Set(
  "the a an of to in on for is are be this that it its and or with by as at from into we our".split(
    " ",
  ),
);
const wordPattern = /[A-Za-z][a-z]+|[A-Z]+(?![a-z])|\d+/gu;
const rustFunction =
  /^\s*(?:pub(?:\([\w:\s]+\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?(?:extern\s+"\w+"\s+)?fn\s+\w+/u;
const scriptFunction =
  /^\s*(?:export\s+)?(?:default\s+)?(?:async\s+)?function\s*\*?\s*\w+|^\s*(?:export\s+)?const\s+\w+\s*(?::[^=]+)?=\s*(?:async\s+)?(?:\([^)]*\)|\w+)\s*(?::[^=]+)?=>/u;
const numericLiteral = /(?<![\w.#$])(?:0x[0-9a-fA-F_]+|\d[\d_]*(?:\.\d+)?(?:e[+-]?\d+)?)\w*/gu;
const rustNamedValue = /^(?:pub(?:\([\w:\s]+\))?\s+)?(?:const|static)\s/u;
const scriptNamedValue =
  /^(?:export\s+)?const\s+[\w$]+\s*(?::[^=]+)?=\s*-?(?:0x[\da-f_]+|[\d_.]+)\s*;?$/iu;
// Literals from this value up are limits that deserve a name, except round
// powers of ten, which usually scale units (percent, milliseconds).
const smallestLimit = 100;
const roundNumbers = new Set([smallestLimit, smallestLimit * 10]);

export function emptyCounts(): Counts {
  return Object.fromEntries(counterNames.map((name) => [name, 0])) as Counts;
}

export function languageOf(path: string): Language | undefined {
  if (path.endsWith(".rs")) return "rust";
  if (path.endsWith(".vue")) return "vue";
  if (/\.(?:[cm]?[jt]s|[jt]sx)$/u.test(path)) return "script";
  return undefined;
}

/** Test code is exempt from the hardcoded-limit counter: its numbers are oracles. */
export function isTestPath(path: string): boolean {
  return (
    path.startsWith("tests/") ||
    /\/(?:tests?|__tests__|e2e|benches)\//u.test(path) ||
    /(?:^|[/_.-])tests?(?:[/_.-]|$)|\.spec\.[cm]?[jt]sx?$/u.test(path.split("/").at(-1) ?? "")
  );
}

const testModule =
  /#\[cfg\((?:all\()?test\b[^\]]*\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;/gu;

/**
 * Paths a Rust file declares as `#[cfg(test)] mod name;`: `name.rs` and
 * everything under `name/`, relative to the file's module directory.
 */
export function rustTestModules(path: string, source: string): string[] {
  const directory = path.slice(0, path.lastIndexOf("/") + 1);
  const stem = path.slice(directory.length, -3);
  const base = ["mod", "lib", "main"].includes(stem) ? directory : `${directory}${stem}/`;
  return [...source.matchAll(testModule)].flatMap((match) => [
    `${base}${match[1]}.rs`,
    `${base}${match[1]}/`,
  ]);
}

export function lineCount(content: string): number {
  if (content.length === 0) return 0;
  return (content.endsWith("\n") ? content.slice(0, -1) : content).split("\n").length;
}

interface CommentLine {
  readonly line: number;
  readonly kind: "line" | "block" | "doc";
  readonly text: string;
}

function words(text: string): string[] {
  return (text.match(wordPattern) ?? []).map((word) => word.toLowerCase());
}

function stem(word: string): string {
  for (const suffix of ["ing", "ies", "es", "ed", "s"])
    if (word.endsWith(suffix) && word.length > suffix.length + 2)
      return word.slice(0, -suffix.length);
  return word;
}

/** A short comment whose content words almost all appear in the next code line. */
function restates(comment: string, nextCode: string | undefined): boolean {
  const content = words(comment)
    .filter((word) => !stopWords.has(word))
    .map(stem);
  if (!content.length || content.length > 8 || !nextCode) return false;
  const code = new Set(words(nextCode).map(stem));
  return content.filter((word) => code.has(word)).length / content.length >= 0.75;
}

function commentCounts(comments: readonly CommentLine[], codeLines: readonly string[]) {
  let issues = 0;
  let restating = 0;
  for (const comment of comments)
    if (issueReference.test(comment.text) && !liveTodo.test(comment.text)) issues++;
  const blocks: CommentLine[][] = [];
  for (const comment of comments) {
    const last = blocks.at(-1)?.at(-1);
    if (last && last.line + 1 === comment.line && last.kind === comment.kind)
      blocks.at(-1)!.push(comment);
    else blocks.push([comment]);
  }
  for (const block of blocks) {
    const text = block
      .map((comment) => comment.text)
      .join(" ")
      .trim();
    if (!text || block.length > 2 || keepMarker.test(text)) continue;
    // `codeLines` is 0-based, so index `line` is the line right after the block.
    const next = codeLines[block.at(-1)!.line];
    if (restates(text, next?.trim() ? next : undefined)) restating += block.length;
  }
  return { issues, restating };
}

/** Functions longer than the limit, by brace matching over masked code lines. */
function longFunctions(codeLines: readonly string[], header: RegExp): number {
  let count = 0;
  let braces = 0;
  let parens = 0;
  let pending: { line: number; braces: number; parens: number } | undefined;
  const open: (number | undefined)[] = [];
  for (const [index, text] of codeLines.entries()) {
    if (!pending && header.test(text)) pending = { line: index, braces, parens };
    for (const char of text) {
      if (char === "(") parens++;
      else if (char === ")") parens--;
      else if (char === ";" && pending && braces === pending.braces && parens === pending.parens)
        pending = undefined;
      else if (char === "{") {
        const starts =
          pending && braces === pending.braces && parens === pending.parens
            ? pending.line
            : undefined;
        if (starts !== undefined) pending = undefined;
        open.push(starts);
        braces++;
      } else if (char === "}") {
        braces--;
        const starts = open.pop();
        if (starts !== undefined && index - starts + 1 > functionLineLimit) count++;
      }
    }
  }
  return count;
}

type ImportKind = 0 | 1 | 2;

/** 1 when the leading import groups mix kinds or put them out of order. */
function mixedImports(groups: readonly ImportKind[][]): number {
  const kinds = new Set(groups.flat());
  if (kinds.size < 2) return 0;
  const flat = groups.flat();
  const ordered = flat.every((kind, index) => index === 0 || flat[index - 1] <= kind);
  return groups.every((group) => new Set(group).size === 1) && ordered ? 0 : 1;
}

function rustImportGroups(lines: readonly string[]): ImportKind[][] {
  const modules = new Set(
    lines.flatMap((text) => /^(?:pub(?:\([\w:\s]+\))?\s+)?mod\s+(\w+)/u.exec(text)?.[1] ?? []),
  );
  const groups: ImportKind[][] = [];
  let current: ImportKind[] = [];
  let seen = false;
  for (const text of lines) {
    const use = /^(?:pub(?:\([\w:\s]+\))?\s+)?use\s+(?:::)?(\w+)/u.exec(text);
    if (use) {
      seen = true;
      const root = use[1];
      current.push(
        ["std", "core", "alloc"].includes(root)
          ? 0
          : ["crate", "self", "super"].includes(root) || modules.has(root)
            ? 2
            : 1,
      );
      continue;
    }
    if (!seen) continue;
    if (!text.trim()) {
      if (current.length) groups.push(current);
      current = [];
      continue;
    }
    if (/^(?:\s|\}|#\[)/u.test(text)) continue;
    break;
  }
  if (current.length) groups.push(current);
  return groups;
}

function scriptImportGroups(lines: readonly string[]): ImportKind[][] {
  const groups: ImportKind[][] = [];
  let current: ImportKind[] = [];
  let started = false;
  for (let index = 0; index < lines.length; index++) {
    const text = lines[index];
    if (text.startsWith("import ")) {
      started = true;
      let statement = text;
      while (!/["'][^"']+["'];?\s*$/u.test(statement) && index + 1 < lines.length)
        statement += lines[++index];
      const specifier = /["']([^"']+)["'];?\s*$/u.exec(statement)?.[1] ?? "";
      current.push(specifier.startsWith("node:") ? 0 : specifier.startsWith(".") ? 2 : 1);
      continue;
    }
    if (!started) continue;
    if (!text.trim()) {
      if (current.length) groups.push(current);
      current = [];
    } else if (!/^(?:\/\/|\/\*| \*)/u.test(text)) break;
  }
  if (current.length) groups.push(current);
  return groups;
}

function isLimitLiteral(literal: string): boolean {
  const digits = literal.replace(/_/gu, "");
  const hex = /^0x([0-9a-f]+)/iu.exec(digits);
  if (hex) return Number.parseInt(hex[1], 16) >= smallestLimit;
  const integer = /^(\d+)(?:[iu](?:8|16|32|64|128|size))?$/u.exec(digits);
  if (!integer) return false;
  const value = Number(integer[1]);
  return value >= smallestLimit && !roundNumbers.has(value);
}

function hardcodedLimits(codeLines: readonly string[], named: RegExp): number {
  let count = 0;
  for (const text of codeLines) {
    if (text.startsWith("#[cfg(test)]")) break;
    const trimmed = text.trim();
    if (!trimmed || named.test(trimmed) || /^(?:#!?\[|@)/u.test(trimmed)) continue;
    for (const match of trimmed.matchAll(numericLiteral)) if (isLimitLiteral(match[0])) count++;
  }
  return count;
}

function testNameIssues(language: Language, source: string): number {
  if (language === "rust") {
    let count = 0;
    for (const match of source.matchAll(
      /#\[(?:tokio::)?test\b[^\]]*\](?:\s*#\[[^\]]*\])*\s*(?:async\s+)?fn\s+(\w+)/gu,
    ))
      if (rustIssueTest.test(match[1])) count++;
    return count;
  }
  let count = 0;
  for (const match of source.matchAll(scriptTestTitle)) if (issueReference.test(match[2])) count++;
  return count;
}

/** Count every readability counter for one file. */
export function countFile(
  path: string,
  source: string,
  options: { readonly lines: boolean; readonly test?: boolean },
): Counts {
  const counts = emptyCounts();
  const language = languageOf(path);
  if (!language) return counts;
  const lines = lineCount(source);
  if (options.lines && lines > fileLineLimit) counts.lines = lines;
  if (issueFileName.test(path.split("/").at(-1) ?? "")) counts["issue-names"] = 1;
  counts["issue-names"] += testNameIssues(language, source);
  let comments: CommentLine[];
  let code: string;
  if (language === "rust") {
    const lexed = lexRust(source);
    const masked = source.split("");
    const blank = (start: number, end: number) => {
      for (let at = start; at < end; at++) if (masked[at] !== "\n") masked[at] = " ";
    };
    for (const comment of lexed.comments) blank(comment.start, comment.end);
    for (const token of lexed.tokens)
      if (token.kind === "literal" && !/^\d/u.test(token.text)) blank(token.start, token.end);
    code = masked.join("");
    comments = lexed.comments.flatMap((comment) =>
      comment.lines.map((text, offset) => ({
        line: comment.line + offset,
        kind: comment.kind,
        text: text.replace(/^\s*\*?/u, "").trim(),
      })),
    );
  } else {
    const masked = maskScript(language === "vue" ? vueScriptSource(source) : source);
    code = masked.code;
    comments = masked.comments.flatMap((comment) =>
      comment.lines.map((text, offset) => ({
        line: comment.line + offset,
        kind: comment.kind,
        text: text.replace(/^\s*\*?/u, "").trim(),
      })),
    );
  }
  const codeLines = code.split("\n");
  const commentResult = commentCounts(comments, codeLines);
  counts["issue-comments"] = commentResult.issues;
  counts["restating-comments"] = commentResult.restating;
  counts["long-functions"] = longFunctions(
    codeLines,
    language === "rust" ? rustFunction : scriptFunction,
  );
  const rawLines = (language === "vue" ? vueScriptSource(source) : source).split("\n");
  counts["mixed-imports"] = mixedImports(
    language === "rust" ? rustImportGroups(rawLines) : scriptImportGroups(rawLines),
  );
  if (!options.test && !isTestPath(path))
    counts["hardcoded-limits"] = hardcodedLimits(
      codeLines,
      language === "rust" ? rustNamedValue : scriptNamedValue,
    );
  return counts;
}
