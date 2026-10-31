import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { formatSfc, type FormatOptionsNapi } from "../../npm/native/index.js";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const corpus = path.join(root, "tests/_fixtures/differential/formatter/sfc-mixed-raw-line-endings");

test("native formatter forwards layout endings while preserving raw body bytes", () => {
  const source = fs.readFileSync(path.join(corpus, "App.vue.txt"), "utf8");
  const expected = fs.readFileSync(path.join(corpus, "reference.expected.txt"), "utf8");
  const options: FormatOptionsNapi = Object.freeze({ endOfLine: "auto" });
  const first = formatSfc(source, options);
  assert.deepEqual(first, { code: expected, changed: true });
  assert.ok(first.code.includes("<pre>first\n  second</pre>"));
  for (let pass = 2; pass <= 3; pass += 1) {
    assert.deepEqual(formatSfc(first.code, options), { code: expected, changed: false });
  }
  assert.deepEqual(options, { endOfLine: "auto" });
  assert.equal(fs.readFileSync(path.join(corpus, "App.vue.txt"), "utf8"), source);
});

test("native formatter accepts each configured terminator and explicit values take precedence", () => {
  const authored = "<template>\r\n<p>value</p>\n</template>\n";
  const lf = "<template>\n  <p>value</p>\n</template>\n";
  const cases = [
    ["lf", lf],
    ["crlf", lf.replaceAll("\n", "\r\n")],
    ["cr", lf.replaceAll("\n", "\r")],
    ["auto", lf.replaceAll("\n", "\r\n")],
  ] as const;
  for (const vueVersion of ["2", "2.7", "3"]) {
    for (const [endOfLine, expected] of cases) {
      const options = Object.freeze({ endOfLine, vueVersion });
      assert.deepEqual(formatSfc(authored, options), { code: expected, changed: true });
      for (let pass = 2; pass <= 3; pass += 1) {
        assert.deepEqual(formatSfc(expected, options), { code: expected, changed: false });
      }
    }
  }
  assert.deepEqual(formatSfc(authored), { code: lf, changed: true });
  for (const newline of ["\n", "\r\n", "\r"]) {
    const source = `<template>${newline}<p>value</p>\n</template>\n`;
    const expected = lf.replaceAll("\n", newline);
    assert.deepEqual(formatSfc(source, { endOfLine: "auto" }), { code: expected, changed: true });
    assert.deepEqual(formatSfc(expected, { endOfLine: "auto" }), {
      code: expected,
      changed: false,
    });
  }
  assert.deepEqual(formatSfc("<template><p>value</p></template>", { endOfLine: "auto" }), {
    code: lf,
    changed: true,
  });
});

test("native formatter rejects unsupported line endings at the public boundary", () => {
  for (const endOfLine of ["", "windows", "CRLF", "\n", 1, true, {}, []]) {
    assert.throws(
      () => formatSfc("<template><p>value</p></template>", { endOfLine } as FormatOptionsNapi),
      (error: unknown) => {
        assert.ok(error instanceof Error);
        assert.equal(
          (error as Error & { code: string }).code,
          typeof endOfLine === "string" ? "InvalidArg" : "StringExpected",
        );
        assert.match(error.message, /endOfLine|EndOfLineNapi/);
        return true;
      },
    );
  }
});

