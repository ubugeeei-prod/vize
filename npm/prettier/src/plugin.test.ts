import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { formatSfc, parseArt } from "@vizejs/native";
import { check, format, formatWithCursor, getFileInfo, type Options } from "prettier";
import { createPlugin } from "./plugin.ts";
import plugin from "./index.ts";
import {
  assertBlocksEquivalent,
  assertRenderedEquivalent,
  renderTemplate,
} from "../test/semantic.ts";

const root = path.resolve(import.meta.dirname, "../../..");
const input =
  '<script setup lang="ts">const label:string="雪 😀";const value=(x:number)=>({x})</script><template><p>{{ label }}</p></template><style>.note{color:red;padding:2px}</style>';

void test("one native format per nonempty file with explicit host option precedence", async () => {
  const calls: unknown[] = [];
  const counted = createPlugin((source, options) => {
    calls.push(options);
    return formatSfc(source, options);
  });
  const options: Options = {
    plugins: [counted],
    filepath: "App.vue",
    printWidth: 60,
    tabWidth: 4,
    useTabs: true,
    semi: false,
    singleQuote: true,
    jsxSingleQuote: true,
    quoteProps: "consistent",
    trailingComma: "none",
    arrowParens: "avoid",
    bracketSpacing: false,
    bracketSameLine: true,
    singleAttributePerLine: true,
  };
  const actual = await format(input, options);
  assert.equal(calls.length, 1);
  assert.deepEqual(calls[0], {
    preset: "prettier",
    vueVersion: "3",
    printWidth: 60,
    tabWidth: 4,
    useTabs: true,
    semi: false,
    singleQuote: true,
    jsxSingleQuote: true,
    quoteProps: "consistent",
    trailingComma: "none",
    arrowParens: "avoid",
    bracketSpacing: false,
    bracketSameLine: true,
    singleAttributePerLine: true,
    endOfLine: "lf",
  });
  assert.equal(actual, formatSfc(input, calls[0] as Parameters<typeof formatSfc>[1]).code);
  await assertBlocksEquivalent(input, actual);
  assert.equal(await format(actual, options), actual);
  assert.equal(calls.length, 2, "one call per invocation, with no stabilization loop");
  assert.equal(await check(actual, options), true);
  assert.equal(calls.length, 3);
  assert.equal((await getFileInfo("App.vue", { plugins: [plugin] })).inferredParser, "vize");
  assert.equal(
    (await getFileInfo("Example.art", { plugins: [plugin] })).inferredParser,
    "vize-art",
  );
});

void test("shared complete packets retain goldens, semantics and fixed points", async () => {
  for (const fixtureFile of ["directives.json", "text.json"]) {
    const fixtures = JSON.parse(
      fs.readFileSync(
        path.join(root, "crates/vize_glyph/tests/fixtures/preset-layout-10609", fixtureFile),
        "utf8",
      ),
    ) as { id: string; input: string; expected?: string; printWidth: number }[];
    for (const fixture of fixtures) {
      for (const [endOfLine, newline] of [
        ["lf", "\n"],
        ["crlf", "\r\n"],
        ["cr", "\r"],
      ] as const) {
        const source = fixture.input.replaceAll("\n", newline);
        const options: Options = {
          filepath: "Packet.vue",
          plugins: [plugin],
          printWidth: fixture.printWidth,
          trailingComma: "none",
          arrowParens: "avoid",
          endOfLine,
        };
        const actual = await format(source, options);
        if (fixture.expected)
          assert.equal(actual, fixture.expected.replaceAll("\n", newline), fixture.id);
        assert.equal(await format(actual, options), actual, `${fixture.id}: second pass`);
        assert.equal(
          await format(await format(actual, options), options),
          actual,
          `${fixture.id}: third pass`,
        );
        await assertBlocksEquivalent(source, actual);
        await assertRenderedEquivalent(source, actual);
      }
    }
  }
});

void test("literal-line Docs preserve indentation, trailing spaces and all host line endings", async () => {
  const literal = "<template>\n  <pre>first  \n\t  second\t \nthird</pre>\n</template>\n";
  let calls = 0;
  const counted = createPlugin(() => {
    calls++;
    return { code: literal, changed: true };
  });
  for (const [endOfLine, newline] of [
    ["lf", "\n"],
    ["crlf", "\r\n"],
    ["cr", "\r"],
    ["auto", "\r\n"],
  ] as const) {
    assert.equal(
      await format(literal.replaceAll("\n", newline), {
        filepath: "Raw.vue",
        plugins: [counted],
        endOfLine,
      }),
      literal.replaceAll("\n", newline),
    );
  }
  assert.equal(calls, 4);
  assert.equal(await format("", { plugins: [plugin], filepath: "Empty.vue" }), "");
});

void test("cursor offsets remain UTF-16 positions across Unicode, BOM and CRLF", async () => {
  for (const newline of ["\n", "\r\n", "\r"]) {
    const source = `\uFEFF<template>${newline}<p>雪 😀 marker</p>${newline}</template>`;
    for (const delta of [0, 1, 3, 5]) {
      const before = source.indexOf("雪 😀") + delta;
      const result = await formatWithCursor(source, {
        filepath: "Cursor.vue",
        plugins: [plugin],
        endOfLine: "auto",
        cursorOffset: before,
      });
      assert.equal(result.cursorOffset, result.formatted.indexOf("雪 😀") + delta);
      assert.equal(result.formatted.startsWith("\uFEFF"), true);
      assert.equal(result.formatted.includes(newline), true);
      const inserted = await formatWithCursor(source, {
        filepath: "Cursor.vue",
        plugins: [plugin],
        endOfLine: "auto",
        cursorOffset: before,
        insertPragma: true,
      });
      assert.equal(inserted.cursorOffset, inserted.formatted.indexOf("雪 😀") + delta);
    }
  }
});

