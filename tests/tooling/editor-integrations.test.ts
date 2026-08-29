import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  loadVueTextMateGrammar,
  tokenizeLines,
  type TextMateToken,
} from "./support/vue-textmate.ts";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
function readText(relativePath: string): string {
  return fs.readFileSync(path.join(root, relativePath), "utf-8");
}

function readJson<T>(relativePath: string): T {
  return JSON.parse(readText(relativePath)) as T;
}

function workspaceVersion(): string {
  const version = readText("Cargo.toml").match(/^version = "(.+)"$/m)?.[1];
  assert.ok(version);
  return version;
}

function quoteAwareTagLookahead(begin: string | undefined): void {
  assert.ok(begin);
  assert.match(begin, /\(\?:\[\^"'<>\]\|"\[\^"\]\*"\|'\[\^'\]\*'\)\*/);
  assert.doesNotMatch(begin, /\[\^>\]\*/);
}

function tokensForText(tokens: TextMateToken[], text: string): TextMateToken[] {
  return tokens.filter((token) => token.text.includes(text));
}

function assertTextHasScope(tokens: TextMateToken[], text: string, scopePart: string): void {
  assert.equal(
    tokensForText(tokens, text).some((token) =>
      token.scopes.some((scope) => scope.includes(scopePart)),
    ),
    true,
    `${JSON.stringify(text)} should include scope ${scopePart}. Tokens: ${JSON.stringify(
      tokensForText(tokens, text),
    )}`,
  );
}

function assertTextDoesNotHaveScope(
  tokens: TextMateToken[],
  text: string,
  scopePart: string,
): void {
  assert.equal(
    tokensForText(tokens, text).some((token) =>
      token.scopes.some((scope) => scope.includes(scopePart)),
    ),
    false,
    `${JSON.stringify(text)} should not include scope ${scopePart}. Tokens: ${JSON.stringify(
      tokensForText(tokens, text),
    )}`,
  );
}

test("vscode-vize wires art-vue documents into editor features", () => {
  const manifest = readJson<{
    activationEvents?: string[];
    contributes?: {
      grammars?: Array<{
        embeddedLanguages?: Record<string, string>;
        language?: string;
        path?: string;
        scopeName?: string;
      }>;
      languages?: Array<{ id?: string; extensions?: string[] }>;
      menus?: {
        commandPalette?: Array<{ command?: string; when?: string }>;
      };
    };
  }>("editors/vscode/package.json");

  assert.equal(manifest.activationEvents?.includes("onLanguage:art-vue"), true);
  assert.equal(
    manifest.contributes?.languages?.some(
      (language) => language.id === "art-vue" && language.extensions?.includes(".art.vue"),
    ),
    true,
  );
  assert.ok(
    manifest.contributes?.grammars?.some(
      (grammar) =>
        grammar.language === "art-vue" &&
        grammar.scopeName === "source.art-vue" &&
        grammar.path === "./syntaxes/art-vue.tmLanguage.json",
    ),
  );
  const vueGrammarContribution = manifest.contributes?.grammars?.find(
    (grammar) => grammar.language === "vue",
  );
  assert.equal(vueGrammarContribution?.embeddedLanguages?.["source.tsx"], "typescriptreact");
  assert.equal(vueGrammarContribution?.embeddedLanguages?.["source.js.jsx"], "javascriptreact");
  assert.equal(vueGrammarContribution?.embeddedLanguages?.["text.pug"], "pug");
  assert.equal(vueGrammarContribution?.embeddedLanguages?.["source.graphql"], "graphql");

  const languageScopedCommands = new Set(["vize.restartServer", "vize.showOutput"]);
  for (const item of manifest.contributes?.menus?.commandPalette ?? []) {
    if (languageScopedCommands.has(item.command ?? "")) {
      assert.match(item.when ?? "", /editorLangId == art-vue/);
    }
  }
  const extensionSource = readText("editors/vscode/src/extension.ts");
  const extensionCoreSource = readText("editors/vscode/src/extension-core.ts");

  assert.match(extensionCoreSource, /SUPPORTED_LANGUAGE_IDS\s*=\s*\["vue", "art-vue", "html"\]/);
  assert.match(extensionCoreSource, /SUPPORTED_URI_SCHEMES\s*=\s*\["file", "untitled"\]/);
  assert.match(extensionCoreSource, /function createDocumentSelector/);
  assert.match(extensionSource, /documentSelector:\s*createDocumentSelector\(\)/);
  assert.match(extensionSource, /onDidChangeConfiguration/);
  assert.match(extensionSource, /scheduleClientSync\(context,\s*"configuration changed"\)/);
  assert.match(extensionSource, /function scheduleClientSync/);
  assert.match(extensionSource, /void syncClientToConfiguration\(context,\s*reason\)/);
  assert.match(extensionSource, /nextClient\.setTrace\(trace\)/);
  assert.match(extensionSource, /Trace\.(Verbose|Messages|Off)/);
});

test("vscode-vize grammar keeps quote-aware block lookaheads", () => {
  type GrammarCapture = {
    name?: string;
    patterns?: GrammarPattern[];
  };
  type GrammarPattern = {
    begin?: string;
    beginCaptures?: Record<string, GrammarCapture>;
    captures?: Record<string, GrammarCapture>;
    contentName?: string;
    include?: string;
    match?: string;
    patterns?: GrammarPattern[];
  };
  const grammar = readJson<{
    repository?: Record<
      string,
      { begin?: string; contentName?: string; patterns?: GrammarPattern[] }
    >;
  }>("editors/vscode/syntaxes/vue.tmLanguage.json");

  const repository = grammar.repository ?? {};

  for (const key of [
    "vue-template-pug",
    "vue-template",
    "vue-script-tsx",
    "vue-script-ts",
    "vue-script-jsx",
    "vue-script-js",
    "vue-style-scss",
    "vue-style-less",
    "vue-style-sass",
    "vue-style-stylus",
    "vue-style-postcss",
    "vue-style-css",
    "vue-custom-block-json",
    "vue-custom-block-yaml",
    "vue-custom-block-toml",
    "vue-custom-block-graphql",
    "vue-custom-block",
  ]) {
    quoteAwareTagLookahead(repository[key]?.begin);
  }

  for (const pattern of repository["vue-directive-attributes"]?.patterns ?? []) {
    assert.doesNotMatch(pattern.begin ?? "", /\(\?<=\\s\|\^\)/);
  }
  const directivePatterns = repository["vue-directive-attributes"]?.patterns ?? [];
  assert.match(
    ' v-bind:[activeKey as keyof Props].camel="makeValue<User>() as User"',
    new RegExp(directivePatterns[0]?.begin ?? ""),
  );
  assert.match(
    ' :[activeKey as keyof Props].prop="makeValue<User>() as User"',
    new RegExp(directivePatterns[1]?.begin ?? ""),
  );
  assert.match(
    ' @[eventName as keyof Emits].stop="handler($event as MouseEvent)"',
    new RegExp(directivePatterns[2]?.begin ?? ""),
  );
  assert.match(
    ' #[slotName as keyof Slots]="slotProps as SlotProps"',
    new RegExp(directivePatterns[3]?.begin ?? ""),
  );
  for (const pattern of directivePatterns) {
    assert.equal(pattern.contentName, "meta.embedded.expression.vue");
    assert.equal(pattern.patterns?.[0]?.include, "#vue-ts-expression");
  }
  assert.equal(
    directivePatterns[0]?.beginCaptures?.["5"]?.patterns?.[0]?.include,
    "#vue-ts-expression",
  );
  assert.equal(
    directivePatterns[1]?.beginCaptures?.["4"]?.patterns?.[0]?.include,
    "#vue-ts-expression",
  );
  assert.equal(
    directivePatterns[2]?.beginCaptures?.["4"]?.patterns?.[0]?.include,
    "#vue-ts-expression",
  );
  assert.equal(
    directivePatterns[3]?.beginCaptures?.["4"]?.patterns?.[0]?.include,
    "#vue-ts-expression",
  );
  assert.equal(repository["vue-interpolation"]?.patterns?.[0]?.include, "source.ts#expression");
  assert.equal(repository["vue-template-pug"]?.patterns?.[1]?.patterns?.[0]?.include, "text.pug");
  assert.equal(repository["vue-script-tsx"]?.patterns?.[1]?.patterns?.[0]?.include, "source.tsx");
  assert.equal(
    repository["vue-generic-attribute"]?.patterns?.[0]?.contentName,
    "meta.embedded.type.typescript",
  );
  assert.equal(
    repository["vue-generic-attribute"]?.patterns?.[0]?.patterns?.[0]?.include,
    "#vue-ts-type",
  );
  assert.match(
    'generic="T extends Record<string, unknown> = Foo<User>"',
    new RegExp(repository["vue-generic-attribute"]?.patterns?.[0]?.begin ?? ""),
  );
  const valueLessDirectivePatterns = repository["vue-directives"]?.patterns ?? [];
  assert.match(
    "v-bind:[activeKey as keyof Props].camel",
    new RegExp(valueLessDirectivePatterns[1]?.match ?? ""),
  );
  assert.equal(
    valueLessDirectivePatterns[1]?.captures?.["5"]?.patterns?.[0]?.include,
    "#vue-ts-expression",
  );
  assert.equal(
    valueLessDirectivePatterns[2]?.captures?.["4"]?.patterns?.[0]?.include,
    "#vue-ts-expression",
  );
  assert.match(
    '<i18n message="a > b" lang="json">',
    new RegExp(repository["vue-custom-block-json"]?.begin ?? ""),
  );
  assert.equal(repository["vue-custom-block-json"]?.contentName, "meta.embedded.block.json");
  assert.equal(repository["vue-custom-block-graphql"]?.patterns?.[0]?.include, "source.graphql");

  const artGrammar = readJson<{
    patterns?: Array<{ include?: string }>;
    scopeName?: string;
  }>("editors/vscode/syntaxes/art-vue.tmLanguage.json");
  assert.equal(artGrammar.scopeName, "source.art-vue");
  assert.deepEqual(artGrammar.patterns, [
    { include: "#art-comments" },
    { include: "#art-block" },
    { include: "source.vue" },
  ]);
});

test("vscode-vize grammar tokenizes TypeScript template expressions without falling back to HTML", async () => {
  const { grammar, registry } = await loadVueTextMateGrammar();

  try {
    const tokens = tokenizeLines(grammar, [
      '<script setup lang="ts" generic="T extends Record<string, unknown> = Foo<User>">',
      "const value = makeValue<T>() as T",
      "</script>",
      "<template>",
      '  <button v-bind:[activeKey as keyof Props].camel="makeValue<User>() as User" :[propName as keyof Props].prop="read<User>() as User" @[eventName as keyof Emits].stop="emit($event as MouseEvent)">',
      "    {{ makeValue<User>() as User }}",
      "  </button>",
      "</template>",
    ]);

    assertTextHasScope(tokens, "Record", "meta.embedded.type.typescript");
    assertTextHasScope(tokens, "User", "meta.embedded.type.typescript");
    assertTextHasScope(tokens, "activeKey", "meta.embedded.expression.vue");
    assertTextHasScope(tokens, "keyof", "meta.embedded.expression.vue");
    assertTextHasScope(tokens, "Props", "meta.embedded.expression.vue");
    assertTextHasScope(tokens, "makeValue", "meta.embedded.expression.vue");
    assertTextHasScope(tokens, "MouseEvent", "meta.embedded.expression.vue");
    assertTextDoesNotHaveScope(tokens, "User", "entity.name.tag.html");
    assertTextDoesNotHaveScope(tokens, "Props", "entity.name.tag.html");
    assertTextDoesNotHaveScope(tokens, "MouseEvent", "entity.name.tag.html");
  } finally {
    registry.dispose();
  }
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
  }>("editors/vscode-art/package.json");

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
  }>("editors/vscode-art/syntaxes/art.tmLanguage.json");

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
  const manifest = readText("editors/zed/extension.toml");
  assert.match(manifest, /^languages = \["Vue", "Art Vue"\]$/m);
  assert.match(manifest, /^"Vue" = "vue"$/m);
  assert.match(manifest, /^"Art Vue" = "art-vue"$/m);
  assert.match(manifest, /^\[grammars\.vue\]$/m);

  const artConfig = readText("editors/zed/languages/art-vue/config.toml");
  assert.match(artConfig, /^name = "Art Vue"$/m);
  assert.match(artConfig, /^grammar = "vue"$/m);
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
      fs.existsSync(path.join(root, "editors/zed/languages/art-vue", filename)),
      true,
      `missing zed art-vue language file: ${filename}`,
    );
  }

  const injections = readText("editors/zed/languages/art-vue/injections.scm");
  assert.match(injections, /directive_attribute/);
  assert.match(injections, /style_element/);
  assert.match(injections, /template_element/);
});
