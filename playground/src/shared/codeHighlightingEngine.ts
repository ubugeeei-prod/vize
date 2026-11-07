import type { CodeHighlightLanguage, ThemedCodeToken } from "./codeHighlighting";

type TokenKind =
  | "attribute"
  | "comment"
  | "keyword"
  | "number"
  | "plain"
  | "punctuation"
  | "string"
  | "tag"
  | "type";

type LexerState = {
  blockComment: boolean;
  htmlComment: boolean;
  htmlTag: boolean;
  quote: '"' | "'" | "`" | null;
};

const colors: Record<Exclude<TokenKind, "plain">, readonly [dark: string, light: string]> = {
  attribute: ["#D8B4FE", "#5A3D86"],
  comment: ["#A39B8F", "#59554F"],
  keyword: ["#F2B8A0", "#7A2F15"],
  number: ["#F1BC78", "#73470D"],
  punctuation: ["#B8AFA2", "#5C5750"],
  string: ["#A7D8A8", "#1B6A38"],
  tag: ["#8ECDF0", "#005F80"],
  type: ["#C4C8FF", "#3D4B80"],
};

const scriptKeywords = new Set([
  "as",
  "async",
  "await",
  "break",
  "case",
  "catch",
  "class",
  "const",
  "continue",
  "debugger",
  "declare",
  "default",
  "delete",
  "do",
  "else",
  "enum",
  "export",
  "extends",
  "finally",
  "for",
  "from",
  "function",
  "if",
  "implements",
  "import",
  "in",
  "instanceof",
  "interface",
  "keyof",
  "let",
  "new",
  "of",
  "private",
  "protected",
  "public",
  "readonly",
  "return",
  "satisfies",
  "static",
  "switch",
  "throw",
  "try",
  "type",
  "typeof",
  "var",
  "while",
  "with",
  "yield",
]);

const scriptTypes = new Set([
  "any",
  "bigint",
  "boolean",
  "false",
  "never",
  "null",
  "number",
  "object",
  "string",
  "symbol",
  "true",
  "undefined",
  "unknown",
  "void",
]);

function append(tokens: ThemedCodeToken[], content: string, kind: TokenKind): void {
  if (content === "") return;
  const palette = kind === "plain" ? undefined : colors[kind];
  const darkColor = palette?.[0];
  const lightColor = palette?.[1];
  const previous = tokens.at(-1);
  if (previous && previous.darkColor === darkColor && previous.lightColor === lightColor) {
    previous.content += content;
    return;
  }
  tokens.push({ content, darkColor, lightColor });
}

function quotedEnd(line: string, start: number, quote: '"' | "'" | "`"): number {
  let escaped = false;
  for (let index = start; index < line.length; index += 1) {
    const character = line[index];
    if (escaped) {
      escaped = false;
    } else if (character === "\\") {
      escaped = true;
    } else if (character === quote) {
      return index + 1;
    }
  }
  return line.length;
}

function tokenizeScriptLine(line: string, state: LexerState): ThemedCodeToken[] {
  const tokens: ThemedCodeToken[] = [];
  let index = 0;
  while (index < line.length) {
    if (state.blockComment) {
      const end = line.indexOf("*/", index);
      const next = end < 0 ? line.length : end + 2;
      append(tokens, line.slice(index, next), "comment");
      state.blockComment = end < 0;
      index = next;
      continue;
    }
    if (state.quote) {
      const end = quotedEnd(line, index, state.quote);
      append(tokens, line.slice(index, end), "string");
      if (line[end - 1] === state.quote) state.quote = null;
      index = end;
      continue;
    }
    if (line.startsWith("//", index)) {
      append(tokens, line.slice(index), "comment");
      break;
    }
    if (line.startsWith("/*", index)) {
      const end = line.indexOf("*/", index + 2);
      const next = end < 0 ? line.length : end + 2;
      append(tokens, line.slice(index, next), "comment");
      state.blockComment = end < 0;
      index = next;
      continue;
    }
    const character = line[index];
    if (character === '"' || character === "'" || character === "`") {
      const end = quotedEnd(line, index + 1, character);
      append(tokens, line.slice(index, end), "string");
      if (line[end - 1] !== character) state.quote = character;
      index = end;
      continue;
    }
    const identifier = line.slice(index).match(/^[A-Za-z_$][\w$]*/u)?.[0];
    if (identifier) {
      append(
        tokens,
        identifier,
        scriptKeywords.has(identifier) ? "keyword" : scriptTypes.has(identifier) ? "type" : "plain",
      );
      index += identifier.length;
      continue;
    }
    const number = line
      .slice(index)
      .match(/^(?:0[xob][\da-f]+|\d+(?:\.\d+)?(?:e[+-]?\d+)?n?)/iu)?.[0];
    if (number) {
      append(tokens, number, "number");
      index += number.length;
      continue;
    }
    const punctuation = line.slice(index).match(/^[{}[\]();,.?:<>+*/%=&|!~-]+/u)?.[0];
    if (punctuation) {
      append(tokens, punctuation, "punctuation");
      index += punctuation.length;
      continue;
    }
    const plain = line.slice(index).match(/^\s+/u)?.[0] ?? character;
    append(tokens, plain, "plain");
    index += plain.length;
  }
  return tokens;
}

