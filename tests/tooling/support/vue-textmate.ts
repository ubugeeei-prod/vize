import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import {
  onigurumaModulePath,
  onigurumaWasmPath,
  shikiLanguageModulePath,
  textmateDependencyVersions,
  textmateModulePath,
} from "./textmate-deps.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const embeddedLanguageNames = [
  "coffee",
  "css",
  "graphql",
  "javascript",
  "json",
  "jsx",
  "less",
  "postcss",
  "pug",
  "sass",
  "scss",
  "stylus",
  "toml",
  "tsx",
  "typescript",
  "yaml",
];

export type TextMateToken = {
  endIndex: number;
  line: string;
  scopes: string[];
  startIndex: number;
  text: string;
};

export type TextMateGrammar = {
  tokenizeLine(
    lineText: string,
    prevState: unknown,
    timeLimit?: number,
  ): {
    ruleStack: unknown;
    stoppedEarly?: boolean;
    tokens: Array<{ endIndex: number; scopes: string[]; startIndex: number }>;
  };
};

type BundledGrammar = { scopeName?: string };

export type TextMateGrammarEvidence = {
  configuredGrammarSha256: string;
  dependencyVersions: typeof textmateDependencyVersions;
  requestedScopes: string[];
  rootScope: string;
};

function readJson<T>(relativePath: string): T {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf8")) as T;
}

async function loadBundledLanguageGrammars(name: string): Promise<BundledGrammar[]> {
  const module = await import(pathToFileURL(shikiLanguageModulePath(name)).href);
  return Array.isArray(module.default) ? module.default : [module.default];
}

/** Load the exact Vue and Art Vue TextMate grammars shipped by the VS Code extension. */
export async function loadVueTextMateGrammar(scopeName = "source.vue") {
  const [{ Registry }, { createOnigurumaEngine }, ...bundledLanguages] = await Promise.all([
    import(pathToFileURL(textmateModulePath).href),
    import(pathToFileURL(onigurumaModulePath).href),
    ...embeddedLanguageNames.map(loadBundledLanguageGrammars),
  ]);
  const engine = await createOnigurumaEngine(fs.readFileSync(onigurumaWasmPath));
  const grammars = new Map<string, unknown>();
  for (const language of bundledLanguages.flat()) {
    if (typeof language.scopeName === "string") grammars.set(language.scopeName, language);
  }
  // Vize's Vue grammar uses VS Code's historical scope spelling while Shiki's
  // bundled PostCSS grammar uses `source.css.postcss`.
  if (!grammars.has("source.postcss") && grammars.has("source.css.postcss")) {
    grammars.set("source.postcss", grammars.get("source.css.postcss"));
  }
  // Pug's historical embedded LESS scope differs from Shiki's canonical scope.
  if (!grammars.has("source.less") && grammars.has("source.css.less")) {
    grammars.set("source.less", grammars.get("source.css.less"));
  }
  grammars.set(
    "source.js.regexp",
    readJson("tests/tooling/fixtures/javascript-regexp.tmLanguage.json"),
  );
  grammars.set("source.sassdoc", readJson("tests/tooling/fixtures/sassdoc.tmLanguage.json"));
  grammars.set("source.vue", readJson("editors/vscode/syntaxes/vue.tmLanguage.json"));
  grammars.set("source.vue.script", readJson("editors/vscode/syntaxes/vue-script.tmLanguage.json"));
  grammars.set("source.art-vue", readJson("editors/vscode/syntaxes/art-vue.tmLanguage.json"));
  const configuredGrammarSha256 = createHash("sha256")
    .update(
      JSON.stringify([...grammars.entries()].sort(([left], [right]) => left.localeCompare(right))),
    )
    .digest("hex");
  const requestedScopes = new Set<string>();

  const registry = new Registry({
    onigLib: {
      createOnigScanner(patterns: Array<string | RegExp>) {
        return engine.createScanner(patterns);
      },
      createOnigString(value: string) {
        return engine.createString(value);
      },
    },
    loadGrammar(requestedScopeName: string) {
      requestedScopes.add(requestedScopeName);
      const loaded = grammars.get(requestedScopeName);
      if (loaded == null) {
        throw new Error(`unresolved TextMate grammar scope: ${requestedScopeName}`);
      }
      return loaded;
    },
  });

  let grammar: TextMateGrammar;
  try {
    const loaded = registry.loadGrammar(scopeName) as TextMateGrammar | null;
    assert.ok(loaded, `failed to load TextMate grammar ${scopeName}`);
    grammar = loaded;
  } catch (error) {
    registry.dispose();
    throw error;
  }
  return {
    getEvidence: (): TextMateGrammarEvidence => ({
      configuredGrammarSha256,
      dependencyVersions: textmateDependencyVersions,
      requestedScopes: [...requestedScopes].sort(),
      rootScope: scopeName,
    }),
    grammar,
    registry,
  };
}

