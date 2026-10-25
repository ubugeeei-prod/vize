const assert = require("node:assert/strict");
const { test } = require("node:test");
const { createMappedLanguageService } = require("../../npm/typescript-plugin/language-service.cjs");
const helper = require("../../npm/native/language-projection.js");
const { LanguageFeature: F } = helper;
const ts = require("../node_modules/typescript");

test("feature dispatch maps Unicode authored spans and keeps edits within exact verbatim text", () => {
  const file = "/src/日本語.vue";
  const source = "😀 <script>const count = 1; count;</script>";
  const prefix = "declare const helper: number;\n";
  const authoredStart = source.indexOf("const count");
  const code = "const count = 1; count;";
  const generated = prefix + code + "\nhelper;";
  const projection = {
    text: generated,
    extension: ".tsx",
    diagnostics: [],
    mappings: [
      [prefix.length, code.length, authoredStart, code.length, 0, 0xfffff],
      [prefix.length + code.length + 1, 6, authoredStart + 6, 5, 1, F.Hover],
    ],
  };
  const document = { source, projection };
  const queried = [];
  const raw = {
    getQuickInfoAtPosition(name, offset) {
      queried.push([name, offset]);
      return { textSpan: { start: offset, length: 5 }, displayParts: [{ text: "count: number" }] };
    },
    findRenameLocations() {
      return [
        { fileName: file, textSpan: { start: prefix.length + 6, length: 5 } },
        { fileName: file, textSpan: { start: prefix.length + code.length + 1, length: 6 } },
        { fileName: "/src/usage.ts", textSpan: { start: 2, length: 5 } },
      ];
    },
    getCompletionEntryDetails() {
      return {
        name: "x",
        codeActions: [
          {
            description: "unsafe mixed edit",
            changes: [
              {
                fileName: file,
                textChanges: [
                  { span: { start: prefix.length + 6, length: 5 }, newText: "total" },
                  { span: { start: 0, length: 0 }, newText: "import x" },
                ],
              },
            ],
          },
        ],
      };
    },
    getFormattingEditsForDocument() {
      throw new Error("Must not call generated formatter");
    },
  };
  const service = createMappedLanguageService(ts, raw, {
    engine: helper,
    get: (name) => (name === file ? document : undefined),
  });
  const info = service.getQuickInfoAtPosition(file, authoredStart + 6);
  assert.deepEqual(info.textSpan, { start: authoredStart + 6, length: 5 });
  assert.deepEqual(queried[0], [file, prefix.length + 6]);
  const edits = service.findRenameLocations(file, authoredStart + 6);
  assert.deepEqual(
    edits.map((value) => [value.fileName, value.textSpan]),
    [
      [file, { start: authoredStart + 6, length: 5 }],
      ["/src/usage.ts", { start: 2, length: 5 }],
    ],
  );
  assert.deepEqual(service.getCompletionEntryDetails(file, authoredStart + 6).codeActions, []);
  assert.deepEqual(service.getFormattingEditsForDocument(file), []);
});

test("projection failures remain diagnostics and generated errors do not disappear", () => {
  const file = "/src/Bad.vue";
  const source = '<script>const value: number = "wrong";</script>';
  let document = { source, error: "native projection failed" };
  const raw = {
    getSyntacticDiagnostics: () => [],
    getSemanticDiagnostics: () => [
      { start: 0, length: 3, code: 2307, category: 1, messageText: "Missing generated helper" },
    ],
  };
  const service = createMappedLanguageService(ts, raw, {
    engine: helper,
    get: (name) => (name === file ? document : undefined),
  });
  assert.match(service.getSemanticDiagnostics(file)[0].messageText, /native projection failed/);
  document = { source, projection: { text: "generated", mappings: [], diagnostics: [] } };
  const diagnostics = service.getSemanticDiagnostics(file);
  assert.equal(diagnostics.length, 1);
  assert.equal(diagnostics[0].code, 2307);
  assert.equal(diagnostics[0].file.text, source);
  assert.match(diagnostics[0].messageText, /generated Vize code/);
});
