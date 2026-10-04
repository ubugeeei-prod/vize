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