export function tokenizeLines(grammar: TextMateGrammar, lines: string[]): TextMateToken[] {
  let ruleStack: unknown = null;
  const tokens: TextMateToken[] = [];

  for (const line of lines) {
    const result = grammar.tokenizeLine(line, ruleStack);
    tokens.push(
      ...result.tokens.map((token) => ({
        endIndex: token.endIndex,
        line,
        scopes: token.scopes,
        startIndex: token.startIndex,
        text: line.slice(token.startIndex, token.endIndex),
      })),
    );
    ruleStack = result.ruleStack;
  }

  return tokens;
}

/**
 * Tokenize one real SFC with strict range and per-line time-limit checks.
 * TextMate may include a one-character newline sentinel at the end of a line,
 * so the terminal bound accepts either the authored length or length + 1.
 */
export function auditTextMateSource(
  grammar: TextMateGrammar,
  source: string,
  rootScope: string,
  label: string,
): { lineCount: number; sha256: string; tokenCount: number } {
  const lines = source.split("\n");
  let ruleStack: unknown = null;
  let tokenCount = 0;
  let nonRootTokenCount = 0;
  const digest = createHash("sha256");

  for (const [lineIndex, line] of lines.entries()) {
    const result = grammar.tokenizeLine(line, ruleStack, 250);
    if (result.stoppedEarly) {
      throw new Error(`${label}:${lineIndex + 1}: TextMate tokenization exceeded 250ms`);
    }
    if (result.tokens.length === 0) {
      throw new Error(`${label}:${lineIndex + 1}: TextMate returned no tokens`);
    }

    let end = 0;
    for (const [tokenIndex, token] of result.tokens.entries()) {
      if (!Number.isSafeInteger(token.startIndex) || !Number.isSafeInteger(token.endIndex)) {
        throw new Error(`${label}:${lineIndex + 1}: token ${tokenIndex} has non-integer bounds`);
      }
      if (token.startIndex !== end || token.endIndex <= token.startIndex) {
        throw new Error(
          `${label}:${lineIndex + 1}: token ${tokenIndex} is not a contiguous positive span (${token.startIndex}..${token.endIndex}, expected ${end})`,
        );
      }
      if (token.endIndex > line.length + 1) {
        throw new Error(
          `${label}:${lineIndex + 1}: token ${tokenIndex} exceeds the line (${token.endIndex} > ${line.length + 1})`,
        );
      }
      if (token.scopes[0] !== rootScope) {
        throw new Error(
          `${label}:${lineIndex + 1}: token ${tokenIndex} lost root scope ${rootScope}`,
        );
      }
      if (token.scopes.some((scope) => scope !== rootScope)) nonRootTokenCount += 1;
      digest.update(
        `${JSON.stringify([lineIndex, token.startIndex, token.endIndex, token.scopes])}\n`,
      );
      end = token.endIndex;
      tokenCount += 1;
    }
    if (end !== line.length && end !== line.length + 1) {
      throw new Error(`${label}:${lineIndex + 1}: TextMate stopped at ${end}/${line.length}`);
    }
    ruleStack = result.ruleStack;
  }

  if (source.trim().length > 0 && nonRootTokenCount === 0) {
    throw new Error(`${label}: non-empty Vue source produced only the root scope`);
  }
  return { lineCount: lines.length, sha256: digest.digest("hex"), tokenCount };
}
