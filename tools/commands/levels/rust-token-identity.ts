// Rust token identity for comment- and whitespace-only changes (evidence
// class N0 of the readability program).
//
//   node tools/commands/levels/rust-token-identity.ts --base <rev> [--head <rev>]
//     [--strict-docs] [--json] [-- <pathspec>...]
//   node tools/commands/levels/rust-token-identity.ts --files <before.rs> <after.rs>
//
// Each changed `.rs` file is lexed on both sides (no compilation) and its token
// stream compared with spans dropped. Plain comments are not tokens, so
// deleting or rewording one keeps a file `identical`. Doc comments become
// `#[doc]` attributes, so a doc-only change is reported as `doc-only`. It fails
// when the item is product text (clap help, napi typings, wasm-bindgen docs),
// and with `--strict-docs` always. A file that reads `line!()` or `column!()`
// fails as `position-sensitive` when those calls move: their values are code.
// Reordering `use` items inside a run of imports is `use-reordered`, which
// passes: import order has no meaning, so regrouping imports is N0 too.
// Added, deleted and moved files are not comment edits, so they fail too. Without
// `--head` the head is the working tree (stage new files with `git add -N`).
//
// Token identity proves only that the source is the same program. Panic sites
// still embed their line and column, so pair this with
// `ir-identity-locations.ts`, which compares the optimized IR with those
// numbers normalized.

import fs from "node:fs";

import { isEntrypoint } from "../../support/command-runtime.ts";
import { changedPairs, type FilePair } from "../../support/readability/revision-pairs.ts";
import { lexRust, tokenKey, type RustToken } from "../../support/readability/rust-tokens.ts";

export type TokenVerdict =
  | "bytes-identical"
  | "identical"
  | "use-reordered"
  | "doc-only"
  | "doc-product-text"
  | "position-sensitive"
  | "different"
  | "added"
  | "deleted"
  | "moved";

export interface TokenResult {
  readonly path: string;
  readonly verdict: TokenVerdict;
  /** First differing token, as `line:column text` on each side. */
  readonly first?: { readonly before: string; readonly after: string };
  readonly tokens: number;
  readonly removedComments: number;
}

const productTextMarkers =
  /#\s*\[\s*(?:cfg_attr\s*\([^\]]*?,\s*)?(?:derive\s*\([^)]*\b(?:Parser|Args|Subcommand|ValueEnum)\b|(?:[\w:]+::)?(?:napi|wasm_bindgen)\b|(?:command|arg|clap)\s*\()/u;
const failing = new Set<TokenVerdict>([
  "doc-product-text",
  "position-sensitive",
  "different",
  "added",
  "deleted",
  "moved",
]);

function firstDifference(left: readonly RustToken[], right: readonly RustToken[]) {
  const length = Math.max(left.length, right.length);
  for (let index = 0; index < length; index++) {
    const a = left[index];
    const b = right[index];
    if (a && b && tokenKey(a) === tokenKey(b)) continue;
    const show = (token?: RustToken) =>
      token ? `${token.line}:${token.column} ${token.text.slice(0, 60)}` : "<end of file>";
    return { before: show(a), after: show(b) };
  }
  return undefined;
}

/** The end (exclusive) of a `use` item starting at `start`, or -1. */
function useItemEnd(tokens: readonly RustToken[], start: number): number {
  let at = start;
  while (tokens[at]?.text === "#" && tokens[at + 1]?.text === "[") {
    at++;
    let depth = 0;
    do {
      if (tokens[at].text === "[") depth++;
      else if (tokens[at].text === "]") depth--;
      at++;
    } while (at < tokens.length && depth > 0);
  }
  if (tokens[at]?.text === "pub") {
    at++;
    if (tokens[at]?.text === "(") while (at < tokens.length && tokens[at++].text !== ")");
  }
  if (tokens[at]?.kind !== "ident" || tokens[at].text !== "use") return -1;
  let depth = 0;
  for (; at < tokens.length; at++) {
    const text = tokens[at].text;
    if (text === "{") depth++;
    else if (text === "}") depth--;
    else if (text === ";" && depth === 0) return at + 1;
  }
  return -1;
}

/**
 * Sort each run of consecutive `use` items (with their attributes). Import
 * order has no meaning in Rust, so regrouping imports keeps the program.
 */
export function sortUseRuns(tokens: readonly RustToken[]): RustToken[] {
  const out: RustToken[] = [];
  for (let index = 0; index < tokens.length;) {
    // An item starts the file or follows an item end, an attribute or a doc comment.
    const boundary =
      index === 0 ||
      tokens[index - 1].kind === "doc" ||
      [";", "{", "}", "]"].includes(tokens[index - 1].text);
    let end = boundary ? useItemEnd(tokens, index) : -1;
    if (end < 0) {
      out.push(tokens[index++]);
      continue;
    }
    const items: RustToken[][] = [];
    while (end > 0) {
      items.push(tokens.slice(index, end));
      index = end;
      end = useItemEnd(tokens, index);
    }
    const key = (item: readonly RustToken[]) => item.map(tokenKey).join(" ");
    items.sort((left, right) => (key(left) < key(right) ? -1 : key(left) > key(right) ? 1 : 0));
    for (const item of items) out.push(...item);
  }
  return out;
}

