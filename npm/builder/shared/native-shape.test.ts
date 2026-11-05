// `bindDefaultExport` takes the native compiler's UTF-8 byte offsets plus the
// text they index, and edits at UTF-16 indices (#11583).
import assert from "node:assert/strict";
import { test } from "node:test";
import { MappedModule } from "./mapped-module.ts";
import { bindDefaultExport } from "./native-shape.ts";

/** The span the native compiler reports for `code`, in UTF-8 bytes. */
function nativeSpan(code: string) {
  const { start, end } = keywordSpan(code);
  return {
    defaultExportStart: Buffer.byteLength(code.slice(0, start)),
    defaultExportKeywordEnd: Buffer.byteLength(code.slice(0, end)),
  };
}

/** The same span as UTF-16 indices: what a caller must not pass. */
function keywordSpan(code: string) {
  const match = /export\s+default/.exec(code)!;
  return { start: match.index, end: match.index + match[0].length };
}

function bind(code: string, shape = nativeSpan(code)) {
  const module = new MappedModule(code, null);
  bindDefaultExport(module, shape, "_sfc_main");
  return module.code;
}

const CASES = {
  ascii: 'const s = "a";\nexport default { name: "A" }',
  cjk: 'const s = "日本";\nexport default { name: "A" }',
  astral: 'const s = "🐱";\r\nexport default { name: "A" }',
  mixed: '// é 🐱 日本\nconst a = "ß", b = `${"🐱"}`;\nexport   default { name: "A" }',
  newline: 'const s = "日本";\nexport\ndefault { name: "A" }',
  combining: 'const s = "e\u0301 cafe\u0301";\nexport default { name: "e\u0301" }',
  after: 'const a = 1;\nexport default { name: "日本 🐱 e\u0301" }',
};

for (const [name, code] of Object.entries(CASES)) {
  void test(`binds the native default export (${name} text)`, () => {
    const keyword = /export\s+default/.exec(code)![0];
    assert.equal(bind(code), code.replace(keyword, "const _sfc_main ="));
  });
}

void test("keeps the source map aligned across the multibyte rewrite", () => {
  const code = 'const s = "🐱";\nexport default { name: "A" }';
  const module = new MappedModule(code, {
    version: 3,
    sources: ["A.vue"],
    names: [],
    mappings: "AAAA;AACA",
  });
  bindDefaultExport(module, nativeSpan(code), "_sfc_main");
  assert.equal(module.code, 'const s = "🐱";\nconst _sfc_main = { name: "A" }');
  assert.ok(module.map);
});

// Negative controls: every span that does not describe this text throws.
void test("rejects UTF-16 indices passed as native byte offsets", () => {
  for (const name of ["cjk", "astral", "mixed", "combining"] as const) {
    const code = CASES[name];
    const { start, end } = keywordSpan(code);
    assert.throws(
      () => bind(code, { defaultExportStart: start, defaultExportKeywordEnd: end }),
      /native component export geometry/i,
      name,
    );
  }
});

void test("ASCII text has identical byte and UTF-16 offsets", () => {
  const { start, end } = keywordSpan(CASES.ascii);
  assert.equal(
    bind(CASES.ascii, { defaultExportStart: start, defaultExportKeywordEnd: end }),
    bind(CASES.ascii),
  );
});

void test("rejects a span made stale by an earlier edit", () => {
  const code = CASES.cjk;
  const module = new MappedModule(code, null);
  module.prepend('import "./style.css";\n');
  assert.throws(
    () => bindDefaultExport(module, nativeSpan(code), "_sfc_main"),
    /native component export geometry/i,
  );
});

void test("rejects missing, negative and mid-character offsets", () => {
  const code = CASES.cjk;
  const { defaultExportStart, defaultExportKeywordEnd } = nativeSpan(code);
  for (const shape of [
    {},
    { defaultExportStart },
    { defaultExportStart: -1, defaultExportKeywordEnd },
    { defaultExportStart: 1.5, defaultExportKeywordEnd },
  ]) {
    assert.throws(() => bind(code, shape), /Missing native component export geometry/);
  }
  // Byte 12 falls inside "日".
  assert.throws(
    () => bind(code, { defaultExportStart: 12, defaultExportKeywordEnd }),
    /Invalid native component export geometry/,
  );
  assert.throws(
    () => bind(code, { defaultExportStart, defaultExportKeywordEnd: Buffer.byteLength(code) + 1 }),
    /Invalid native component export geometry/,
  );
});