function tokenizeHtmlLine(line: string, state: LexerState): ThemedCodeToken[] {
  const tokens: ThemedCodeToken[] = [];
  let index = 0;
  while (index < line.length) {
    if (state.htmlComment) {
      const end = line.indexOf("-->", index);
      const next = end < 0 ? line.length : end + 3;
      append(tokens, line.slice(index, next), "comment");
      state.htmlComment = end < 0;
      index = next;
      continue;
    }
    if (line.startsWith("<!--", index)) {
      const end = line.indexOf("-->", index + 4);
      const next = end < 0 ? line.length : end + 3;
      append(tokens, line.slice(index, next), "comment");
      state.htmlComment = end < 0;
      index = next;
      continue;
    }
    const character = line[index];
    if (character === "<") {
      const prefix = line.startsWith("</", index) ? "</" : "<";
      append(tokens, prefix, "punctuation");
      index += prefix.length;
      const tag = line.slice(index).match(/^[\w:-]+/u)?.[0];
      if (tag) {
        append(tokens, tag, "tag");
        index += tag.length;
      }
      state.htmlTag = true;
      continue;
    }
    if (state.htmlTag && (line.startsWith("/>", index) || character === ">")) {
      const suffix = line.startsWith("/>", index) ? "/>" : ">";
      append(tokens, suffix, "punctuation");
      index += suffix.length;
      state.htmlTag = false;
      continue;
    }
    if (state.htmlTag && (character === '"' || character === "'")) {
      const end = quotedEnd(line, index + 1, character);
      append(tokens, line.slice(index, end), "string");
      index = end;
      continue;
    }
    if (state.htmlTag) {
      const attribute = line.slice(index).match(/^(?:[@:#]|v-)?[\w.-]+/u)?.[0];
      if (attribute) {
        append(tokens, attribute, "attribute");
        index += attribute.length;
        continue;
      }
      if (character === "=") {
        append(tokens, character, "punctuation");
        index += 1;
        continue;
      }
    }
    const nextTag = line.indexOf("<", index + 1);
    const next = state.htmlTag ? index + 1 : nextTag < 0 ? line.length : nextTag;
    append(tokens, line.slice(index, next), "plain");
    index = next;
  }
  return tokens;
}

function tokenizeDataLine(
  line: string,
  state: LexerState,
  language: "css" | "json",
): ThemedCodeToken[] {
  const tokens = tokenizeScriptLine(line, state);
  if (language === "json") {
    for (const token of tokens) {
      if (token.darkColor == null && /^(?:false|null|true)$/u.test(token.content)) {
        [token.darkColor, token.lightColor] = colors.type;
      }
    }
    return tokens;
  }
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    if (token.darkColor == null && /^@[\w-]+$/u.test(token.content)) {
      [token.darkColor, token.lightColor] = colors.keyword;
    } else if (
      token.darkColor == null &&
      /^[\w-]+$/u.test(token.content) &&
      tokens.slice(index + 1).find((candidate) => candidate.content.trim() !== "")?.content === ":"
    ) {
      [token.darkColor, token.lightColor] = colors.attribute;
    }
  }
  return tokens;
}

/** Dependency-free lexer initialization retained for the worker lifecycle contract. */
export async function initCodeHighlighter(): Promise<void> {}

/**
 * Produces safe, line-preserving tokens without loading a browser grammar runtime.
 * It intentionally covers the playground's five output languages rather than
 * attempting editor-grade semantic highlighting.
 */
export async function codeToThemedTokenLines(
  code: string,
  language: CodeHighlightLanguage,
): Promise<ThemedCodeToken[][]> {
  if (code === "") return [];
  const lines = code.split("\n");
  if (lines.at(-1) === "") lines.pop();
  const state: LexerState = {
    blockComment: false,
    htmlComment: false,
    htmlTag: false,
    quote: null,
  };
  return lines.map((line) => {
    if (language === "html") return tokenizeHtmlLine(line, state);
    if (language === "css" || language === "json") {
      return tokenizeDataLine(line, state, language);
    }
    return tokenizeScriptLine(line, state);
  });
}
