import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readJson<T>(relativePath: string): T {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf-8")) as T;
}

test("legacy vscode-art language and grammar declarations stay self-consistent", () => {
  const manifest = readJson<{
    activationEvents?: string[];
    contributes?: {
      grammars?: Array<{
        embeddedLanguages?: Record<string, string>;
        language?: string;
        path?: string;
        scopeName?: string;
      }>;
      languages?: Array<{
        aliases?: string[];
        configuration?: string;
        extensions?: string[];
        id?: string;
      }>;
    };
  }>("npm/editor/vscode-art/package.json");

  const language = manifest.contributes?.languages?.find((entry) => entry.id === "art-vue");
  assert.ok(language, "vize-art should declare an art-vue language");
  assert.deepEqual(language.extensions, [".art.vue"]);
  assert.deepEqual(language.aliases, ["Art Vue", "art"]);
  assert.equal(language.configuration, "./language-configuration.json");

  const grammar = manifest.contributes?.grammars?.find((entry) => entry.language === "art-vue");
  assert.ok(grammar, "vize-art should declare an art-vue grammar");
  // vize-art deliberately uses the legacy dotted scope (source.art.vue), which is
  // distinct from vscode-vize's hyphenated source.art-vue; pin that divergence.
  assert.equal(grammar.scopeName, "source.art.vue");
  assert.equal(grammar.path, "./syntaxes/art.tmLanguage.json");

  // The referenced grammar file must exist and declare the same scopeName the
  // manifest promises, or VS Code silently fails to tokenize art-vue documents.
  const grammarRelativePath = path.join("npm/editor/vscode-art", grammar.path);
  assert.equal(
    fs.existsSync(path.join(root, grammarRelativePath)),
    true,
    `grammar file ${grammarRelativePath} should exist`,
  );
  const grammarFile = readJson<{ scopeName?: string }>(grammarRelativePath);
  assert.equal(grammarFile.scopeName, "source.art.vue");

  assert.equal(manifest.activationEvents?.includes("onLanguage:art-vue"), true);

  const embeddedLanguages = grammar.embeddedLanguages ?? {};
  assert.equal(embeddedLanguages["source.ts"], "typescript");
  assert.equal(embeddedLanguages["source.css.scss"], "scss");
  assert.equal(embeddedLanguages["source.json"], "json");
});

test("legacy vscode-art language-configuration comments, brackets, and folding are correct", () => {
  const config = readJson<{
    autoClosingPairs?: Array<{ close?: string; open?: string }>;
    brackets?: string[][];
    comments?: { blockComment?: string[] };
    folding?: { markers?: { end?: string; start?: string } };
    surroundingPairs?: string[][];
    wordPattern?: string;
  }>("npm/editor/vscode-art/language-configuration.json");

  assert.deepEqual(config.comments?.blockComment, ["<!--", "-->"]);
  assert.deepEqual(config.brackets, [
    ["<", ">"],
    ["{", "}"],
    ["[", "]"],
    ["(", ")"],
  ]);

  const autoClosingPairs = config.autoClosingPairs ?? [];
  assert.ok(
    autoClosingPairs.some((pair) => pair.open === "<!--" && pair.close === " -->"),
    "auto-closing pairs should include the HTML comment pair",
  );
  for (const quote of ['"', "'", "`"]) {
    assert.ok(
      autoClosingPairs.some((pair) => pair.open === quote && pair.close === quote),
      `auto-closing pairs should include the ${quote} quote pair`,
    );
  }

  // vize-art uses the array form of surroundingPairs (["<", ">"]) rather than the
  // object form ({ open, close }); pin that representation.
  assert.deepEqual(config.surroundingPairs, [
    ["<", ">"],
    ["{", "}"],
    ["[", "]"],
    ["(", ")"],
    ['"', '"'],
    ["'", "'"],
    ["`", "`"],
  ]);

  assert.match(config.folding?.markers?.start ?? "", /#region/);
  assert.match(config.folding?.markers?.end ?? "", /#endregion/);

  assert.ok(config.wordPattern, "language configuration should declare a word pattern");
  assert.equal(new RegExp(config.wordPattern).test("foo-bar"), true);
});
