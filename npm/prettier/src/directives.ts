import type { ParserOptions } from "prettier";

interface HtmlNode {
  kind: string;
  name?: string;
  value?: string;
  children?: HtmlNode[];
}

const ignoreDirective = /^prettier-ignore(?:\b|-)/u;

export async function assertSupportedDirectives(
  text: string,
  options: ParserOptions,
): Promise<void> {
  if (!text.includes("prettier-ignore")) return;
  // This rare validation uses the host's parser, never this plugin's parser.
  // Script strings and template attributes containing comment-like text remain data.
  const { parsers } = await import("prettier/plugins/html");
  const ast = (await parsers.vue.parse(text, { ...options, parser: "vue" })) as HtmlNode;
  function visit(node: HtmlNode): void {
    if (node.kind === "comment" && ignoreDirective.test(node.value?.trim() ?? "")) {
      throw new Error(
        "prettier-plugin-vize does not support HTML node/attribute/range prettier-ignore directives. Use .prettierignore or a checked @noprettier file pragma.",
      );
    }
    if (node.kind === "element" && node.name === "style") {
      for (const child of node.children ?? []) {
        if (child.kind === "text" && hasCssIgnoreComment(child.value ?? "")) {
          throw new Error(
            "prettier-plugin-vize does not support CSS prettier-ignore regions. Use .prettierignore or a checked @noprettier file pragma.",
          );
        }
      }
    }
    for (const child of node.children ?? []) visit(child);
  }
  visit(ast);
}

function hasCssIgnoreComment(css: string): boolean {
  let quote = "";
  for (let index = 0; index < css.length; index++) {
    const char = css[index];
    if (char === "\\") {
      index++;
      continue;
    }
    if (quote) {
      if (char === quote) quote = "";
      continue;
    }
    if (char === "'" || char === '"') {
      quote = char;
      continue;
    }
    if (char === "/" && css[index + 1] === "*") {
      const end = css.indexOf("*/", index + 2);
      if (end < 0) return false;
      if (ignoreDirective.test(css.slice(index + 2, end).trim())) return true;
      index = end + 1;
    }
  }
  return false;
}

function firstComment(text: string): string {
  return /^\s*<!--([\s\S]*?)-->/u.exec(text)?.[1] ?? "";
}

export const hasPragma = (text: string): boolean =>
  /@(?:format|prettier)\b/u.test(firstComment(text));
export const hasIgnorePragma = (text: string): boolean =>
  /@(?:noformat|noprettier)\b/u.test(firstComment(text));
