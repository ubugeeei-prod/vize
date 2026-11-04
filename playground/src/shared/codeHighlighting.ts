export type CodeHighlightLanguage = "javascript" | "json" | "css" | "html" | "typescript";

export interface ThemedCodeToken {
  content: string;
  darkColor: string | undefined;
  lightColor: string | undefined;
}

export interface HighlightRequest {
  id: number;
  code: string;
  language: CodeHighlightLanguage;
}
export type HighlightResponse =
  | { type: "ready" }
  | { type: "initialization-error" }
  | { type: "result"; id: number; lines: ThemedCodeToken[][] }
  | { type: "error"; id: number };

export function escapeHtml(value: string): string {
  return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

export function normalizePlainHtmlLines(code: string): string[] {
  if (!code) {
    return [];
  }
  const lines = code.split("\n");
  if (lines.length > 0 && lines[lines.length - 1] === "") {
    lines.pop();
  }
  return lines.map((line) => (line ? escapeHtml(line) : "&nbsp;"));
}

export function codeTokensToHtml(tokens: ThemedCodeToken[]): string {
  if (tokens.every((token) => token.content === "")) {
    return "&nbsp;";
  }
  return tokens
    .map(
      (token) =>
        `<span style="--d:${token.darkColor ?? "var(--code-foreground)"};--l:${
          token.lightColor ?? "var(--code-foreground)"
        }">${escapeHtml(token.content)}</span>`,
    )
    .join("");
}
