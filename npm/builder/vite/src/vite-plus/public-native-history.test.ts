import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { formatSfc, type FormatOptionsNapi } from "@vizejs/native";
import { runPublicNativeFormatter } from "../../../../../tests/differential/formatter-native.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../../..");
void test(
  "public NAPI sorting retains whole independent results/errors and actual build custody",
  {
    skip: process.env.GITHUB_ACTIONS !== "true" || process.platform !== "linux",
  },
  () => {
    const report = runPublicNativeFormatter(root);
    assert.deepEqual(report.summary, {
      plannedCases: 9,
      legacyByteMatches: 8,
      legacyErrorMatches: 1,
      legacyFailures: 0,
      nativeUnsupported: 9,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    });
    assert.equal(
      report.rows.reduce((count, row) => count + row.passes.length, 0),
      25,
    );
  },
);

void test("public formatter presets retain whole outputs and fixed points", () => {
  const source = `<template>
  <div v-bind:id="id" class="a">{{ id }}</div>
</template>

<script lang="ts" setup>
const id = "a";
</script>
`;
  const vize = `<script setup lang="ts">
const id = "a";
</script>

<template>
  <div :id="id" class="a">{{ id }}</div>
</template>
`;
  const compact = `<template>
  <div :id="id" class="a">{{ id }}</div>
</template>

<script lang="ts" setup>
const id = "a";
</script>
`;
  const documentation = `<template>
  <div
    :id="id"
    class="a"
  >{{ id }}</div>
</template>

<script lang="ts" setup>
const id = "a";
</script>
`;
  assert.deepEqual(formatSfc(source), { code: vize, changed: true });
  for (const [options, expected] of [
    [{}, vize],
    [{ preset: "vize" }, vize],
    [{ preset: "prettier" }, source],
    [{ preset: "compact" }, compact],
    [{ preset: "documentation" }, documentation],
  ] satisfies Array<[FormatOptionsNapi, string]>) {
    Object.freeze(options);
    assert.deepEqual(formatSfc(source, options), { code: expected, changed: expected !== source });
    for (let pass = 2; pass <= 3; pass += 1) {
      assert.deepEqual(formatSfc(expected, options), { code: expected, changed: false });
    }
  }
});

void test("public Prettier preset reuses CSS, inline-style and void-element policies", () => {
  const controls = [
    [
      "<style>\n@import 'a.css';\n.a {\n  content: 'it\\'s';\n  font-family: 'A \"B\"';\n}\n</style>\n",
      '<style>\n@import "a.css";\n.a {\n  content: "it\'s";\n  font-family: \'A "B"\';\n}\n</style>\n',
    ],
    [
      '<template>\n  <div style="color:red;top:0;" :style="{ a: 1 }">a</div>\n</template>\n',
      '<template>\n  <div style="color: red; top: 0" :style="{ a: 1 }">a</div>\n</template>\n',
    ],
    [
      '<template>\n  <ul>\n    <li v-for="item, i in items" :key="i"><img :src="item"><br></li>\n    <li v-for="{ a, b } of pairs" :key="a">{{ b }}</li>\n    <Input v-for="x in xs" :key="x"></Input>\n  </ul>\n</template>\n',
      '<template>\n  <ul>\n    <li v-for="(item, i) in items" :key="i"><img :src="item" /><br /></li>\n    <li v-for="{ a, b } of pairs" :key="a">{{ b }}</li>\n    <Input v-for="x in xs" :key="x"></Input>\n  </ul>\n</template>\n',
    ],
    [
      '<style lang="scss">\n.a {\n\tcolor: red;\n\n\n\t&:hover {\n\t\tcolor: blue;\n\t}\n}\n</style>\n',
      '<style lang="scss">\n.a {\n  color: red;\n\n  &:hover {\n    color: blue;\n  }\n}\n</style>\n',
    ],
  ] as const;
  const options = Object.freeze({ preset: "prettier" as const });
  for (const [source, expected] of controls) {
    assert.deepEqual(formatSfc(source, options), { code: expected, changed: true });
    for (let pass = 2; pass <= 3; pass += 1) {
      assert.deepEqual(formatSfc(expected, options), { code: expected, changed: false });
    }
  }
});

