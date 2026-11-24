// AST identity for comment- and whitespace-only TypeScript, JavaScript and Vue
// changes (evidence class N0 of the readability program).
//
//   node tools/commands/levels/ast-identity.ts --base <rev> [--head <rev>] [--json]
//     [-- <pathspec>...]
//   node tools/commands/levels/ast-identity.ts --files <before> <after>
//
// Both sides are parsed with oxc (the `parseSync` that Vite+ ships, so no extra
// dependency) and the ESTree programs are compared with every position field
// dropped. Comments are not in the tree, but some comments are semantics, not
// prose: `@__PURE__` and `@__NO_SIDE_EFFECTS__` annotations, `@vite-ignore`,
// bundler magic comments, `sourceMappingURL`, TypeScript directives and
// references, and lint directives. Those are compared in order, so deleting
// one fails as `semantic-comment`. In a Vue SFC every `<script>` block is
// compared as a tree and everything outside the blocks (template, styles and
// their HTML comments, which the compiler may emit) must keep its bytes.
// Added, deleted and moved files fail, as does a file that does not parse.

import fs from "node:fs";

import { parseSync } from "vite";

import { isEntrypoint } from "../../support/command-runtime.ts";
import { vueScripts } from "../../support/readability/vue-scripts.ts";
import { changedPairs } from "../../support/readability/revision-pairs.ts";

export type AstVerdict =
  | "bytes-identical"
  | "identical"
  | "semantic-comment"
  | "template-changed"
  | "different"
  | "parse-error"
  | "added"
  | "deleted"
  | "moved";

export interface AstResult {
  readonly path: string;
  readonly verdict: AstVerdict;
  readonly detail?: string;
}