function positionCalls(tokens: readonly RustToken[]): string[] {
  const calls: string[] = [];
  for (let index = 0; index + 1 < tokens.length; index++) {
    const token = tokens[index];
    if (token.kind === "ident" && (token.text === "line" || token.text === "column"))
      if (tokens[index + 1].text === "!" && ["(", "{", "["].includes(tokens[index + 2]?.text))
        calls.push(`${token.line}:${token.column}`);
  }
  return calls;
}

/** Compare one file's two sides. */
export function compareRust(path: string, before?: string, after?: string): TokenResult {
  if (before === undefined || after === undefined)
    return {
      path,
      verdict: before === undefined ? "added" : "deleted",
      tokens: 0,
      removedComments: 0,
    };
  const left = lexRust(before);
  const right = lexRust(after);
  const removedComments = Math.max(0, left.comments.length - right.comments.length);
  const base = { path, tokens: right.tokens.length, removedComments };
  if (before === after) return { ...base, verdict: "bytes-identical" };
  const withoutDocs = (tokens: readonly RustToken[]) =>
    tokens.filter((token) => token.kind !== "doc");
  const code = [withoutDocs(left.tokens), withoutDocs(right.tokens)] as const;
  const codeDifference = firstDifference(...code);
  const reordered = codeDifference !== undefined;
  if (reordered && firstDifference(sortUseRuns(code[0]), sortUseRuns(code[1])))
    return { ...base, verdict: "different", first: codeDifference };
  const leftCalls = positionCalls(left.tokens);
  if (leftCalls.length && leftCalls.join() !== positionCalls(right.tokens).join())
    return { ...base, verdict: "position-sensitive" };
  const docDifference = reordered
    ? firstDifference(sortUseRuns(left.tokens), sortUseRuns(right.tokens))
    : firstDifference(left.tokens, right.tokens);
  if (!docDifference) return { ...base, verdict: reordered ? "use-reordered" : "identical" };
  const productText = productTextMarkers.test(before) || productTextMarkers.test(after);
  return { ...base, verdict: productText ? "doc-product-text" : "doc-only", first: docDifference };
}

export function compareRange(
  root: string,
  base: string,
  head: string | undefined,
  paths: readonly string[] = [],
): TokenResult[] {
  const pairs: FilePair[] = changedPairs(root, base, head, (file) => file.endsWith(".rs"), paths);
  // A rename changes module paths and panic-location file strings: class N1, not N0.
  return pairs.map((pair) =>
    pair.basePath && pair.basePath !== pair.path
      ? {
          path: `${pair.basePath} -> ${pair.path}`,
          verdict: "moved",
          tokens: 0,
          removedComments: 0,
        }
      : compareRust(pair.path, pair.before, pair.after),
  );
}

export function isPassing(results: readonly TokenResult[], strictDocs = false): boolean {
  return results.every(
    (result) => !failing.has(result.verdict) && !(strictDocs && result.verdict === "doc-only"),
  );
}

export function summarize(results: readonly TokenResult[]): string {
  const counts = new Map<TokenVerdict, number>();
  for (const result of results) counts.set(result.verdict, (counts.get(result.verdict) ?? 0) + 1);
  const removed = results.reduce((total, result) => total + result.removedComments, 0);
  const lines = [
    `rust token identity: ${results.length} changed .rs files; ` +
      [...counts].map(([verdict, count]) => `${count} ${verdict}`).join(", ") +
      `; ${removed} comment(s) removed`,
  ];
  for (const result of results)
    if (result.verdict !== "identical" && result.verdict !== "bytes-identical")
      lines.push(
        `  ${result.verdict} ${result.path}` +
          (result.first ? ` (base ${result.first.before} | head ${result.first.after})` : ""),
      );
  return lines.join("\n");
}

function main(argv: readonly string[]): number {
  const option = (name: string) => {
    const at = argv.indexOf(name);
    return at >= 0 ? argv[at + 1] : undefined;
  };
  const strictDocs = argv.includes("--strict-docs");
  let results: TokenResult[];
  const filesAt = argv.indexOf("--files");
  if (filesAt >= 0) {
    const [before, after] = argv.slice(filesAt + 1, filesAt + 3);
    results = [compareRust(after, fs.readFileSync(before, "utf8"), fs.readFileSync(after, "utf8"))];
  } else {
    const base = option("--base");
    if (!base) throw new Error("--base <rev> or --files <before> <after> is required");
    const separator = argv.indexOf("--");
    results = compareRange(
      process.cwd(),
      base,
      option("--head"),
      separator >= 0 ? argv.slice(separator + 1) : [],
    );
  }
  console.log(argv.includes("--json") ? JSON.stringify(results, null, 2) : summarize(results));
  return isPassing(results, strictDocs) ? 0 : 1;
}

if (isEntrypoint(import.meta.url)) {
  try {
    process.exitCode = main(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