void test("public formatter exposes each Prettier normalizer as an explicit Node option", () => {
  const controls = [
    [
      "<style>\n@import 'a.css';\n.a {\n  content: 'it\\'s';\n  font-family: 'A \"B\"';\n}\n</style>\n",
      '<style>\n@import "a.css";\n.a {\n  content: "it\'s";\n  font-family: \'A "B"\';\n}\n</style>\n',
    ],
    [
      '<template>\n  <div style="color:red;top:0;" :style="{ a: 1 }">a</div>\n</template>\n',
      '<template>\n  <div style="color: red; top: 0" :style="{ a: 1 }">a</div>\n</template>\n',
    ],
    [
      '<template>\n  <ul>\n    <li v-for="item, i in items" :key="i"><img :src="item"><br></li>\n    <li v-for="{ a, b } of pairs" :key="a">{{ b }}</li>\n    <Input v-for="x in xs" :key="x"></Input>\n  </ul>\n</template>\n',
      '<template>\n  <ul>\n    <li v-for="(item, i) in items" :key="i"><img :src="item" /><br /></li>\n    <li v-for="{ a, b } of pairs" :key="a">{{ b }}</li>\n    <Input v-for="x in xs" :key="x"></Input>\n  </ul>\n</template>\n',
    ],
    [
      '<style lang="scss">\n.a {\n\tcolor: red;\n\n\n\t&:hover {\n\t\tcolor: blue;\n\t}\n}\n</style>\n',
      '<style lang="scss">\n.a {\n  color: red;\n\n  &:hover {\n    color: blue;\n  }\n}\n</style>\n',
    ],
  ] as const;
  const enabled = Object.freeze({
    normalizeCssQuotes: true,
    normalizeInlineStyles: true,
    normalizeVForParens: true,
    selfCloseVoidElements: true,
    reindentPreprocessorStyles: true,
  }) satisfies FormatOptionsNapi;
  const disabled = Object.freeze({
    preset: "prettier" as const,
    normalizeCssQuotes: false,
    normalizeInlineStyles: false,
    normalizeVForParens: false,
    selfCloseVoidElements: false,
    reindentPreprocessorStyles: false,
  }) satisfies FormatOptionsNapi;
  for (const [source, expected] of controls) {
    assert.deepEqual(formatSfc(source, enabled), { code: expected, changed: true });
    for (let pass = 2; pass <= 3; pass += 1) {
      assert.deepEqual(formatSfc(expected, enabled), { code: expected, changed: false });
    }
    assert.deepEqual(formatSfc(source, disabled), { code: source, changed: false });
  }
  assert.deepEqual(enabled, {
    normalizeCssQuotes: true,
    normalizeInlineStyles: true,
    normalizeVForParens: true,
    selfCloseVoidElements: true,
    reindentPreprocessorStyles: true,
  });
});

void test("explicit native formatter values override presets including false", () => {
  const source = `<template>
  <div v-bind:id="id" class="a">{{ id }}</div>
</template>

<script lang="ts" setup>
const id = "a";
</script>
`;
  const options = Object.freeze({
    preset: "vize" as const,
    sortAttributes: false,
    normalizeDirectiveShorthands: false,
    semi: false,
    singleQuote: true,
  });
  const expected = `<script lang="ts" setup>
const id = 'a'
</script>

<template>
  <div v-bind:id="id" class="a">{{ id }}</div>
</template>
`;
  assert.deepEqual(formatSfc(source, options), { code: expected, changed: true });
  assert.deepEqual(formatSfc(expected, options), { code: expected, changed: false });
  assert.deepEqual(options, {
    preset: "vize",
    sortAttributes: false,
    normalizeDirectiveShorthands: false,
    semi: false,
    singleQuote: true,
  });
});