const scriptExtension = /\.(?:[cm]?[jt]s|[jt]sx|vue)$/u;
const positionKeys = new Set(["start", "end", "range", "loc"]);
const semanticComment =
  /[@#]__(?:PURE|NO_SIDE_EFFECTS)__|@vite-ignore|webpack[A-Z]\w*\s*:|vite-ignore|sourceMappingURL=|^\s*@ts-|^\/\s*<reference\b|\b(?:eslint|oxlint)-(?:disable|enable)|@jsx|@license|@preserve|^\s*!|\bistanbul ignore|\bc8 ignore|\bv8 ignore/u;

type Lang = "js" | "jsx" | "ts" | "tsx";

function langOf(path: string): Lang {
  if (path.endsWith(".tsx")) return "tsx";
  if (path.endsWith(".jsx")) return "jsx";
  return /\.[cm]?ts$/u.test(path) ? "ts" : "js";
}

interface ParsedScript {
  readonly tree: string;
  readonly semantic: readonly string[];
  readonly errors: number;
}

function parseScript(path: string, source: string, lang: Lang): ParsedScript {
  const result = parseSync(path, source, { lang, sourceType: "unambiguous" });
  const tree = JSON.stringify(result.program, (key, value: unknown) => {
    if (positionKeys.has(key)) return undefined;
    return typeof value === "bigint" ? String(value) : value;
  });
  const semantic: string[] = [];
  if (result.comments.some((comment) => semanticComment.test(comment.value))) {
    let end = 0;
    let offset = 0;
    for (const comment of result.comments) {
      const gap = source.slice(end, comment.start);
      offset += gap.replace(/\s/gu, "").length;
      end = comment.end;
      if (!semanticComment.test(comment.value)) continue;
      const next = /^\s*/u.exec(source.slice(end))![0];
      const lineSensitive = /@ts-|(?:eslint|oxlint)-|istanbul|[cv]8 ignore/u.test(comment.value);
      const nextLines = lineSensitive ? next.split(/\r\n|[\r\n\u2028\u2029]/u).length - 1 : 0;
      semantic.push(JSON.stringify([offset, nextLines, comment.value.trim()]));
    }
  }
  return { tree, semantic, errors: result.errors.length };
}

function vueParts(source: string) {
  const { blocks, errors } = vueScripts(source);
  const rest: string[] = [];
  let end = 0;
  const scripts = blocks.map((block) => {
    rest.push(source.slice(end, block.loc.start.offset));
    end = block.loc.end.offset;
    const lang: Lang =
      block.lang === "ts" || block.lang === "tsx" || block.lang === "jsx" ? block.lang : "js";
    return { lang, source: block.content };
  });
  rest.push(source.slice(end));
  return { scripts, rest, errors };
}

function compareScripts(path: string, before: ParsedScript, after: ParsedScript): AstResult {
  if (before.errors || after.errors)
    return { path, verdict: "parse-error", detail: `${before.errors}/${after.errors} errors` };
  if (before.tree !== after.tree) return { path, verdict: "different" };
  if (before.semantic.join("\n") !== after.semantic.join("\n"))
    return {
      path,
      verdict: "semantic-comment",
      detail: before.semantic.filter((value) => !after.semantic.includes(value)).join(" | "),
    };
  return { path, verdict: "identical" };
}

/** Compare one file's two sides. */
export function compareScriptFile(path: string, before?: string, after?: string): AstResult {
  if (before === undefined) return { path, verdict: "added" };
  if (after === undefined) return { path, verdict: "deleted" };
  if (before === after) return { path, verdict: "bytes-identical" };
  if (!path.endsWith(".vue")) {
    const lang = langOf(path);
    return compareScripts(path, parseScript(path, before, lang), parseScript(path, after, lang));
  }
  const left = vueParts(before);
  const right = vueParts(after);
  if (left.errors.length || right.errors.length) return { path, verdict: "parse-error" };
  if (JSON.stringify(left.rest) !== JSON.stringify(right.rest))
    return { path, verdict: "template-changed" };
  for (const [index, script] of left.scripts.entries()) {
    const other = right.scripts[index];
    const name = `${path}#script${index}`;
    const result = compareScripts(
      path,
      parseScript(name, script.source, script.lang),
      parseScript(name, other.source, other.lang),
    );
    if (result.verdict !== "identical") return result;
  }
  return { path, verdict: "identical" };
}

export function compareRange(
  root: string,
  base: string,
  head: string | undefined,
  paths: readonly string[] = [],
): AstResult[] {
  return changedPairs(root, base, head, (file) => scriptExtension.test(file), paths).map((pair) =>
    pair.basePath && pair.basePath !== pair.path
      ? { path: `${pair.basePath} -> ${pair.path}`, verdict: "moved" }
      : compareScriptFile(pair.path, pair.before, pair.after),
  );
}

export function isPassing(results: readonly AstResult[]): boolean {
  return results.every(
    (result) => result.verdict === "identical" || result.verdict === "bytes-identical",
  );
}

export function summarize(results: readonly AstResult[]): string {
  const counts = new Map<AstVerdict, number>();
  for (const result of results) counts.set(result.verdict, (counts.get(result.verdict) ?? 0) + 1);
  const lines = [
    `ast identity: ${results.length} changed script files; ` +
      [...counts].map(([verdict, count]) => `${count} ${verdict}`).join(", "),
  ];
  for (const result of results)
    if (!isPassing([result]))
      lines.push(`  ${result.verdict} ${result.path}${result.detail ? ` (${result.detail})` : ""}`);
  return lines.join("\n");
}

function main(argv: readonly string[]): number {
  const option = (name: string) => {
    const at = argv.indexOf(name);
    return at >= 0 ? argv[at + 1] : undefined;
  };
  let results: AstResult[];
  const filesAt = argv.indexOf("--files");
  if (filesAt >= 0) {
    const [before, after] = argv.slice(filesAt + 1, filesAt + 3);
    results = [
      compareScriptFile(after, fs.readFileSync(before, "utf8"), fs.readFileSync(after, "utf8")),
    ];
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
  return isPassing(results) ? 0 : 1;
}

if (isEntrypoint(import.meta.url)) {
  try {
    process.exitCode = main(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
