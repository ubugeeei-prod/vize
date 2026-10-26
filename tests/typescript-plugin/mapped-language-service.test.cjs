const assert = require("node:assert/strict");
const { test } = require("node:test");
const { createMappedLanguageService } = require("../../npm/typescript-plugin/language-service.cjs");
const helper = require("../../npm/native/language-projection.js");
const { LanguageFeature: F } = helper;
const ts = require("../node_modules/typescript");

void test("feature dispatch maps Unicode authored spans and keeps edits within exact verbatim text", () => {
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
    toLineColumnOffset(name, offset) {
      return ts.getLineAndCharacterOfPosition(
        ts.createSourceFile(name, generated, ts.ScriptTarget.Latest),
        offset,
      );
    },
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
  assert.deepEqual(service.toLineColumnOffset(file, info.textSpan.start), {
    line: 0,
    character: authoredStart + 6,
  });
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

void test("projection failures remain diagnostics and generated errors do not disappear", () => {
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

void test("opt-out preserves the original service and ordinary JSX configuration errors remain visible", () => {
  const init = require("../../npm/typescript-plugin/index.cjs");
  const file = "/src/plain.tsx";
  const text = "<div />;";
  const options = { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext, types: [] };
  const host = {
    getScriptFileNames: () => [file],
    getScriptVersion: () => "1",
    getScriptSnapshot: (name) => (name === file ? ts.ScriptSnapshot.fromString(text) : undefined),
    getCompilationSettings: () => options,
    getCurrentDirectory: () => "/src",
    getDefaultLibFileName: ts.getDefaultLibFilePath,
    fileExists: (name) => name === file,
    readFile: (name) => (name === file ? text : undefined),
  };
  const original = ts.createLanguageService(host);
  const plugin = init({ typescript: ts });
  const keys = Object.keys(host);
  assert.equal(
    plugin.create({
      languageService: original,
      languageServiceHost: host,
      config: { enabled: false },
    }),
    original,
  );
  assert.deepEqual(Object.keys(host), keys);
  const wrapped = plugin.create({
    languageService: original,
    languageServiceHost: host,
    serverHost: ts.sys,
  });
  assert(wrapped.getSemanticDiagnostics(file).some((diagnostic) => diagnostic.code === 17004));
  assert.equal(host.getCompilationSettings().jsx, undefined);
});

void test("syntax and semantic packets preserve their channels while EXPECT sees the complete collection", () => {
  const file = "/src/Broken.vue";
  const source = "<script>broken</script>";
  const syntax = {
    start: 0,
    length: 1,
    code: 1109,
    category: 1,
    messageText: "Expression expected",
  };
  const semantic = { start: 1, length: 1, code: 2322, category: 1, messageText: "Type mismatch" };
  const projection = {
    text: "xy",
    mappings: [[0, 2, 8, 2, 0, 0xfffff]],
    diagnostics: [{ start: 0, length: 8, code: 100001, messageText: "Unclosed script" }],
  };
  const service = createMappedLanguageService(
    ts,
    { getSyntacticDiagnostics: () => [syntax], getSemanticDiagnostics: () => [semantic] },
    { engine: helper, get: () => ({ source, projection }) },
  );
  assert.deepEqual(
    service.getSyntacticDiagnostics(file).map((value) => value.code),
    [1109, 100001],
  );
  assert.deepEqual(
    service.getSemanticDiagnostics(file).map((value) => value.code),
    [2322],
  );
});

void test("suggestion packets never repeat the syntax and semantic checks", () => {
  const file = "/src/Suggest.vue";
  const projection = { text: "xy", mappings: [[0, 2, 8, 2, 0, 0xfffff]], diagnostics: [] };
  const suggestion = { start: 0, length: 2, code: 6133, category: 2, messageText: "unused" };
  const unexpected = () => assert.fail("suggestions must not recompute errors");
  const service = createMappedLanguageService(
    ts,
    {
      getSyntacticDiagnostics: unexpected,
      getSemanticDiagnostics: unexpected,
      getSuggestionDiagnostics: () => [suggestion],
    },
    { engine: helper, get: () => ({ source: "<script>xy</script>", projection }) },
  );
  const [mapped] = service.getSuggestionDiagnostics(file);
  assert.deepEqual([mapped.code, mapped.start, mapped.length], [6133, 8, 2]);
});