void test("public formatter exposes independent JSX and property quote policies", () => {
  const source = `<script setup lang="tsx">
const value={"name":1,plain:2}
const view=<div title="world"/>
</script>
`;
  const expected = `<script setup lang="tsx">
const value = { "name": 1, plain: 2 };
const view = <div title='world' />;
</script>
`;
  const options = Object.freeze({
    singleQuote: false,
    jsxSingleQuote: true,
    quoteProps: "preserve" as const,
  }) satisfies FormatOptionsNapi;
  assert.deepEqual(formatSfc(source, options), { code: expected, changed: true });
  for (let pass = 2; pass <= 3; pass += 1) {
    assert.deepEqual(formatSfc(expected, options), { code: expected, changed: false });
  }
  assert.deepEqual(options, {
    singleQuote: false,
    jsxSingleQuote: true,
    quoteProps: "preserve",
  });
});

void test("public formatter exposes the trailing comma policy", () => {
  const source = `<script setup lang="ts">
const value={first:1,second:2,}
const result=make(
  value,
  true,
)
</script>
`;
  const expected = `<script setup lang="ts">
const value = { first: 1, second: 2 };
const result = make(value, true);
</script>
`;
  const options = Object.freeze({ trailingComma: "none" as const }) satisfies FormatOptionsNapi;
  assert.deepEqual(formatSfc(source, options), { code: expected, changed: true });
  for (let pass = 2; pass <= 3; pass += 1) {
    assert.deepEqual(formatSfc(expected, options), { code: expected, changed: false });
  }
  assert.deepEqual(options, { trailingComma: "none" });
});

void test("public formatter exposes the arrow parentheses policy", () => {
  const source = `<script setup lang="ts">
const mapper=(value)=>value+1
</script>
`;
  const expected = `<script setup lang="ts">
const mapper = value => value + 1;
</script>
`;
  const options = Object.freeze({ arrowParens: "avoid" as const }) satisfies FormatOptionsNapi;
  assert.deepEqual(formatSfc(source, options), { code: expected, changed: true });
  for (let pass = 2; pass <= 3; pass += 1) {
    assert.deepEqual(formatSfc(expected, options), { code: expected, changed: false });
  }
  assert.deepEqual(options, { arrowParens: "avoid" });
});

void test("public formatter rejects unknown presets and wrong JavaScript types", () => {
  for (const preset of ["", "docs", "PRETTIER", 1, true, {}, []]) {
    assert.throws(
      () => formatSfc("<template />", { preset } as FormatOptionsNapi),
      (error: unknown) => {
        assert.ok(error instanceof Error);
        assert.equal(
          (error as Error & { code: string }).code,
          typeof preset === "string" ? "InvalidArg" : "StringExpected",
        );
        if (typeof preset === "string") {
          assert.equal(
            error.message,
            "Unknown formatter preset; expected 'vize', 'prettier', 'compact', or 'documentation'",
          );
        } else {
          assert.match(error.message, /preset|string/i);
        }
        return true;
      },
    );
  }
  assert.throws(
    () => formatSfc("<template />", { quoteProps: "keep" } as FormatOptionsNapi),
    (error: unknown) => {
      assert.ok(error instanceof Error);
      assert.equal((error as Error & { code: string }).code, "InvalidArg");
      assert.equal(
        error.message,
        "Unknown quoteProps; expected 'as-needed', 'consistent', or 'preserve'",
      );
      return true;
    },
  );
  assert.throws(
    () => formatSfc("<template />", { trailingComma: "never" } as FormatOptionsNapi),
    (error: unknown) => {
      assert.ok(error instanceof Error);
      assert.equal((error as Error & { code: string }).code, "InvalidArg");
      assert.equal(error.message, "Unknown trailingComma; expected 'all', 'es5', or 'none'");
      return true;
    },
  );
  assert.throws(
    () => formatSfc("<template />", { arrowParens: "preserve" } as FormatOptionsNapi),
    (error: unknown) => {
      assert.ok(error instanceof Error);
      assert.equal((error as Error & { code: string }).code, "InvalidArg");
      assert.equal(error.message, "Unknown arrowParens; expected 'always' or 'avoid'");
      return true;
    },
  );
});
