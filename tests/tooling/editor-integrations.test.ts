import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readJson<T>(relativePath: string): T {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf-8")) as T;
}

function workspaceVersion(): string {
  const version = fs
    .readFileSync(path.join(root, "Cargo.toml"), "utf-8")
    .match(/^version = "(.+)"$/m)?.[1];

  assert.ok(version);
  return version;
}

function quoteAwareTagLookahead(begin: string | undefined): void {
  assert.ok(begin);
  assert.match(begin, /\(\?:\[\^"'<>\]\|"\[\^"\]\*"\|'\[\^'\]\*'\)\*/);
  assert.doesNotMatch(begin, /\[\^>\]\*/);
}

test("vscode-vize wires art-vue documents into editor features", () => {
  const manifest = readJson<{
    activationEvents?: string[];
    contributes?: {
      menus?: {
        commandPalette?: Array<{ when?: string }>;
      };
    };
  }>("npm/vscode-vize/package.json");

  assert.equal(manifest.activationEvents?.includes("onLanguage:art-vue"), true);

  for (const item of manifest.contributes?.menus?.commandPalette ?? []) {
    assert.match(item.when ?? "", /editorLangId == art-vue/);
  }

  const extensionSource = fs.readFileSync(
    path.join(root, "npm/vscode-vize/src/extension.ts"),
    "utf-8",
  );

  assert.match(extensionSource, /SUPPORTED_LANGUAGE_IDS\s*=\s*\["vue", "art-vue"\]/);
  assert.match(extensionSource, /SUPPORTED_URI_SCHEMES\s*=\s*\["file", "untitled"\]/);
  assert.match(extensionSource, /documentSelector:\s*SUPPORTED_URI_SCHEMES\.flatMap/);
  assert.match(extensionSource, /onDidChangeConfiguration/);
  assert.match(extensionSource, /syncClientToConfiguration\(context,\s*"configuration changed"\)/);
  assert.match(extensionSource, /nextClient\.setTrace\(trace\)/);
  assert.match(extensionSource, /Trace\.(Verbose|Messages|Off)/);
});

test("vscode-vize grammar keeps quote-aware block lookaheads", () => {
  const grammar = readJson<{
    repository?: Record<string, { begin?: string }>;
  }>("npm/vscode-vize/syntaxes/vue.tmLanguage.json");

  const repository = grammar.repository ?? {};

  for (const key of [
    "vue-template",
    "vue-script-ts",
    "vue-script-js",
    "vue-style-scss",
    "vue-style-less",
    "vue-style-css",
    "vue-custom-block",
  ]) {
    quoteAwareTagLookahead(repository[key]?.begin);
  }
});

test("vscode-art grammar stays aligned with vue-aware editor support", () => {
  const manifest = readJson<{
    contributes?: {
      grammars?: Array<{
        embeddedLanguages?: Record<string, string>;
      }>;
    };
    scripts?: Record<string, string>;
    version?: string;
  }>("npm/vscode-art/package.json");

  assert.equal(manifest.version, workspaceVersion());
  assert.equal(manifest.scripts?.compile, "tsgo -p ./");
  assert.equal(manifest.scripts?.watch, "tsgo -watch -p ./");

  const embeddedLanguages = manifest.contributes?.grammars?.[0]?.embeddedLanguages ?? {};
  assert.equal(embeddedLanguages["source.css.scss"], "scss");
  assert.equal(embeddedLanguages["source.css.less"], "less");
  assert.equal(embeddedLanguages["source.json"], "json");

  const grammar = readJson<{
    patterns?: Array<{ include?: string }>;
    repository?: Record<string, { begin?: string; patterns?: Array<{ include?: string }> }>;
  }>("npm/vscode-art/syntaxes/art.tmLanguage.json");

  assert.deepEqual(
    (grammar.patterns ?? []).map((pattern) => pattern.include),
    [
      "#vue-comments",
      "#art-block",
      "#vue-template",
      "#vue-script",
      "#vue-style",
      "#vue-custom-block",
    ],
  );

  const repository = grammar.repository ?? {};
  quoteAwareTagLookahead(repository["art-block"]?.begin);
  quoteAwareTagLookahead(repository["variant-block"]?.begin);
  quoteAwareTagLookahead(repository["vue-script-ts"]?.begin);
  quoteAwareTagLookahead(repository["vue-style-scss"]?.begin);

  assert.ok(repository["variant-args-single"]);
  assert.ok(repository["variant-args-double"]);
  assert.ok(repository["vue-directive-attributes"]);
  assert.ok(repository["html-tags"]);

  assert.deepEqual(
    (repository["variant-content"]?.patterns ?? []).map((pattern) => pattern.include),
    ["#vue-comments", "#vue-interpolation", "#vue-directives", "#html-tags", "#html-entities"],
  );
});

test("zed-vize registers art-vue as a first-party language", () => {
  const manifest = fs.readFileSync(path.join(root, "npm/zed-vize/extension.toml"), "utf-8");
  assert.match(manifest, /^languages = \["Vue", "Art Vue"\]$/m);
  assert.match(manifest, /^"Vue" = "vue"$/m);
  assert.match(manifest, /^"Art Vue" = "art-vue"$/m);
  assert.match(manifest, /^\[grammars\.art-vue\]$/m);

  const artConfig = fs.readFileSync(
    path.join(root, "npm/zed-vize/languages/art-vue/config.toml"),
    "utf-8",
  );
  assert.match(artConfig, /^name = "Art Vue"$/m);
  assert.match(artConfig, /^grammar = "art-vue"$/m);
  assert.match(artConfig, /^path_suffixes = \["art\.vue"\]$/m);
  assert.match(artConfig, /^prettier_parser_name = "vue"$/m);

  for (const filename of [
    "brackets.scm",
    "highlights.scm",
    "indents.scm",
    "injections.scm",
    "outline.scm",
    "overrides.scm",
  ]) {
    assert.equal(
      fs.existsSync(path.join(root, "npm/zed-vize/languages/art-vue", filename)),
      true,
      `missing zed art-vue language file: ${filename}`,
    );
  }

  const injections = fs.readFileSync(
    path.join(root, "npm/zed-vize/languages/art-vue/injections.scm"),
    "utf-8",
  );
  assert.match(injections, /directive_attribute/);
  assert.match(injections, /style_element/);
  assert.match(injections, /template_element/);
});

test("CI packages editor extension artifacts", () => {
  const workflow = fs.readFileSync(path.join(root, ".github/workflows/check.yml"), "utf-8");
  assert.match(
    workflow,
    /name: Check and package editor extensions[\s\S]*package:editor-extensions/,
  );
});
