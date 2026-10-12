import assert from "node:assert/strict";
import { createRequire } from "node:module";
import plugin from "../dist/eslint.mjs";
import oxlintPlugin from "../dist/index.mjs";
import {
  qualifyReferenceHost,
  referenceQualificationSelected,
} from "./reference-host-acceptance.mjs";

// The workspace's declared Nuxt oracle owns these real installed consumers.
const workspace = createRequire(new URL("../../../package.json", import.meta.url));
const oracle = createRequire(workspace.resolve("@nuxt/eslint-config"));
const { Linter, SourceCode } = oracle("eslint");
const parser = oracle("vue-eslint-parser");
const require = createRequire(import.meta.url);
const native = require("@vizejs/native");

function config(rules) {
  return [
    {
      files: ["**/*.vue"],
      languageOptions: { parser },
      plugins: { vize: plugin },
      settings: { vize: { preset: "incremental", helpLevel: "none" } },
      rules,
    },
  ];
}

const source =
  '<template>😀<img src="logo.png"></template>\r\n' +
  "<script>export default {};</script>\r\n" +
  "<script setup>const value = 1;</script>\r\n" +
  "<style scoped>img { display: block; }</style>\r\n";
const alt = { "vize/a11y/alt-text": "error" };
const linter = new Linter();
const messages = linter.verify(source, config(alt), { filename: "Unsaved.vue" });
assert.equal(messages.length, 1);
assert.deepEqual(
  {
    rule: messages[0].ruleId,
    severity: messages[0].severity,
    line: messages[0].line,
    column: messages[0].column,
    endLine: messages[0].endLine,
    endColumn: messages[0].endColumn,
  },
  { rule: "vize/a11y/alt-text", severity: 2, line: 1, column: 13, endLine: 1, endColumn: 33 },
);
assert.equal(messages[0].message, "<img> elements must have an alt attribute");
assert.equal(messages[0].suggestions, undefined);
const clean = source.replace('<img src="logo.png">', '<img src="logo.png" alt="Logo">');
assert.deepEqual(linter.verify(clean, config(alt), { filename: "Unsaved.vue" }), []);
assert.deepEqual(
  linter.verify(source, config({ "vize/a11y/alt-text": "off" }), { filename: "Unsaved.vue" }),
  [],
);

// Real parser-owned AST reuse must not borrow a prior file's participation.
const parsed = parser.parseForESLint(source, {
  ecmaVersion: "latest",
  sourceType: "module",
  range: true,
  loc: true,
  tokens: true,
  comment: true,
});
const reused = new SourceCode({ ...parsed, text: source, parserServices: parsed.services });
const reusableConfig = config(alt);
reusableConfig[0].files.push("**/*.txt");
assert.equal(linter.verify(reused, reusableConfig, { filename: "Repeated.vue" }).length, 1);
assert.deepEqual(linter.verify(reused, reusableConfig, { filename: "Repeated.txt" }), []);
const excluded = config(alt);
excluded[0].settings.vize.preset = "essential";
assert.deepEqual(linter.verify(reused, excluded, { filename: "Repeated.vue" }), []);
assert.equal(linter.verify(reused, reusableConfig, { filename: "Repeated.vue" }).length, 1);

// Actual parser refusal happens before Program; it must not become clean lint.
const refused = linter.verify("<template></template><script>const = ;</script>", config(alt), {
  filename: "Invalid.vue",
});
assert(refused.some((message) => message.fatal && message.severity === 2));

// Full original source survives multiple script blocks and template/style work.
const multiple =
  '<template><img><div v-for="item in items">{{ item }}</div></template>\n' +
  '<script>import { defineProps } from "vue"; export default {};</script>\n' +
  '<script setup>import { defineEmits } from "vue"; const items = [];</script>\n' +
  "<style>div { color: red; }</style>\n";
const names = [
  "a11y/alt-text",
  "vue/require-v-for-key",
  "script/no-import-compiler-macros",
  "vue/require-scoped-style",
];
const enabled = Object.fromEntries(names.map((name) => [`vize/${name}`, "error"]));
const nativeMessages = native.lintPatinaSfc(multiple, {
  filename: "Multiple.vue",
  preset: "incremental",
  enabledRules: names,
}).diagnostics;
const actual = linter.verify(multiple, config(enabled), { filename: "Multiple.vue" });
assert.deepEqual(
  new Set(actual.map((message) => message.ruleId)),
  new Set(names.map((name) => `vize/${name}`)),
);
assert.equal(actual.length, nativeMessages.length);
for (const message of actual) {
  assert(
    nativeMessages.some(
      (diagnostic) =>
        `vize/${diagnostic.rule}` === message.ruleId &&
        diagnostic.location.start.line === message.line &&
        diagnostic.location.start.column === message.column &&
        diagnostic.location.end.line === message.endLine &&
        diagnostic.location.end.column === message.endColumn,
    ),
  );
}

