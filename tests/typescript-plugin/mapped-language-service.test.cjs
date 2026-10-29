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

void test("real TypeScript file references return only complete authored Vue spans", () => {
  const file = "/src/Consumer.vue",
    dependency = "/src/dep.ts";
  const script = 'import { value } from "./dep";\nvalue;\n';
  const before = '<script setup lang="ts">\n// 😀 日本語\n';
  const source = before + script + "</script>";
  const prefix = "declare const generatedHelper: unknown;\n".repeat(12);
  const texts = { [file]: prefix + script, [dependency]: "export const value = 1;" };
  const host = {
    getScriptFileNames: () => Object.keys(texts),
    getScriptVersion: () => "1",
    getScriptSnapshot: (name) =>
      texts[name] === undefined ? undefined : ts.ScriptSnapshot.fromString(texts[name]),
    getCompilationSettings: () => ({
      noLib: true,
      allowNonTsExtensions: true,
      module: ts.ModuleKind.ESNext,
    }),
    getCurrentDirectory: () => "/src",
    getDefaultLibFileName: () => "/lib.d.ts",
    getScriptKind: () => ts.ScriptKind.TS,
    readFile: (name) => texts[name],
    fileExists: (name) => Object.hasOwn(texts, name),
    resolveModuleNames: (names) =>
      names.map(() => ({ resolvedFileName: dependency, extension: ts.Extension.Ts })),
  };
  const original = ts.createLanguageService(host);
  const projection = {
    text: texts[file],
    diagnostics: [],
    mappings: [[prefix.length, script.length, before.length, script.length, 0, 0xfffff]],
  };
  const service = createMappedLanguageService(ts, original, {
    engine: helper,
    get: (name) => (name === file ? { source, projection } : undefined),
  });
  const raw = original.getFileReferences(dependency);
  assert.equal(raw.length, 1);
  assert(raw[0].textSpan.start > source.length, "the genuine TypeScript result is generated");
  assert.deepEqual(service.getFileReferences(dependency), [
    {
      ...raw[0],
      textSpan: { start: source.indexOf("./dep"), length: "./dep".length },
      contextSpan: { start: before.length, length: script.split("\n")[0].length },
    },
  ]);
});

void test("unsupported SFC call-hierarchy followups never leak cross-file generated spans", () => {
  const normal = {
    from: { file: "/src/plain.ts", span: { start: 1, length: 2 } },
    fromSpans: [{ start: 3, length: 1 }],
  };
  const unsafe = {
    from: { file: "/src/Component.vue", span: { start: 2000, length: 20 } },
    fromSpans: [{ start: 2010, length: 2 }],
  };
  const normalOutgoing = { to: normal.from, fromSpans: normal.fromSpans };
  const unsafeOutgoing = { to: unsafe.from, fromSpans: unsafe.fromSpans };
  const calls = [];
  const raw = {
    provideCallHierarchyIncomingCalls(...args) {
      calls.push(args);
      return [normal, unsafe];
    },
    provideCallHierarchyOutgoingCalls(...args) {
      calls.push(args);
      return [normalOutgoing, unsafeOutgoing];
    },
  };
  const service = createMappedLanguageService(ts, raw, { engine: helper });
  assert.deepEqual(service.provideCallHierarchyIncomingCalls("/src/Component.vue", 20), []);
  assert.deepEqual(service.provideCallHierarchyOutgoingCalls("/src/Component.art", 20), []);
  assert.deepEqual(calls, [], "unsupported authored positions never reach the generated service");
  assert.deepEqual(service.provideCallHierarchyIncomingCalls("/src/plain.ts", 3), [normal]);
  assert.deepEqual(service.provideCallHierarchyOutgoingCalls("/src/plain.ts", 3), [normalOutgoing]);
  assert.equal(service.provideCallHierarchyIncomingCalls("/src/plain.ts", 3)[0], normal);
  assert.deepEqual(calls, [
    ["/src/plain.ts", 3],
    ["/src/plain.ts", 3],
    ["/src/plain.ts", 3],
  ]);
});

