import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import ts from "typescript";

import { repoRoot } from "./_helpers/moonbit.ts";

const configurableRuleOptions = [
  "html/no-empty-palpable-content",
  "musea/prefer-design-tokens",
  "script/custom-event-name-casing",
  "script/define-props-destructuring",
  "script/no-restricted-globals",
  "script/no-restricted-members",
  "type/strict-boolean-expressions",
  "vue/attribute-hyphenation",
  "vue/component-name-in-template-casing",
  "vue/html-self-closing",
  "vue/no-mutating-props",
  "vue/require-component-registration",
  "vue/sfc-element-order",
  "vue/v-on-event-hyphenation",
];

test("rule option docs enumerate every typed lint rule option", () => {
  const optionsDoc = readDocs([
    path.join(repoRoot, "docs/content/rules/options.md"),
    path.join(repoRoot, "docs/content/rules/options-script.md"),
    path.join(repoRoot, "docs/content/rules/options-vue.md"),
    path.join(repoRoot, "docs/content/rules/options-musea.md"),
    path.join(repoRoot, "docs/content/rules/options-type.md"),
  ]);

  assertRuleOptionsReference(optionsDoc, {
    title: /# Rule Options/,
    unknownFields: /Unknown\s+option fields are rejected/,
    scopedReplacement: /later matching config entries replace the full option object/,
    badLabel: "Bad",
    goodLabel: "Good",
  });
});

test("Japanese rule option docs mirror every typed lint rule option", () => {
  const optionsDoc = readDocs([
    path.join(repoRoot, "docs/content/ja/rules/options.md"),
    path.join(repoRoot, "docs/content/ja/rules/options-type.md"),
  ]);

  assertRuleOptionsReference(optionsDoc, {
    title: /# ルールのオプション/,
    unknownFields: /定義されていないフィールドを指定するとエラーになります/,
    scopedReplacement: /後から一致する設定がある場合は、\s*そのルールのオプション全体を置き換え/,
    badLabel: "悪い",
    goodLabel: "良い",
  });
});

function assertRuleOptionsReference(
  optionsDoc: string,
  labels: {
    title: RegExp;
    unknownFields: RegExp;
    scopedReplacement: RegExp;
    badLabel: string;
    goodLabel: string;
  },
): void {
  assert.match(optionsDoc, labels.title);
  assert.match(optionsDoc, labels.unknownFields);
  assert.match(optionsDoc, labels.scopedReplacement);

  for (const ruleId of configurableRuleOptions) {
    assert.match(
      optionsDoc,
      new RegExp(escapeRegExp(`| \`${ruleId}\` |`)),
      `${ruleId} must be documented in the lint rule option table`,
    );
    const section = sectionFor(optionsDoc, `## \`${ruleId}\``);
    assert.ok(section.includes(labels.badLabel), `${ruleId} must include a bad example`);
    assert.ok(section.includes(labels.goodLabel), `${ruleId} must include a good example`);
    assert.match(section, /```(?:json|ts|vue)[\s\S]*?```/u, `${ruleId} must include code`);
  }
}

