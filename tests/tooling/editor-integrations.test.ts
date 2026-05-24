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
      grammars?: Array<{ language?: string; path?: string; scopeName?: string }>;
      languages?: Array<{ id?: string; extensions?: string[] }>;
      menus?: {
        commandPalette?: Array<{ command?: string; when?: string }>;
      };
    };
  }>("npm/vscode-vize/package.json");

  assert.equal(manifest.activationEvents?.includes("onLanguage:art-vue"), true);
  assert.equal(
    manifest.contributes?.languages?.some(
      (language) => language.id === "art-vue" && language.extensions?.includes(".art.vue"),
    ),
    true,
  );
  assert.equal(
    manifest.contributes?.grammars?.some(
      (grammar) =>
        grammar.language === "art-vue" &&
        grammar.scopeName === "source.art-vue" &&
        grammar.path === "./syntaxes/art-vue.tmLanguage.json",
    ),
    true,
  );

  const languageScopedCommands = new Set(["vize.restartServer", "vize.showOutput"]);
  for (const item of manifest.contributes?.menus?.commandPalette ?? []) {
    if (languageScopedCommands.has(item.command ?? "")) {
      assert.match(item.when ?? "", /editorLangId == art-vue/);
    }
  }

  const extensionSource = fs.readFileSync(
    path.join(root, "npm/vscode-vize/src/extension.ts"),
    "utf-8",
  );

  assert.match(extensionSource, /SUPPORTED_LANGUAGE_IDS\s*=\s*\["vue", "art-vue"\]/);
  assert.match(extensionSource, /SUPPORTED_URI_SCHEMES\s*=\s*\["file", "untitled"\]/);
  assert.match(extensionSource, /documentSelector:\s*SUPPORTED_URI_SCHEMES\.flatMap/);
  assert.match(extensionSource, /onDidChangeConfiguration/);
  assert.match(extensionSource, /scheduleClientSync\(context,\s*"configuration changed"\)/);
  assert.match(extensionSource, /function scheduleClientSync/);
  assert.match(extensionSource, /void syncClientToConfiguration\(context,\s*reason\)/);
  assert.match(extensionSource, /nextClient\.setTrace\(trace\)/);
  assert.match(extensionSource, /Trace\.(Verbose|Messages|Off)/);
});

test("vscode-vize grammar keeps quote-aware block lookaheads", () => {
  const grammar = readJson<{
    repository?: Record<string, { begin?: string; patterns?: Array<{ begin?: string }> }>;
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

  for (const pattern of repository["vue-directive-attributes"]?.patterns ?? []) {
    assert.doesNotMatch(pattern.begin ?? "", /\(\?<=\\s\|\^\)/);
  }

  const artGrammar = readJson<{
    patterns?: Array<{ include?: string }>;
    scopeName?: string;
  }>("npm/vscode-vize/syntaxes/art-vue.tmLanguage.json");
  assert.equal(artGrammar.scopeName, "source.art-vue");
  assert.deepEqual(artGrammar.patterns, [{ include: "source.vue" }]);
});

test("vscode-art grammar stays aligned with vue-aware editor support", () => {
  const manifest = readJson<{
    contributes?: {
      grammars?: Array<{
        embeddedLanguages?: Record<string, string>;
      }>;
    };
    license?: string;
    scripts?: Record<string, string>;
    version?: string;
  }>("npm/vscode-art/package.json");

  assert.equal(manifest.version, workspaceVersion());
  assert.equal(manifest.license, "MIT");
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
  const buildTasks = fs.readFileSync(path.join(root, "tools/vite-plus/tasks/build.ts"), "utf-8");
  const testTasks = fs.readFileSync(
    path.join(root, "tools/vite-plus/tasks/test-benchmark.ts"),
    "utf-8",
  );

  assert.match(
    workflow,
    /name: Check and package editor extensions[\s\S]*package:editor-extensions/,
  );
  assert.match(buildTasks, /package:vscode-extension[\s\S]*assert-vsix-package\.mjs/);
  assert.match(buildTasks, /package:editor-extensions[\s\S]*assert-vsix-package\.mjs/);
  assert.match(testTasks, /test:vscode-extension:vsix[\s\S]*assert-vsix-package\.mjs/);
  assert.match(testTasks, /test:vscode-extension:host[\s\S]*pnpm run test:host/);
  assert.match(buildTasks, /package:zed-extension[\s\S]*assert-zed-package\.mjs/);
  assert.match(testTasks, /test:zed-extension:package[\s\S]*package:zed-extension/);
  assert.match(testTasks, /test:zed-extension:unit[\s\S]*cargo test/);
  assert.match(buildTasks, /package:editor-extensions[\s\S]*test:zed-extension:unit/);
  assert.match(buildTasks, /package:nvim-extension[\s\S]*assert-nvim-package\.mjs/);
  assert.match(buildTasks, /package:editor-extensions[\s\S]*package:nvim-extension/);
  assert.match(testTasks, /test:nvim-extension:headless[\s\S]*nvim --headless/);
  assert.match(testTasks, /test:nvim-extension:package[\s\S]*package:nvim-extension/);
  assert.match(buildTasks, /package:vim-extension[\s\S]*assert-vim-package\.mjs/);
  assert.match(buildTasks, /package:editor-extensions[\s\S]*package:vim-extension/);
  assert.match(testTasks, /test:vim-extension:headless[\s\S]*vim -Nu NONE/);
  assert.match(testTasks, /test:vim-extension:package[\s\S]*package:vim-extension/);
  assert.match(buildTasks, /package:helix-extension[\s\S]*assert-helix-package\.mjs/);
  assert.match(buildTasks, /package:editor-extensions[\s\S]*package:helix-extension/);
  assert.match(testTasks, /test:helix-extension:package[\s\S]*package:helix-extension/);
  assert.match(buildTasks, /package:emacs-extension[\s\S]*assert-emacs-package\.mjs/);
  assert.match(buildTasks, /package:editor-extensions[\s\S]*package:emacs-extension/);
  assert.match(testTasks, /test:emacs-extension:package[\s\S]*package:emacs-extension/);
});