for (const newline of ["\n", "\r\n"]) {
  void test(
    "component navigation uses current Program aliases and lexical ownership " +
      JSON.stringify(newline),
    () => {
      const path = require("node:path");
      const {
        resolveVueImportDefinition,
      } = require("../../npm/typescript-plugin/import-resolution.cjs");
      const main = "/src/main.ts",
        one = "/src/One.vue",
        two = "/src/Two.vue";
      const diskSource = [
        'const compass = "😀 日本語";',
        'import Card from "./One.vue";',
        'import { default as Named } from "./One.vue";',
        'import { Bridge } from "./bridge";',
        "Card; Named; Bridge;",
        "function shadow(Card: number) { return Card; }",
        "const object = { Card: 1 }; object.Card;",
        "const shorthand = { Card };",
      ].join(newline);
      const files = {
        [main]: diskSource,
        [one]: "declare const One: new () => {}; export default One;",
        [two]: "declare const Two: new () => {}; export default Two;",
        "/src/bridge.ts": 'export { default as Bridge } from "./One.vue";',
      };
      const overlays = new Map();
      let version = 0;
      const resolve = (name, containing) => {
        const exact = path.posix.resolve(path.posix.dirname(containing), name);
        const fileName = Object.hasOwn(files, exact + ".ts") ? exact + ".ts" : exact;
        return Object.hasOwn(files, fileName)
          ? { resolvedFileName: fileName, extension: ts.Extension.Ts }
          : undefined;
      };
      const host = {
        getScriptFileNames: () => [main],
        getScriptVersion: () => String(version),
        getProjectVersion: () => String(version),
        getScriptSnapshot: (name) => {
          const text = overlays.get(name) ?? files[name];
          return text === undefined ? undefined : ts.ScriptSnapshot.fromString(text);
        },
        getCompilationSettings: () => ({
          noLib: true,
          allowNonTsExtensions: true,
          module: ts.ModuleKind.ESNext,
        }),
        getCurrentDirectory: () => "/src",
        getDefaultLibFileName: () => "/lib.d.ts",
        getScriptKind: () => ts.ScriptKind.TS,
        readFile: (name) => files[name],
        fileExists: (name) => Object.hasOwn(files, name),
        resolveModuleNames: (names, containing) => names.map((name) => resolve(name, containing)),
      };
      const service = ts.createLanguageService(host);
      let adapterParseCalls = 0;
      const adapterTs = {
        ...ts,
        createSourceFile(...args) {
          adapterParseCalls++;
          return ts.createSourceFile(...args);
        },
      };
      const support = { virtual: { engine: helper, get: () => undefined, resolve } };
      const query = (text, marker, offset = 1) =>
        resolveVueImportDefinition(
          adapterTs,
          support,
          main,
          text.indexOf(marker) + offset,
          service,
        );
      assert.equal(query(diskSource, "Card;").definition.fileName, one);
      assert.equal(query(diskSource, "Named;").definition.fileName, one);
      assert.equal(query(diskSource, "Bridge;").definition.fileName, one);
      const opened = diskSource.replace('Card from "./One.vue"', 'Card from "./Two.vue"');
      overlays.set(main, opened);
      version++;
      assert.equal(host.readFile(main), diskSource, "the real host's readFile remains disk text");
      assert.equal(query(opened, "Card;").definition.fileName, two);
      assert.equal(query(opened, '"./Two.vue"', 2).definition.fileName, two);
      assert.equal(query(opened, "Card: number"), undefined);
      assert.equal(query(opened, "return Card", "return ".length + 1), undefined);
      assert.equal(query(opened, "Card: 1"), undefined);
      assert.equal(query(opened, "object.Card", "object.".length + 1), undefined);
      assert.equal(
        query(opened, "{ Card }", 3).definition.fileName,
        two,
        "actual shorthand value ownership is preserved",
      );
      overlays.set(main, "");
      version++;
      assert.equal(
        resolveVueImportDefinition(adapterTs, support, main, 0, service),
        undefined,
        "an unsaved empty buffer never uses stale disk imports",
      );
      assert.equal(
        adapterParseCalls,
        0,
        "all navigation reuses the genuine Program; no second authored-source parse",
      );
    },
  );
}

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