void test("host pragmas bypass native work and script ignore remains owned by the engine", async () => {
  let calls = 0;
  const counted = createPlugin((source, options) => {
    calls++;
    return formatSfc(source, options);
  });
  assert.equal(
    await format(input, { filepath: "App.vue", plugins: [counted], requirePragma: true }),
    input,
  );
  assert.equal(calls, 0);
  const ignored = `<!-- @noprettier -->\n${input}`;
  assert.equal(
    await format(ignored, { filepath: "App.vue", plugins: [counted], checkIgnorePragma: true }),
    ignored,
  );
  assert.equal(calls, 0);
  const inserted = await format(input, {
    filepath: "App.vue",
    plugins: [counted],
    insertPragma: true,
  });
  assert.equal(inserted.startsWith("<!-- @format -->"), true);
  assert.equal(calls, 1);
  const script =
    '<script setup>\n// prettier-ignore\nconst keep ={ x:1, y:  2 };\nconst data="<!-- prettier-ignore -->";\n</script><template><p>hello</p></template><style>.a::before{content:"/* prettier-ignore */"}</style>';
  const actual = await format(script, { filepath: "Ignored.vue", plugins: [counted] });
  assert.equal(actual.includes("const keep ={ x:1, y:  2 };"), true);
  assert.equal(calls, 2);
});

void test("unsupported option and ignore requests fail before native formatting", async () => {
  let calls = 0;
  const counted = createPlugin(() => {
    calls++;
    return { code: "", changed: false };
  });
  for (const unsupported of [
    { htmlWhitespaceSensitivity: "ignore" },
    { vueIndentScriptAndStyle: true },
    { embeddedLanguageFormatting: "off" },
    { proseWrap: "always" },
    { objectWrap: "collapse" },
    { experimentalTernaries: true },
    { experimentalOperatorPosition: "start" },
    { printWidth: 0 },
    { printWidth: 321 },
    { tabWidth: 25 },
    { rangeStart: 3 },
    { rangeEnd: 5 },
  ]) {
    await assert.rejects(
      format(input, { filepath: "Unsupported.vue", plugins: [counted], ...unsupported } as Options),
      /prettier-plugin-vize/u,
    );
  }
  for (const source of [
    '<template><!-- prettier-ignore --><p  id="x">x</p></template>',
    "<template><!-- prettier-ignore-start --><p>x</p><!-- prettier-ignore-end --></template>",
    "<style>/* prettier-ignore */ .a{ color : red }</style>",
  ])
    await assert.rejects(
      format(source, { filepath: "Unsupported.vue", plugins: [counted] }),
      /does not support/u,
    );
  assert.equal(calls, 0);
});

void test("Art uses the existing engine and retains variant metadata and rendering", async () => {
  const source =
    '<script setup lang="ts">const label:string="雪"</script><art title="Demo"><variant name="Basic"><p class="item">{{ label }}</p></variant></art><style>.item{color:blue}</style>';
  const actual = await format(source, { filepath: "Example.art", plugins: [plugin] });
  assert.equal(actual, formatSfc(source, { preset: "prettier", endOfLine: "lf" }).code);
  assert.equal(await format(actual, { filepath: "Example.art", plugins: [plugin] }), actual);
  const before = parseArt(source);
  const after = parseArt(actual);
  const semanticVariants = (value: typeof after) => ({
    ...value,
    variants: value.variants.map(({ template: _template, ...metadata }) => metadata),
  });
  assert.deepEqual(semanticVariants(after), semanticVariants(before));
  assert.deepEqual(after.metadata, before.metadata);
  assert.equal(after.variants.length, 1);
  assert.equal(after.variants[0].name, "Basic");
  assert.equal(
    await renderTemplate(after.variants[0].template, { label: "雪 😀" }),
    await renderTemplate(before.variants[0].template, { label: "雪 😀" }),
  );
  await assertBlocksEquivalent(source, actual);
});

void test("semantic oracles reject changed imports, style values and rendered adjacency", async () => {
  await assert.rejects(
    assertBlocksEquivalent(
      '<script>import value from "./before"</script>',
      '<script>import value from "./after"</script>',
    ),
    { name: "AssertionError" },
  );
  await assert.rejects(
    assertBlocksEquivalent("<style>.a{color:red}</style>", "<style>.a{color:blue}</style>"),
    { name: "AssertionError" },
  );
  await assert.rejects(
    assertRenderedEquivalent(
      "<template><p><span>left</span>{{ label }}</p></template>",
      "<template><p><span>left</span> {{ label }}</p></template>",
    ),
    { name: "AssertionError" },
  );
});

void test("explicit parser aliases and Vue grammar options use the same native pass", async () => {
  const calls: string[] = [];
  const counted = createPlugin((source, options) => {
    calls.push(options.vueVersion ?? "");
    return formatSfc(source, options);
  });
  for (const parser of ["vize", "vue", "vize-art"]) {
    const actual = await format(input, {
      parser,
      plugins: [counted],
      vizeVueVersion: "2",
    } as Options);
    assert.equal(
      actual,
      formatSfc(input, { preset: "prettier", vueVersion: "2", endOfLine: "lf" }).code,
    );
  }
  assert.deepEqual(calls, ["2", "2", "2"]);
});
