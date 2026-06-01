import { createHighlighter, type Highlighter, type ThemeRegistration } from "shiki";

export type CodeHighlightLanguage = "javascript" | "json" | "css" | "html" | "typescript";

export interface ThemedCodeToken {
  content: string;
  darkColor: string | undefined;
  lightColor: string | undefined;
}

// Custom themes - warm earthy tones matching brand
const vizeDarkTheme: ThemeRegistration = {
  name: "vize-dark",
  type: "dark",
  colors: {
    "editor.background": "#171717",
    "editor.foreground": "#EEE8DB",
  },
  tokenColors: [
    {
      scope: ["keyword", "storage.type", "storage.modifier"],
      settings: { foreground: "#F2B8A0" },
    },
    {
      scope: ["entity.name.function", "support.function"],
      settings: { foreground: "#FFD08A" },
    },
    {
      scope: ["entity.name.tag", "punctuation.definition.tag"],
      settings: { foreground: "#8ECDF0" },
    },
    {
      scope: ["entity.other.attribute-name"],
      settings: { foreground: "#D8B4FE" },
    },
    { scope: ["string", "string.quoted"], settings: { foreground: "#A7D8A8" } },
    {
      scope: ["constant.numeric", "constant.language"],
      settings: { foreground: "#F1BC78" },
    },
    {
      scope: ["variable", "variable.other"],
      settings: { foreground: "#EEE8DB" },
    },
    {
      scope: ["comment", "punctuation.definition.comment"],
      settings: { foreground: "#A39B8F" },
    },
    {
      scope: ["punctuation", "meta.brace"],
      settings: { foreground: "#B8AFA2" },
    },
    {
      scope: ["entity.name.type", "support.type"],
      settings: { foreground: "#C4C8FF" },
    },
    {
      scope: ["meta.property-name", "support.type.property-name"],
      settings: { foreground: "#9DD4FF" },
    },
    {
      scope: ["meta.property-value", "support.constant.property-value"],
      settings: { foreground: "#BADE9B" },
    },
  ],
};

const vizeLightTheme: ThemeRegistration = {
  name: "vize-light",
  type: "light",
  colors: {
    "editor.background": "#F3EFE2",
    "editor.foreground": "#141414",
  },
  tokenColors: [
    {
      scope: ["keyword", "storage.type", "storage.modifier"],
      settings: { foreground: "#7A2F15" },
    },
    {
      scope: ["entity.name.function", "support.function"],
      settings: { foreground: "#6B4A00" },
    },
    {
      scope: ["entity.name.tag", "punctuation.definition.tag"],
      settings: { foreground: "#005F80" },
    },
    {
      scope: ["entity.other.attribute-name"],
      settings: { foreground: "#5A3D86" },
    },
    { scope: ["string", "string.quoted"], settings: { foreground: "#1B6A38" } },
    {
      scope: ["constant.numeric", "constant.language"],
      settings: { foreground: "#73470D" },
    },
    {
      scope: ["variable", "variable.other"],
      settings: { foreground: "#141414" },
    },
    {
      scope: ["comment", "punctuation.definition.comment"],
      settings: { foreground: "#59554F" },
    },
    {
      scope: ["punctuation", "meta.brace"],
      settings: { foreground: "#5C5750" },
    },
    {
      scope: ["entity.name.type", "support.type"],
      settings: { foreground: "#3D4B80" },
    },
    {
      scope: ["meta.property-name", "support.type.property-name"],
      settings: { foreground: "#005F73" },
    },
    {
      scope: ["meta.property-value", "support.constant.property-value"],
      settings: { foreground: "#40680F" },
    },
  ],
};

type SharedHighlighterState = {
  highlighter: Highlighter | null;
  highlighterPromise: Promise<Highlighter> | null;
};

type SharedGlobal = typeof globalThis & {
  __vizeCodeHighlightState?: SharedHighlighterState;
};

const sharedGlobal = globalThis as SharedGlobal;
const sharedState =
  sharedGlobal.__vizeCodeHighlightState ??
  (sharedGlobal.__vizeCodeHighlightState = {
    highlighter: null,
    highlighterPromise: null,
  });

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

export async function initCodeHighlighter() {
  if (sharedState.highlighter) {
    return sharedState.highlighter;
  }
  if (!sharedState.highlighterPromise) {
    sharedState.highlighterPromise = createHighlighter({
      themes: [vizeDarkTheme, vizeLightTheme],
      langs: ["javascript", "json", "css", "html", "typescript"],
    }).then((instance) => {
      sharedState.highlighter = instance;
      return instance;
    });
  }
  return sharedState.highlighterPromise;
}

export async function codeToThemedHtmlLines(
  code: string,
  language: CodeHighlightLanguage,
): Promise<string[]> {
  const tokenLines = await codeToThemedTokenLines(code, language);
  return tokenLines.map(codeTokensToHtml);
}

export async function codeToThemedTokenLines(
  code: string,
  language: CodeHighlightLanguage,
): Promise<ThemedCodeToken[][]> {
  const highlighter = await initCodeHighlighter();

  // Tokenize with both themes so CSS can switch colors without JS re-render.
  const darkTokens = highlighter.codeToTokens(code, {
    lang: language,
    theme: "vize-dark",
  });
  const lightTokens = highlighter.codeToTokens(code, {
    lang: language,
    theme: "vize-light",
  });

  let darkLines = darkTokens.tokens;
  let lightLines = lightTokens.tokens;

  if (darkLines.length > 0 && darkLines[darkLines.length - 1].length === 0) {
    darkLines = darkLines.slice(0, -1);
  }
  if (lightLines.length > 0 && lightLines[lightLines.length - 1].length === 0) {
    lightLines = lightLines.slice(0, -1);
  }

  return darkLines.map((lineTokens, lineIdx) => {
    if (lineTokens.length === 0) {
      return [{ content: "", darkColor: undefined, lightColor: undefined }];
    }
    return lineTokens.map((token, tokenIdx) => ({
      content: token.content,
      darkColor: token.color,
      lightColor: lightLines[lineIdx]?.[tokenIdx]?.color ?? token.color,
    }));
  });
}

function codeTokensToHtml(tokens: ThemedCodeToken[]): string {
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
