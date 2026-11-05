import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import ts from "typescript";

import { repoRoot } from "./_helpers/tool-commands.ts";
import { isOperationalDiagnosticReference } from "../../docs/scripts/rules/operational-diagnostics.ts";

const configurableRuleOptions = [
  "html/no-empty-palpable-content",
  "musea/prefer-design-tokens",
  "script/custom-event-name-casing",
  "script/define-macros-order",
  "script/define-props-destructuring",
  "script/no-restricted-globals",
  "script/no-restricted-members",
  "type/strict-boolean-expressions",
  "vue/attribute-hyphenation",
  "vue/component-name-in-template-casing",
  "vue/html-self-closing",
  "vue/html-quotes",
  "vue/mustache-interpolation-spacing",
  "vue/no-mutating-props",
  "vue/prop-name-casing",
  "vue/require-component-registration",
  "vue/sfc-element-order",
  "vue/v-on-event-hyphenation",
  "vue/v-slot-style",
];

test("rule option docs enumerate every typed lint rule option", () => {
  const optionsDoc = readDocs([
    path.join(repoRoot, "docs/content/rules/options.md"),
    path.join(repoRoot, "docs/content/rules/options-script.md"),
    path.join(repoRoot, "docs/content/rules/options-vue.md"),
    path.join(repoRoot, "docs/content/rules/options-attributes.md"),
    path.join(repoRoot, "docs/content/rules/options-props.md"),
    path.join(repoRoot, "docs/content/rules/options-musea.md"),
    path.join(repoRoot, "docs/content/rules/options-type.md"),
  ]);

  assertRuleOptionsReference(optionsDoc, {
    title: /# Rule Options/,
    unknownFields: /Unknown\s+option fields are rejected/,
    scopedReplacement: /later matching config entries replace the full option object/,
    badLabel: /Bad/,
    goodLabel: /Good/,
  });
});

test("Japanese rule option docs mirror every typed lint rule option", () => {
  const optionsDoc = readDocs([
    path.join(repoRoot, "docs/content/ja/rules/options.md"),
    path.join(repoRoot, "docs/content/ja/rules/options-script.md"),
    path.join(repoRoot, "docs/content/ja/rules/options-vue.md"),
    path.join(repoRoot, "docs/content/ja/rules/options-attributes.md"),
    path.join(repoRoot, "docs/content/ja/rules/options-props.md"),
    path.join(repoRoot, "docs/content/ja/rules/options-musea.md"),
    path.join(repoRoot, "docs/content/ja/rules/options-type.md"),
  ]);

  assertRuleOptionsReference(optionsDoc, {
    title: /# ルールのオプション/,
    unknownFields: /定義されていないフィールドを指定するとエラーになります/,
    scopedReplacement: /後から一致する設定がある場合は、\s*そのルールのオプション全体を置き換え/,
    badLabel: /悪い|報告される/,
    goodLabel: /良い|沿った|許可される/,
  });
});