// Fix edits are real native edits; the absent/false transport stays unchanged.
const before = source.replace('<img src="logo.png">', '<div  id="x"></div>');
const after = before.replace('<div  id="x">', '<div id="x">');
const options = {
  filename: "Fix.vue",
  preset: "incremental",
  enabledRules: ["vue/no-multi-spaces"],
};
const original = native.lintPatinaSfc(before, options);
assert.deepEqual(native.lintPatinaSfc(before, { ...options, includeFixes: false }), original);
assert.equal(
  JSON.stringify(native.lintPatinaSfc(before, { ...options, includeFixes: false })),
  JSON.stringify(original),
);
const detailed = native.lintPatinaSfc(before, { ...options, includeFixes: true });
assert(detailed.diagnostics.some((diagnostic) => diagnostic.fix?.edits.length));
assert.deepEqual(
  {
    ...detailed,
    diagnostics: detailed.diagnostics.map(({ fix: _fix, ...diagnostic }) => diagnostic),
  },
  original,
);
const fixed = linter.verifyAndFix(before, config({ "vize/vue/no-multi-spaces": "error" }), {
  filename: "Fix.vue",
});
assert.equal(fixed.fixed, true);
assert.equal(fixed.output, after);
assert.deepEqual(fixed.messages, []);

// Changing a real reused AST's configured options must produce a fresh pass.
const optionSource =
  "<template><my-component /></template>\n" +
  '<script setup>import MyComponent from "./MyComponent.vue";</script>\n';
const optionParsed = parser.parseForESLint(optionSource, {
  ecmaVersion: "latest",
  sourceType: "module",
  range: true,
  loc: true,
  tokens: true,
  comment: true,
});
const optionCode = new SourceCode({
  ...optionParsed,
  text: optionSource,
  parserServices: optionParsed.services,
});
const casing = (value) =>
  config({ "vize/vue/component-name-in-template-casing": ["error", value] });
assert.equal(
  linter.verify(optionCode, casing("PascalCase"), { filename: "Options.vue" }).length,
  1,
);
assert.deepEqual(linter.verify(optionCode, casing("kebab-case"), { filename: "Options.vue" }), []);
assert.equal(
  linter.verify(optionCode, casing("PascalCase"), { filename: "Options.vue" }).length,
  1,
);

// The native emitter's line convention differs for these separators. Refuse
// inconsistent metadata visibly rather than silently moving the diagnostic.
for (const separator of ["\r", "\u2028", "\u2029"]) {
  assert.throws(
    () =>
      linter.verify(`<template>${separator}<img></template>`, config(alt), {
        filename: "Separators.vue",
      }),
    /location metadata/,
  );
}

// Invalid UTF-16 cannot borrow normalized native text for an unsaved fix.
for (const surrogate of ["\ud800", "\udfff"]) {
  assert.throws(
    () =>
      linter.verify(`<template>${surrogate}<img></template>`, config(alt), {
        filename: "Surrogates.vue",
      }),
    /unpaired UTF-16 surrogate/,
  );
}

// A separate export must not convert or change the existing Oxlint rules.
for (const rule of Object.values(oxlintPlugin.rules)) {
  assert.equal(typeof rule.createOnce, "function");
  assert.equal(rule.create, undefined);
}
const referenceSelected = referenceQualificationSelected();
if (referenceSelected) {
  qualifyReferenceHost({
    oracle,
    nativePath: require.resolve("@vizejs/native"),
    source,
    clean,
    before,
    after,
  });
}
console.log(
  JSON.stringify({
    consumer: "eslint",
    version: Linter.version,
    parser: parser.meta,
    sourceAuthority: "unsaved full source",
    referenceHost: referenceSelected ? "qualified pinned prerelease" : "unqualified; not selected",
    controls: [
      "positive",
      "negative",
      "off",
      "parse-refusal",
      "multiple-blocks",
      "UTF-16",
      "CRLF",
      "fix-bytes",
      "native-default-JSON",
      "no-invented-suggestions",
      "unpaired-surrogate-refusal",
      "default-export",
    ],
  }),
);
