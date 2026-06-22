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
const shikiLangsPath = path.join(
  root,
  "node_modules/.pnpm/@shikijs+langs@4.0.2/node_modules/@shikijs/langs/dist",
);

type TextMateToken = {
  endIndex: number;
  line: string;
  scopes: string[];
  startIndex: number;
  text: string;
};

function readJson<T>(relativePath: string): T {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf-8")) as T;
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

async function loadBundledLanguageGrammar(name: string) {
  const module = await import(pathToFileURL(path.join(shikiLangsPath, `${name}.mjs`)).href);
  return Array.isArray(module.default) ? module.default[0] : module.default;
}

async function loadVueTextMateGrammar(scopeName = "source.vue") {
  const [{ Registry }, { createOnigurumaEngine }, sourceTs, sourceJson] = await Promise.all([
    import(pathToFileURL(textmateModulePath).href),
    import(pathToFileURL(onigurumaModulePath).href),
    loadBundledLanguageGrammar("typescript"),
    loadBundledLanguageGrammar("json"),
  ]);
  const engine = await createOnigurumaEngine(fs.readFileSync(onigurumaWasmPath));
  const grammars = new Map<string, unknown>([
    ["source.vue", readJson("editors/vscode/syntaxes/vue.tmLanguage.json")],
    ["source.art-vue", readJson("editors/vscode/syntaxes/art-vue.tmLanguage.json")],
    ["source.ts", sourceTs],
    ["source.json", sourceJson],
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

  const grammar = registry.loadGrammar(scopeName);
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

test("vscode-vize template grammar recurses through template content", () => {
  const grammar = readJson<{
    repository?: Record<string, { patterns?: Array<{ include?: string }> }>;
  }>("editors/vscode/syntaxes/vue.tmLanguage.json");
  const repository = grammar.repository ?? {};

  assert.deepEqual(
    (repository["vue-template-content"]?.patterns ?? []).map((pattern) => pattern.include),
    [
      "#vue-comments",
      "#vue-interpolation",
      "#vue-template",
      "#vue-directives",
      "#html-tags",
      "#html-entities",
    ],
  );
  assert.deepEqual(
    (repository["vue-template-tag"]?.patterns ?? []).map((pattern) => pattern.include),
    ["#vue-directive-attributes", "#vue-directives", "#vue-tag-attributes"],
  );
  assert.equal(
    repository["vue-generic-attribute"]?.patterns?.[0]?.patterns?.[0]?.include,
    "#vue-ts-type",
  );
  assert.equal(
    repository["vue-directive-attributes"]?.patterns?.[0]?.patterns?.[0]?.include,
    "#vue-ts-expression",
  );
  assert.match(
    "generic='T extends Foo<User>'",
    new RegExp(repository["vue-generic-attribute"]?.patterns?.[0]?.begin ?? ""),
  );
});

test("vscode-vize grammar keeps Vue scopes after nested template blocks", async () => {
  const { grammar, registry } = await loadVueTextMateGrammar();

  try {
    const tokens = tokenizeLines(grammar, [
      "<template>",
      '  <div class="foo">',
      "    <!-- nested branch -->",
      '    <template v-if="true">',
      "      <Foo>label</Foo>",
      "    </template>",
      "    <div v-else>fallback</div>",
      "",
      '    <Bar :open="false">',
      "      message<br />more",
      "    </Bar>",
      "  </div>",
      "</template>",
    ]);

    assertTextHasScope(tokens, "nested branch", "comment.block.html");
    assertTextHasScope(tokens, "v-if", "keyword.control.directive.vue");
    assertTextHasScope(tokens, "true", "meta.embedded.expression.vue");
    assertTextHasScope(tokens, "v-else", "keyword.control.directive.vue");
    assertTextHasScope(tokens, "open", "entity.other.attribute-name.binding.vue");
    assertTextHasScope(tokens, "false", "meta.embedded.expression.vue");
    assertTextHasScope(tokens, "Bar", "entity.name.tag.html");
    assertTextHasScope(tokens, "br", "entity.name.tag.html");
  } finally {
    registry.dispose();
  }
});

test("vscode-vize grammar keeps TS generics and assertions inside attribute values", async () => {
  const { grammar, registry } = await loadVueTextMateGrammar();

  try {
    const tokens = tokenizeLines(grammar, [
      '<script setup lang="ts" generic="T extends Record<string, unknown> = Foo<User>">',
      "const value = makeValue<T>() as T",
      "</script>",
      "<template>",
      '  <button :items="makeList<User>() as Array<User>" v-bind:[activeKey as keyof Props].camel="makeValue<User>() as User" data-x="ok">',
      "    {{ makeValue<User>() as User }}",
      "  </button>",
      "  <input :value='makeValue<User>() as User' data-single=\"ok\" />",
      "  <span>after</span>",
      "</template>",
    ]);

    assertTextHasScope(tokens, "Record", "meta.embedded.type.typescript");
    assertTextHasScope(tokens, "makeList", "entity.name.function.ts");
    assertTextHasScope(tokens, "as", "keyword.control.as.ts");
    assertTextHasScope(tokens, "Props", "entity.name.type.ts");
    assertTextHasScope(tokens, "data-x", "entity.other.attribute-name.html");
    assertTextHasScope(tokens, "data-single", "entity.other.attribute-name.html");
    assertTextHasScope(tokens, "span", "entity.name.tag.html");
    assertTextDoesNotHaveScope(tokens, "data-x", "meta.embedded.expression.vue");
    assertTextDoesNotHaveScope(tokens, "data-single", "meta.embedded.expression.vue");
    assertTextDoesNotHaveScope(tokens, "span", "meta.embedded.expression.vue");
    assertTextDoesNotHaveScope(tokens, "after", "meta.embedded.type.typescript");
  } finally {
    registry.dispose();
  }
});

test("vscode-vize art grammar preserves Musea art highlighting around TS attributes", async () => {
  const { grammar, registry } = await loadVueTextMateGrammar("source.art-vue");

  try {
    const tokens = tokenizeLines(grammar, [
      '<art title="Button" component="Button">',
      '  <variant name="typed" args=\'{"items":[1]}\'>',
      '    <Button :items="makeList<User>() as Array<User>" v-bind:[activeKey as keyof Props].camel="makeValue<User>() as User" data-x="ok">',
      "      {{ makeValue<User>() as User }}",
      "    </Button>",
      "  </variant>",
      "</art>",
    ]);

    assertTextHasScope(tokens, "art", "entity.name.tag.art.vue");
    assertTextHasScope(tokens, "variant", "entity.name.tag.variant.vue");
    assertTextHasScope(tokens, "makeList", "entity.name.function.ts");
    assertTextHasScope(tokens, "as", "keyword.control.as.ts");
    assertTextHasScope(tokens, "Props", "entity.name.type.ts");
    assertTextHasScope(tokens, "data-x", "entity.other.attribute-name.html");
    assertTextHasScope(tokens, "Button", "entity.name.tag.html");
    assertTextDoesNotHaveScope(tokens, "variant", "meta.embedded.expression.vue");
    assertTextDoesNotHaveScope(tokens, "data-x", "meta.embedded.expression.vue");
  } finally {
    registry.dispose();
  }
});