function assertRuleOptionsReference(
  optionsDoc: string,
  labels: {
    title: RegExp;
    unknownFields: RegExp;
    scopedReplacement: RegExp;
    badLabel: RegExp;
    goodLabel: RegExp;
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
    assert.match(section, labels.badLabel, `${ruleId} must include a bad example`);
    assert.match(section, labels.goodLabel, `${ruleId} must include a good example`);
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

test("every configurable rule's page links to its typed options", () => {
  const reference = (locale: string, rule: string) =>
    fs.readFileSync(
      path.join(repoRoot, `docs/content/${locale}rules/reference/${rule.replaceAll("/", "-")}.md`),
      "utf8",
    );
  const facts = (page: string) =>
    page.split("\n").find((line) => /^\| `(?:error|warning)` \|/.test(line))!;
  const table = tableRules(
    fs.readFileSync(path.join(repoRoot, "docs/content/rules/options.md"), "utf8"),
  );
  for (const ruleId of [...configurableRuleOptions, ...table]) {
    assert.match(
      facts(reference("", ruleId)),
      /\| \[Rule options\]\(\.\.\/options\.md\) \|$/,
      ruleId,
    );
    assert.match(
      facts(reference("ja/", ruleId)),
      /\| \[ルールのオプション\]\(\.\.\/options\.md\) \|$/,
      ruleId,
    );
    assert.match(
      facts(reference("fr/", ruleId)),
      /\| \[Options des règles\]\(\/rules\/options\.md\) \|$/,
      ruleId,
    );
  }
  assert.match(facts(reference("", "a11y/img-alt")), /\| None \|$/, "a rule without options");
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
    "script/define-macros-order",
    "vue/component-name-in-template-casing",
    "script/custom-event-name-casing",
    "vue/prop-name-casing",
    "script/define-props-destructuring",
    "vue/no-mutating-props",
    "vue/require-component-registration",
    "vue/sfc-element-order",
    "vue/html-self-closing",
    "vue/html-quotes",
    "vue/mustache-interpolation-spacing",
    "vue/v-on-event-hyphenation",
    "vue/v-bind-style",
    "vue/v-on-style",
    "vue/v-slot-style",
    "vue/attribute-hyphenation",
    "vue/valid-v-slot",
    "musea/prefer-design-tokens",
    "type/strict-boolean-expressions",
  ];
  for (const locale of ["", "ja/"]) {
    const source = fs.readFileSync(
      path.join(repoRoot, `docs/content/${locale}rules/options.md`),
      "utf8",
    );
    const currentOptions = tableRules(source);
    assert.equal(currentOptions.length, expectedOptions.length + 1, locale || "en");
    assert.equal(
      currentOptions[4],
      "vue/valid-v-on",
      "the single new modifier option follows the original restriction rules",
    );
    assert.deepEqual(
      currentOptions.filter((rule) => rule !== "vue/valid-v-on"),
      expectedOptions,
      locale || "en",
    );
    assert.match(
      source.split("\n").find((line) => line.startsWith("| `vue/valid-v-on` |"))!,
      /`\{ modifiers\?: string\[\] \}`/,
    );
  }
  const reference = path.join(repoRoot, "docs/content/rules/reference");
  const typeRules = fs
    .readdirSync(reference)
    .filter((file) => file.startsWith("type-") && !isOperationalDiagnosticReference(file))
    .map((file) => fs.readFileSync(path.join(reference, file), "utf8"));
  for (const page of typeRules) {
    assert.match(page, /"typeAware": true/, "type-aware configuration");
    assert.match(page, /reads type information from your TypeScript project/);
  }
  assert.deepEqual(
    typeRules.map((page) => page.match(/^# `([^`]+)`/m)![1]),
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
  for (const doc of ["rules/options-script.md", "ja/rules/options-script.md"]) {
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
  for (const doc of ["rules/options-props.md", "ja/rules/options-props.md"]) {
    const section = sectionFor(
      fs.readFileSync(path.join(repoRoot, "docs/content", doc), "utf8"),
      "## `vue/prop-name-casing`",
    );
    const options = [...section.matchAll(/^\s*ruleOptions: (\{[^\n]+\})[, ]*$/gm)].map(
      (match) => match[1]!,
    );
    assert.equal(options.length, 1, `${doc}: complete declared-prop casing options`);
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
  const invalidCasing = diagnose(['{ "vue/prop-name-casing": { casing: "unsupported" } }']);
  assert(
    invalidCasing.some((item) => item.code === 2322),
    "invalid declared-prop casing is refused",
  );
  const unsupportedIgnore = diagnose(['{ "vue/prop-name-casing": { ignoreProps: ["myProp"] } }']);
  assert(
    unsupportedIgnore.some((item) => item.code === 2353),
    "unexposed ignoreProps is refused",
  );
  const oldField = diagnose(['{ "script/define-props-destructuring": { mode: "always" } }']);
  assert(
    oldField.some((item) => item.code === 2353),
    "the inherited unknown mode field is refused",
  );
});
