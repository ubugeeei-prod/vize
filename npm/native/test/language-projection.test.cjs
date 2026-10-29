const assert = require("node:assert/strict");
const { test } = require("node:test");
const {
  toUtf16LanguageProjection,
  applyDiagnosticDirectives,
  mapLanguageDiagnosticRange,
} = require("../language-projection.js");

void test("diagnostic punctuation gaps follow the native span law without enabling navigation", () => {
  // A generated handler diagnostic encloses synthetic parentheses around a
  // mapped expression, as the actual TS6 service reports.
  const projection = {
    mappings: [
      [2, 3, 4, 3, 1, 0],
      [10, 4, 20, 4, 0, 0xfffff],
    ],
  };
  assert.deepEqual(mapLanguageDiagnosticRange(projection, 9, 6), {
    start: 7,
    length: 17,
    fidelity: "approximate",
  });
  assert.deepEqual(mapLanguageDiagnosticRange(projection, 11, 2), {
    start: 21,
    length: 2,
    fidelity: "exact",
  });
  assert.deepEqual(mapLanguageDiagnosticRange(projection, 3, 1), {
    start: 4,
    length: 3,
    fidelity: "atom",
  });
  assert.deepEqual(mapLanguageDiagnosticRange(projection, 6, 2), {
    start: 7,
    length: 0,
    fidelity: "none",
  });
  assert.deepEqual(mapLanguageDiagnosticRange(projection, 14, 0), {
    start: 24,
    length: 0,
    fidelity: "exact",
  });
  assert.equal(projection.mappings[0][5], 0);
});

void test("converts independent UTF-8 spaces, preserving CRLF and diagnostic-only metadata", () => {
  const projection = toUtf16LanguageProjection(
    "😀\r\né",
    JSON.stringify({
      text: "//日本\r\n😀é",
      extension: ".tsx",
      mappings: [[10, 6, 0, 8, 1, 0]],
      diagnostics: [{ start: 6, length: 2, code: 1, messageText: "authored" }],
      semanticLinks: [
        { sourceStart: 10, sourceLength: 4, targetStart: 14, targetLength: 2, kind: "copy" },
      ],
      diagnosticDirectives: {
        directives: [[6, 2, 10, 16, 1, 0]],
        unusedExpectDirectiveDiagnostics: [{ code: 4, messageText: "unused" }],
      },
    }),
  );
  assert.deepEqual(projection.mappings, [[6, 3, 0, 5, 1, 0]]);
  assert.deepEqual(projection.diagnostics, [
    { start: 4, length: 1, code: 1, messageText: "authored" },
  ]);
  assert.deepEqual(projection.semanticLinks, [
    { sourceStart: 6, sourceLength: 2, targetStart: 8, targetLength: 1, kind: "copy" },
  ]);
  assert.deepEqual(projection.diagnosticDirectives.directives, [[4, 1, 6, 9, 1, 0]]);
  assert.throws(
    () =>
      toUtf16LanguageProjection(
        "😀",
        JSON.stringify({
          text: "a",
          extension: ".tsx",
          mappings: [[0, 1, 1, 1, 0, 0]],
          diagnostics: [],
        }),
      ),
    /UTF-8 boundary/,
  );
});

void test("ASCII projection offsets remain exact at both ends and reject invalid boundaries", () => {
  const raw = (start, length) =>
    JSON.stringify({
      text: "line\r\nend",
      extension: ".tsx",
      mappings: [[start, length, start, length, 0, 0xfffff]],
      diagnostics: [],
    });
  for (let start = 0; start <= 9; start++) {
    for (let length = 0; length <= 9 - start; length++) {
      assert.deepEqual(toUtf16LanguageProjection("line\r\nend", raw(start, length)).mappings, [
        [start, length, start, length, 0, 0xfffff],
      ]);
    }
  }
  for (const offset of [-1, 10, 0.5, Number.MAX_SAFE_INTEGER + 1]) {
    assert.throws(() => toUtf16LanguageProjection("line\r\nend", raw(offset, 0)), /UTF-8 boundary/);
  }
});

void test("aggregates EXPECT across the complete collection and preserves native owner-copy ignores", () => {
  const projection = {
    diagnostics: [],
    diagnosticDirectives: {
      directives: [
        [5, 4, 10, 30, 1, 0],
        [5, 4, 40, 50, 0, 0],
      ],
      unusedExpectDirectiveDiagnostics: [{ code: 4, messageText: "unused" }],
    },
  };
  const syntax = { start: 15, code: 1005, messageText: "expected" };
  const echo = { start: 43, code: 2322, messageText: "copy" };
  const real = { start: 55, code: 2339, messageText: "real" };
  assert.deepEqual(applyDiagnosticDirectives(projection, [syntax, echo, real]), {
    diagnostics: [real],
    sourceDiagnostics: [],
  });
  assert.deepEqual(applyDiagnosticDirectives(projection, [echo, real]), {
    diagnostics: [real],
    sourceDiagnostics: [{ start: 5, length: 4, code: 4, messageText: "unused" }],
  });
});
