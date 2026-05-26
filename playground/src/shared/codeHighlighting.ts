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
    "editor.background": "#1a1a1a",
    "editor.foreground": "#E6E2D6",
  },
  tokenColors: [
    {
      scope: ["keyword", "storage.type", "storage.modifier"],
      settings: { foreground: "#D4BA92" },
    },
    {
      scope: ["entity.name.function", "support.function"],
      settings: { foreground: "#E2CBA6" },
    },
    {
      scope: ["entity.name.tag", "punctuation.definition.tag"],
      settings: { foreground: "#D0BA9E" },
    },
    {
      scope: ["entity.other.attribute-name"],
      settings: { foreground: "#9C9488" },
    },
    { scope: ["string", "string.quoted"], settings: { foreground: "#A8B5A0" } },
    {
      scope: ["constant.numeric", "constant.language"],
      settings: { foreground: "#DABA8C" },
    },
    {
      scope: ["variable", "variable.other"],
      settings: { foreground: "#E6E2D6" },
    },
    {
      scope: ["comment", "punctuation.definition.comment"],
      settings: { foreground: "#6B6560" },
    },
    {
      scope: ["punctuation", "meta.brace"],
      settings: { foreground: "#8A8478" },
    },
    {
      scope: ["entity.name.type", "support.type"],
      settings: { foreground: "#B8ADA0" },
    },
    {
      scope: ["meta.property-name", "support.type.property-name"],
      settings: { foreground: "#D0BA9E" },
    },
    {
      scope: ["meta.property-value", "support.constant.property-value"],
      settings: { foreground: "#A8B5A0" },
    },
  ],
};

const vizeLightTheme: ThemeRegistration = {
  name: "vize-light",
  type: "light",
  colors: {
    "editor.background": "#ddd9cd",
    "editor.foreground": "#121212",
  },
  tokenColors: [
    {
      scope: ["keyword", "storage.type", "storage.modifier"],
      settings: { foreground: "#73603E" },
    },
    {
      scope: ["entity.name.function", "support.function"],
      settings: { foreground: "#655232" },
    },
    {
      scope: ["entity.name.tag", "punctuation.definition.tag"],
      settings: { foreground: "#65573E" },
    },
    {
      scope: ["entity.other.attribute-name"],
      settings: { foreground: "#6B6050" },
    },
    { scope: ["string", "string.quoted"], settings: { foreground: "#5A6B50" } },
    {
      scope: ["constant.numeric", "constant.language"],
      settings: { foreground: "#735C2E" },
    },
    {
      scope: ["variable", "variable.other"],
      settings: { foreground: "#121212" },
    },
    {
      scope: ["comment", "punctuation.definition.comment"],
      settings: { foreground: "#9A9590" },
    },
    {
      scope: ["punctuation", "meta.brace"],
      settings: { foreground: "#6B6560" },
    },
    {
      scope: ["entity.name.type", "support.type"],
      settings: { foreground: "#6B5F50" },
    },
    {
      scope: ["meta.property-name", "support.type.property-name"],
      settings: { foreground: "#65573E" },
    },
    {
      scope: ["meta.property-value", "support.constant.property-value"],
      settings: { foreground: "#4A5F3E" },
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
        `<span style="--d:${token.darkColor};--l:${token.lightColor}">${escapeHtml(
          token.content,
        )}</span>`,
    )
    .join("");
}