test("native attributeGroups preserves complete output and fixed points across Vue versions", (t) => {
  const cases = [
    {
      id: "exact-groups",
      body: '<div title="t" class="c" data-role="main"></div>',
      groups: [["data-role"], ["class"], ["*"]],
      expected: '<div data-role="main" class="c" title="t"></div>',
    },
    {
      id: "prefix-and-catch-all",
      body: '<div class="c" data-z="z" title="t" data-a="a"></div>',
      groups: [["title"], ["data-*"], ["*"]],
      expected: '<div title="t" data-a="a" data-z="z" class="c"></div>',
    },
    {
      id: "empty-group-and-unmatched",
      body: '<div class="c" data-z="z" title="t" data-a="a"></div>',
      groups: [[], ["data-*"]],
      expected: '<div data-a="a" data-z="z" class="c" title="t"></div>',
    },
    {
      id: "empty-groups",
      body: '<div id="i" title="t" class="c"></div>',
      groups: [],
      expected: '<div class="c" id="i" title="t"></div>',
    },
    {
      id: "dynamic-barrier",
      body: '<div title="t" class="c" :data-value="value" data-z="z" data-a="a"></div>',
      groups: [["data-*"], ["title"], ["*"]],
      expected: '<div title="t" class="c" :data-value="value" data-a="a" data-z="z"></div>',
    },
    {
      id: "duplicate-static-names",
      body: '<div title="first" class="c" title="last"></div>',
      groups: [["class"], ["*"]],
      expected: '<div title="first" class="c" title="last"></div>',
    },
  ];
  const observations = [];
  for (const vueVersion of ["2", "2.7", "3"]) {
    for (const item of cases) {
      const source = `<template>${item.body}</template>`;
      const expected = `<template>\n  ${item.expected}\n</template>\n`;
      const groups = structuredClone(item.groups);
      for (const group of groups) Object.freeze(group);
      Object.freeze(groups);
      const options = Object.freeze({ vueVersion, attributeGroups: groups });
      const before = JSON.stringify(options);
      const first = formatSfc(source, options);
      assert.deepEqual(first, { code: expected, changed: true }, `${vueVersion}/${item.id}`);
      for (let pass = 2; pass <= 3; pass += 1) {
        assert.deepEqual(formatSfc(first.code, options), { code: expected, changed: false });
      }
      assert.equal(JSON.stringify(options), before);
      observations.push({ id: item.id, vueVersion, options, result: first });
    }
  }
  t.diagnostic(JSON.stringify({ schema: "glyph-native-attribute-groups-v1", observations }));
});

test("native attributeGroups applies after every preset and respects explicit sorting controls", (t) => {
  const body = '<div title="t" class="c" data-role="main"></div>';
  const source = `<template>${body}</template>`;
  const custom = '<div data-role="main" class="c" title="t"></div>';
  const defaultBody = '<div class="c" data-role="main" title="t"></div>';
  assert.deepEqual(formatSfc(source), {
    code: `<template>\n  ${defaultBody}\n</template>\n`,
    changed: true,
  });
  const observations = [];
  for (const preset of ["vize", "prettier", "compact", "documentation"] as const) {
    for (const sortAttributes of [true, false]) {
      const options = {
        preset,
        sortAttributes,
        singleAttributePerLine: false,
        attributeGroups: [["data-role"], ["class"], ["*"]],
      };
      const expected = `<template>\n  ${sortAttributes ? custom : body}\n</template>\n`;
      const before = structuredClone(options);
      const first = formatSfc(source, options);
      assert.deepEqual(first, { code: expected, changed: true }, `${preset}/${sortAttributes}`);
      for (let pass = 2; pass <= 3; pass += 1) {
        assert.deepEqual(formatSfc(first.code, options), { code: expected, changed: false });
      }
      assert.deepEqual(options, before);
      observations.push({ preset, sortAttributes, result: first });
    }
  }
  const options: FormatOptionsNapi = {
    attributeGroups: [["data-*"], ["*"]],
    attributeSortOrder: "as-written",
  };
  const authored = '<div class="c" data-z="z" title="t" data-a="a"></div>';
  const expected =
    '<template>\n  <div data-z="z" data-a="a" class="c" title="t"></div>\n</template>\n';
  assert.deepEqual(formatSfc(`<template>${authored}</template>`, options), {
    code: expected,
    changed: true,
  });
  assert.deepEqual(formatSfc(expected, options), { code: expected, changed: false });
  t.diagnostic(
    JSON.stringify({ schema: "glyph-native-attribute-groups-presets-v1", observations }),
  );
});

test("native attributeGroups rejects non-nested or non-string patterns", () => {
  for (const attributeGroups of [1, "id", ["id"], [[1]]]) {
    assert.throws(() => Reflect.apply(formatSfc, undefined, ["<template />", { attributeGroups }]));
  }
});
