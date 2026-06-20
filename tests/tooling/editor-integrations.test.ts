import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const textmateModulePath = path.join(
  root,
  "node_modules/.pnpm/@shikijs+vscode-textmate@10.0.2/node_modules/@shikijs/vscode-textmate/dist/index.js",
);
const onigurumaModulePath = path.join(
  root,
  "node_modules/.pnpm/@shikijs+engine-oniguruma@4.0.2/node_modules/@shikijs/engine-oniguruma/dist/index.mjs",
);
const onigurumaWasmPath = path.join(
  root,
  "node_modules/.pnpm/shiki@4.0.2/node_modules/shiki/dist/onig.wasm",
);

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

function createStubGrammar(scopeName: string) {
  const patterns = [
    { match: "\\b(as|extends|keyof|typeof|infer|satisfies)\\b", name: "keyword.operator.ts" },
    { match: "\\b[A-Za-z_$][\\w$]*\\b", name: "identifier.ts" },
    { match: "[<>{}()\\[\\].,:?=+\\-*/|&!]+", name: "punctuation.ts" },
    { match: "\"(?:\\\\.|[^\"])*\"|'(?:\\\\.|[^'])*'|`(?:\\\\.|[^`])*`", name: "string.ts" },
  ];

  return {
    scopeName,
    patterns,
    repository: {
      expression: { patterns },
      "type-inner": { patterns },
    },
  };
}

async function loadVueTextMateGrammar() {
  const [{ Registry }, { createOnigurumaEngine }] = await Promise.all([
    import(pathToFileURL(textmateModulePath).href),
    import(pathToFileURL(onigurumaModulePath).href),
  ]);
  const engine = await createOnigurumaEngine(fs.readFileSync(onigurumaWasmPath));
  const grammars = new Map<string, unknown>([
    ["source.vue", readJson("npm/editor/vscode/syntaxes/vue.tmLanguage.json")],
    ["source.art-vue", readJson("npm/editor/vscode/syntaxes/art-vue.tmLanguage.json")],
  ]);
  const registry = new Registry({
    onigLib: {
      createOnigScanner(patterns: Array<string | RegExp>) {
        return engine.createScanner(patterns);
      },
      createOnigString(value: string) {
        return engine.createString(value);
      },
    },
    loadGrammar(scopeName: string) {
      return grammars.get(scopeName) ?? createStubGrammar(scopeName);
    },
  });

  const grammar = registry.loadGrammar("source.vue");
  assert.ok(grammar);
  return { grammar, registry };
}

function tokenizeLines(
  grammar: {
    tokenizeLine(
      lineText: string,
      prevState: unknown,
    ): { ruleStack: unknown; tokens: TextMateToken[] };
  },
  lines: string[],
): TextMateToken[] {
  let ruleStack: unknown = null;
  const tokens: TextMateToken[] = [];

  for (const line of lines) {
    const result = grammar.tokenizeLine(line, ruleStack);
    for (const token of result.tokens) {
      tokens.push({
        endIndex: token.endIndex,
        line,
        scopes: token.scopes,
        startIndex: token.startIndex,
        text: line.slice(token.startIndex, token.endIndex),
      });
    }
    ruleStack = result.ruleStack;
  }

  return tokens;
}

type TextMateToken = {
  endIndex: number;
  line: string;
  scopes: string[];
  startIndex: number;
  text: string;
};

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
  }>("npm/editor/vscode/package.json");

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

  const extensionSource = fs.readFileSync(
    path.join(root, "npm/editor/vscode/src/extension.ts"),
    "utf-8",
  );

  assert.match(extensionSource, /SUPPORTED_LANGUAGE_IDS\s*=\s*\["vue", "art-vue", "html"\]/);
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
  }>("npm/editor/vscode/syntaxes/vue.tmLanguage.json");

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
  }>("npm/editor/vscode/syntaxes/art-vue.tmLanguage.json");
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
  }>("npm/editor/vscode-art/package.json");

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
  }>("npm/editor/vscode-art/syntaxes/art.tmLanguage.json");

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
  const manifest = fs.readFileSync(path.join(root, "npm/editor/zed/extension.toml"), "utf-8");
  assert.match(manifest, /^languages = \["Vue", "Art Vue"\]$/m);
  assert.match(manifest, /^"Vue" = "vue"$/m);
  assert.match(manifest, /^"Art Vue" = "art-vue"$/m);
  assert.match(manifest, /^\[grammars\.art-vue\]$/m);

  const artConfig = fs.readFileSync(
    path.join(root, "npm/editor/zed/languages/art-vue/config.toml"),
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
      fs.existsSync(path.join(root, "npm/editor/zed/languages/art-vue", filename)),
      true,
      `missing zed art-vue language file: ${filename}`,
    );
  }

  const injections = fs.readFileSync(
    path.join(root, "npm/editor/zed/languages/art-vue/injections.scm"),
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
