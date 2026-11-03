import assert from "node:assert/strict";
import { test } from "node:test";
import { parseSync } from "oxc-parser";

import { bindDefaultExport } from "../../shared/native-shape.ts";
import { analyzeModuleOutput } from "./utils/module-output.ts";

const gaps = [
  " ",
  "\r\n",
  "/* note */",
  "/**//**/",
  "/* default decoy */ \r\n// export decoy\r\n",
  "\uFEFF",
  "\u2003/* 😀 */\uFEFF",
  "// note\u2028",
  "// note\u2029",
];

function moduleText(source: string) {
  return {
    code: source,
    replace(start: number, end: number, text: string) {
      this.code = this.code.slice(0, start) + text + this.code.slice(end);
    },
  };
}

void test("native export geometry accepts parsed comments and UTF-8 byte positions", () => {
  for (const prefix of ["", "const label = '雪😀';\r\n"]) {
    for (const gap of gaps) {
      const code = `${prefix}export${gap}default/* tail */ { label: '雪' };`;
      const module = moduleText(code);
      bindDefaultExport(
        module,
        {
          defaultExportStart: Buffer.byteLength(prefix),
          defaultExportKeywordEnd: Buffer.byteLength(`${prefix}export${gap}default`),
        },
        "_sfc_main",
      );
      assert.equal(module.code, `${prefix}const _sfc_main =/* tail */ { label: '雪' };`);
      assert.deepEqual(parseSync("output.mjs", module.code).errors, []);
    }
  }
});

void test("fallback module analysis preserves the same comment-aware keyword geometry", () => {
  for (const prefix of ["", "const label = '雪😀';\r\n"]) {
    for (const gap of gaps) {
      const code = `${prefix}export${gap}default/* tail */ { label: '雪' };`;
      const shape = analyzeModuleOutput(code);
      assert.equal(shape.defaultExportStart, Buffer.byteLength(prefix));
      assert.equal(
        shape.defaultExportKeywordEnd,
        Buffer.byteLength(`${prefix}export${gap}default`),
      );
      const module = moduleText(code);
      bindDefaultExport(module, shape, "_sfc_main");
      assert.equal(module.code, `${prefix}const _sfc_main =/* tail */ { label: '雪' };`);
    }
  }
});

void test("invalid trivia and stale offsets still fail instead of binding a component", () => {
  for (const gap of ["", " + ", "/* unterminated ", "// no newline ", "/*ok*/ unexpected "]) {
    const code = `export${gap}default {};`;
    assert.throws(() =>
      bindDefaultExport(
        moduleText(code),
        {
          defaultExportStart: 0,
          defaultExportKeywordEnd: Buffer.byteLength(`export${gap}default`),
        },
        "_sfc_main",
      ),
    );
  }
  const prefix = "const label = '雪😀';\r\n";
  const code = `${prefix}export/* note */default {};`;
  for (const start of [prefix.length, Buffer.byteLength(prefix) + 1]) {
    assert.throws(() =>
      bindDefaultExport(
        moduleText(code),
        {
          defaultExportStart: start,
          defaultExportKeywordEnd: Buffer.byteLength(`${prefix}export/* note */default`),
        },
        "_sfc_main",
      ),
    );
  }
});