test("configuration docs link to the full lint rule option reference", () => {
  const configuration = fs.readFileSync(
    path.join(repoRoot, "docs/content/guide/configuration.md"),
    "utf8",
  );

  assert.match(configuration, /### Lint Rule Options/);
  assert.match(configuration, /\[Rule Options\]\(\.\.\/rules\/options\.md\)/);

  const jaConfiguration = fs.readFileSync(
    path.join(repoRoot, "docs/content/ja/guide/configuration.md"),
    "utf8",
  );

  assert.match(jaConfiguration, /### ルールのオプション/);
  assert.match(jaConfiguration, /<span id="lint-rule-options"><\/span>/);
  assert.match(jaConfiguration, /\[ルールのオプション\]\(\.\.\/rules\/options\.md\)/);
});

test("rules overview links to lint rule option references", () => {
  const index = fs.readFileSync(path.join(repoRoot, "docs/content/rules/index.md"), "utf8");
  const jaIndex = fs.readFileSync(path.join(repoRoot, "docs/content/ja/rules/index.md"), "utf8");

  assert.match(index, /\[Rule Options\]\(\.\/options\.md\)/);
  assert.match(jaIndex, /\[ルールのオプション\]\(\.\/options\.md\)/);
});

test("all rules reference shows which rules accept lint rule options", () => {
  const allRules = fs.readFileSync(path.join(repoRoot, "docs/content/rules/all.md"), "utf8");

  assert.match(
    allRules,
    /\| Rule \| Examples \| Severity \| Presets \| Fixable \| Options \| Implementation \| Description \| Category \|/,
  );
  assert.match(allRules, /rule-option support/);

  for (const ruleId of configurableRuleOptions) {
    const row = allRules
      .split("\n")
      .find((line) => line.startsWith(`| [\`${ruleId}\`](https://vizejs.dev/rules/all.html#`));
    assert.ok(row, `${ruleId} must appear in all rules`);
    assert.match(
      row,
      /\[`ruleOptions`\]\(\.\/options\.md\)/,
      `${ruleId} must link to lint rule option docs`,
    );
  }

  const a11yRow = allRules
    .split("\n")
    .find((line) => line.startsWith("| [`a11y/img-alt`](https://vizejs.dev/rules/all.html#"));
  assert.ok(a11yRow, "a sample non-configurable rule must appear in all rules");
  assert.match(a11yRow, /\| No \| \[source\]/);
});

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function readDocs(paths: string[]): string {
  return paths.map((filePath) => fs.readFileSync(filePath, "utf8")).join("\n");
}

function sectionFor(source: string, heading: string): string {
  const start = source.indexOf(heading);
  assert.notEqual(start, -1, `missing ${heading}`);
  const next = source.indexOf("\n## ", start + heading.length);
  return source.slice(start, next === -1 ? source.length : next);
}

test("strict boolean rows render within the complete option and type-aware tables", () => {
  const expectedOptions = [
    "html/no-empty-palpable-content",
    "script/no-restricted-globals",
    "script/no-restricted-members",
    "vue/component-name-in-template-casing",
    "script/custom-event-name-casing",
    "script/define-props-destructuring",
    "vue/no-mutating-props",
    "vue/require-component-registration",
    "vue/sfc-element-order",
    "vue/html-self-closing",
    "vue/v-on-event-hyphenation",
    "vue/attribute-hyphenation",
    "vue/valid-v-on",
    "vue/valid-v-slot",
    "musea/prefer-design-tokens",
    "type/strict-boolean-expressions",
  ];
  for (const locale of ["", "ja/"]) {
    const source = fs.readFileSync(
      path.join(repoRoot, `docs/content/${locale}rules/options.md`),
      "utf8",
    );
    assert.deepEqual(tableRules(source).toSorted(), expectedOptions.toSorted(), locale || "en");
  }
  const allRules = fs.readFileSync(path.join(repoRoot, "docs/content/rules/all.md"), "utf8");
  const typeRows = allRules.split("\n").filter((line) => line.startsWith("| [`type/"));
  assert.equal(typeRows.length, 6);
  assert.ok(typeRows.every((row) => row.endsWith("| Type Aware |")));
  assert.deepEqual(
    tableRules(allRules).filter((name) => name.startsWith("type/")),
    [
      "type/no-floating-promises",
      "type/no-reactivity-loss",
      "type/no-unsafe-template-binding",
      "type/require-typed-emits",
      "type/require-typed-props",
      "type/strict-boolean-expressions",
    ],
  );
});

function tableRules(source: string): string[] {
  const block = source
    .split(/\n\s*\n/u)
    .find((block) => /^\| (?:Rule|ルール) \|[^\n]*\n\| --- \|/u.test(block));
  assert.ok(block, "table header and separator must share a block");
  return [...block.matchAll(/^\| (?:\[)?`([^`]+)`(?:\]\([^)]*\))? \|/gmu)].map(
    (match) => match[1]!,
  );
}

void test("documented props rule options typecheck against the generated public configuration API", () => {
  const recipes: string[] = [];
  for (const doc of [
    "rules/options-script.md",
    "ja/rules/options.md",
    "ja/rules/options-script.md",
  ]) {
    const section = sectionFor(
      fs.readFileSync(path.join(repoRoot, "docs/content", doc), "utf8"),
      "## `script/define-props-destructuring`",
    );
    const options = [...section.matchAll(/^\s*ruleOptions: (\{[^\n]+\})[, ]*$/gm)].map(
      (match) => match[1]!,
    );
    assert.equal(options.length, 2, `${doc}: complete setup and migration options`);
    recipes.push(...options);
  }
  const filename = path.join(repoRoot, ".cache/docs-props-options-contract.ts");
  const generated = path.join(repoRoot, "npm/cli/src/types/generated.ts");
  const compilerOptions: ts.CompilerOptions = {
    noEmit: true,
    strict: true,
    skipLibCheck: true,
    types: [],
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.NodeNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
    allowImportingTsExtensions: true,
  };
  const diagnose = (options: string[]) => {
    const source =
      `import type { LintRuleOptions } from ${JSON.stringify(generated)};\n` +
      options.map((value, index) => `const recipe${index}: LintRuleOptions = ${value};`).join("\n");
    const host = ts.createCompilerHost(compilerOptions);
    const original = host.getSourceFile.bind(host);
    host.getSourceFile = (file, languageVersion, onError, fresh) =>
      file === filename
        ? ts.createSourceFile(file, source, languageVersion, true)
        : original(file, languageVersion, onError, fresh);
    return ts.getPreEmitDiagnostics(ts.createProgram([filename], compilerOptions, host));
  };
  assert.deepEqual(
    diagnose(recipes).map((item) => ts.flattenDiagnosticMessageText(item.messageText, "\n")),
    [],
    "every whole documented option object must satisfy the actual generated API",
  );
  const oldField = diagnose(['{ "script/define-props-destructuring": { mode: "always" } }']);
  assert(
    oldField.some((item) => item.code === 2353),
    "the inherited unknown mode field is refused",
  );
});
