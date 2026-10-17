import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { loadFrozenTemplateCompiler } from "./support/patina-template-provider.ts";
import { compileFunction } from "node:vm";
import { gunzipSync } from "node:zlib";

const root = path.resolve(import.meta.dirname, "../..");
const overlay = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-authority-review-corrections.json",
    ),
    "utf8",
  ),
);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const proofBytes = fs.readFileSync(
  path.join(
    root,
    "tests/_fixtures/differential/lint/legacy-instance-apis-9217",
    overlay.proof.path,
  ),
);
assert.equal(proofBytes.length, overlay.proof.bytes);
assert.equal(hash(proofBytes), overlay.proof.sha256);
const proofJson = gunzipSync(proofBytes);
assert.equal(proofJson.length, overlay.proof.uncompressedBytes);
assert.equal(hash(proofJson), overlay.proof.uncompressedSha256);
const fixture = JSON.parse(proofJson);

const packageManifest = path.join(root, "tools/benchmarks/scripts/package.json");
const requireProvider = createRequire(
  process.env.VIZE_LEGACY_API_PROVIDER_MANIFEST ?? packageManifest,
);
const requireParser = createRequire(
  process.env.VIZE_LEGACY_API_SCRIPT_PARSER_MANIFEST ?? packageManifest,
);

function compilerPacket(compiler, source) {
  const diagnostic = (error) =>
    typeof error === "string"
      ? error
      : {
          name: error.name,
          message: error.message,
          code: error.code ?? null,
          loc: error.loc ?? null,
        };
  const parsed = compiler.parse(source, { filename: "Component.vue" });
  let script = null;
  let scriptError = null;
  try {
    const compiled = compiler.compileScript(parsed.descriptor, {
      id: "component-binding-control",
      inlineTemplate: false,
    });
    script = {
      content: compiled.content,
      bindings: compiled.bindings ?? {},
      imports: compiled.imports ?? {},
      scriptAst: compiled.scriptAst ?? null,
      scriptSetupAst: compiled.scriptSetupAst ?? null,
      map: compiled.map ?? null,
    };
  } catch (error) {
    scriptError = { name: error.name, message: error.message };
  }
  const compiled = compiler.compileTemplate({
    id: "component-binding-control",
    filename: "Component.vue",
    source: parsed.descriptor.template?.content ?? "",
    compilerOptions: {
      mode: "module",
      prefixIdentifiers: true,
      bindingMetadata: script?.bindings ?? {},
    },
  });
  // This is the JSON observation boundary, preserving the entire selected
  // public fields/ASTs/maps without changing names, ranges or diagnostic order.
  return JSON.parse(
    JSON.stringify({
      parseErrors: parsed.errors.map(diagnostic),
      script,
      scriptError,
      template: {
        code: compiled.code,
        source: compiled.source,
        preamble: compiled.preamble ?? "",
        errors: compiled.errors.map(diagnostic),
        tips: compiled.tips,
        map: compiled.map ?? null,
      },
    }),
  );
}

function erasePacket(typescript, source) {
  const result = typescript.transpileModule(source, {
    compilerOptions: {
      target: typescript.ScriptTarget.ES2022,
      module: typescript.ModuleKind.CommonJS,
    },
    reportDiagnostics: true,
  });
  return {
    outputText: result.outputText,
    diagnostics: (result.diagnostics ?? []).map((diagnostic) => ({
      code: diagnostic.code,
      category: diagnostic.category,
      start: diagnostic.start ?? null,
      length: diagnostic.length ?? null,
      messageText: diagnostic.messageText,
    })),
  };
}

function legacyRuntimePacket(compiler, typescript, vueRuntime, source, requireRuntime) {
  const erase = (source) => erasePacket(typescript, source);
  const compiled = compilerPacket(compiler, source);
  const js = erase(compiled.template.code);
  const module = { exports: {} };
  compileFunction(js.outputText, ["require", "module", "exports"], { filename: "Component.js" })(
    requireRuntime,
    module,
    module.exports,
  );
  const operations = [],
    warnings = [],
    errors = [],
    children = [];
  let nextId = 0;
  const parents = new WeakMap();
  const node = (kind, extra = {}) => {
    const n = { id: nextId++, kind, ...extra, children: [] };
    operations.push({ operation: "create", id: n.id, kind, ...extra });
    return n;
  };
  const renderer = vueRuntime.createRenderer({
    createElement: (tag) => node("element", { tag }),
    createText: (text) => node("text", { text }),
    createComment: (text) => node("comment", { text }),
    setText: (n, text) => {
      n.text = text;
      operations.push({ operation: "setText", id: n.id, text });
    },
    setElementText: (n, text) => {
      n.text = text;
      operations.push({ operation: "setElementText", id: n.id, text });
    },
    patchProp: (n, key, previous, next) => {
      (n.props ??= {})[key] = next;
      operations.push({ operation: "patchProp", id: n.id, key, previous: previous ?? null, next });
    },
    insert: (n, parent, anchor = null) => {
      const previous = parents.get(n);
      if (previous) {
        const index = previous.children.indexOf(n);
        if (index >= 0) previous.children.splice(index, 1);
      }
      const index = anchor ? parent.children.indexOf(anchor) : -1;
      parent.children.splice(index < 0 ? parent.children.length : index, 0, n);
      parents.set(n, parent);
      operations.push({
        operation: "insert",
        id: n.id,
        parent: parent.id,
        anchor: anchor?.id ?? null,
      });
    },
    remove: (n) => {
      const parent = parents.get(n);
      if (parent) {
        const index = parent.children.indexOf(n);
        if (index >= 0) parent.children.splice(index, 1);
      }
      parents.delete(n);
      operations.push({ operation: "remove", id: n.id });
    },
    parentNode: (n) => parents.get(n) ?? null,
    nextSibling: (n) => {
      const parent = parents.get(n);
      return parent?.children[parent.children.indexOf(n) + 1] ?? null;
    },
  });
  const Child = {
    created() {
      const instance = vueRuntime.getCurrentInstance();
      children.push({
        props: instance.vnode.props ?? null,
        slotNames: Object.keys(instance.slots),
      });
    },
    render() {
      return this.$slots.default?.();
    },
  };
  const component = { name: "LegacyControl", components: { Child }, render: module.exports.render };
  const app = renderer.createApp(component);
  app.config.warnHandler = (message, _instance, trace) => warnings.push({ message, trace });
  app.config.errorHandler = (error, _instance, info) =>
    errors.push({ name: error.name, message: error.message, info });
  const snapshot = (n) => ({
    id: n.id,
    kind: n.kind,
    tag: n.tag,
    text: n.text,
    props: n.props,
    children: n.children.map(snapshot),
  });
  const container = node("root");
  let mounted = false,
    unmounted = false,
    mountedTree = null;
  try {
    app.mount(container);
    mounted = true;
    mountedTree = snapshot(container);
    app.unmount();
    unmounted = true;
  } catch (error) {
    errors.push({ name: error.name, message: error.message, info: "mount/unmount" });
  }
  return {
    renderJavaScript: js,
    instrumentation:
      "Unchanged actual compiler-dom generated render runs in pinned Vue custom renderer. Child renders its actual default slot. Complete host operations, resulting mounted host-owned tree fields (id/kind/tag/text/props/children), actual vnode plain attributes, Vue warnings and errors are preserved. No compat mode, source rewrite or seeded context binding.",
    observation: { mounted, unmounted, children, mountedTree, operations, warnings, errors },
  };
}

await test("authority review corrections retain whole frozen receipts and actual default Vue compiler/runtime evidence", async () => {
  assert.equal(fixture.schema, 1);
  assert.equal(fixture.records.length, 10);
  assert.equal(overlay.guardRecords.length, 4);
  assert.deepEqual(overlay.legacyCorrections, fixture.legacyCorrections);
  assert.deepEqual(overlay.frozenInput, fixture.frozenInput);
  assert.deepEqual(overlay.frozenCliInput, fixture.frozenCliInput);
  assert.equal(new Set(fixture.records.map((record) => record.id)).size, 10);
  const scriptParser = requireParser("@typescript-eslint/parser");
  for (const [name, key] of [
    ["eslint", "eslint"],
    ["eslint-plugin-vue", "plugin"],
    ["vue-eslint-parser", "parser"],
  ]) {
    assert.equal(requireProvider(`${name}/package.json`).version, fixture.provider[key]);
  }
  assert.equal(scriptParser.version, fixture.provider.scriptParser);
  const plugin = requireProvider("eslint-plugin-vue");
  const directory = path.dirname(requireProvider.resolve("eslint-plugin-vue/package.json"));
  for (const source of [
    ...Object.values(fixture.provider.ruleSources),
    fixture.provider.utilitySource,
  ]) {
    assert.equal(hash(fs.readFileSync(path.join(directory, source.path))), source.sha256);
  }
  const { compiler, vueRuntime, typescript, requireVuePackage } = loadFrozenTemplateCompiler(
    overlay.proof.path,
    fixture.compilerProvider,
    "sfc",
  );
  assert.equal(typescript.version, fixture.typescriptProvider.version);
  assert.equal(
    hash(fs.readFileSync(requireVuePackage.resolve("typescript"))),
    fixture.typescriptProvider.source.sha256,
  );
  assert.equal(vueRuntime.version, fixture.runtimeProvider.vue);
  const requireDom = createRequire(requireVuePackage.resolve("@vue/runtime-dom/package.json"));
  const runtimeDirectory = path.dirname(requireDom.resolve("@vue/runtime-core/package.json"));
  assert.equal(
    hash(fs.readFileSync(path.join(runtimeDirectory, fixture.runtimeProvider.source.path))),
    fixture.runtimeProvider.source.sha256,
  );
  const { Linter } = requireProvider("eslint");
  const linter = new Linter();
  const config = {
    files: ["**/*.vue"],
    languageOptions: {
      parser: requireProvider("vue-eslint-parser"),
      parserOptions: {
        parser: { js: scriptParser, ts: scriptParser, tsx: scriptParser },
        ecmaVersion: "latest",
        sourceType: "module",
        ecmaFeatures: { jsx: true },
        extraFileExtensions: [".vue"],
      },
    },
    linterOptions: { reportUnusedDisableDirectives: "off" },
    plugins: { vue: plugin },
    rules: {
      "vue/no-deprecated-dollar-listeners-api": "error",
      "vue/no-deprecated-dollar-scopedslots-api": "error",
    },
  };

  const fixtureDirectory = path.join(
    root,
    "tests/_fixtures/differential/lint/legacy-instance-apis-9217",
  );
  const originals = [];
  for (const input of [fixture.frozenInput, fixture.frozenCliInput]) {
    const bytes = fs.readFileSync(path.join(fixtureDirectory, input.path));
    assert.equal(bytes.length, input.bytes);
    assert.equal(hash(bytes), input.sha256);
    originals.push(JSON.parse(bytes));
  }
  assert.equal(fixture.legacyCorrections.length, 6);
  assert.deepEqual(
    fixture.legacyCorrections.map((r) => r.id),
    [
      "legacy-slot-scope",
      "legacy-template-scope",
      "legacy-entity-scope",
      "legacy-default-reference",
      "failed-legacy-slot",
      "legacy-unit-refusal",
    ],
  );
  for (const expected of fixture.records) {
    await test(expected.id, () => {
      assert.equal(hash(expected.source), expected.sourceSha256);
      assert.deepEqual(
        linter.verify(expected.source, config, { filename: "Component.vue" }),
        expected.referenceMessages,
        "whole unfiltered provider packet",
      );
      const actual = compilerPacket(compiler, expected.source);
      assert.deepEqual(
        actual,
        expected.compiler,
        "whole compiler AST/bindingMetadata/render/map packet",
      );
      assert.deepEqual(actual.parseErrors, []);
      assert.deepEqual(actual.template.errors, []);
      if (expected.kind === "frozen-legacy-correction") {
        const original = originals[0].records.find((r) => r.id === expected.id);
        assert.deepEqual(
          expected.frozenOriginalRecord,
          original,
          "the complete immutable original row is retained",
        );
        assert.equal(expected.source, original.source);
        assert.equal(expected.sourceSha256, original.sourceSha256);
        const runtime = JSON.parse(
          JSON.stringify(
            legacyRuntimePacket(
              compiler,
              typescript,
              vueRuntime,
              expected.source,
              requireVuePackage,
            ),
          ),
        );
        assert.deepEqual(
          runtime,
          expected.runtime,
          "whole unchanged generated render/runtime/host packet",
        );
        assert.equal(runtime.observation.mounted, true);
        assert.equal(runtime.observation.unmounted, true);
        assert.deepEqual(runtime.observation.errors, []);
        const listenersWarnings = runtime.observation.warnings.filter(
          (w) =>
            w.message ===
            'Property "$listeners" was accessed during render but is not defined on instance.',
        );
        // This count is an additional focused assertion. The entire original
        // warnings packet above remains unfiltered and must match exactly.
        assert.equal(listenersWarnings.length, expected.nativeCorrection.messages.length);
        if (expected.id === "legacy-default-reference") {
          assert.ok(!actual.template.code.includes("_ctx.$listeners"));
          assert.equal(
            runtime.observation.children[0].props["slot-scope"],
            "{ handlers = $listeners }",
          );
        } else {
          assert.ok(actual.template.code.includes("_ctx.$listeners"));
        }
        const correction = fixture.legacyCorrections.find((r) => r.id === expected.id);
        assert.equal(correction.source, original.source);
        assert.equal(correction.sourceSha256, original.sourceSha256);
        assert.deepEqual(correction.nativeMessages, expected.nativeCorrection.messages);
        assert.equal(correction.nativeGap, false);
        assert.equal(correction.expectedStatus, correction.nativeMessages.length ? 1 : 0);
        const historical = originals[1].cases.find((r) => r.id === "legacy-slot-scope");
        const prototype = historical.expected[0].messages[0];
        const messages = correction.nativeMessages.map((m) => ({
          ...prototype,
          line: 1,
          column: m.start + 1,
          endLine: 1,
          endColumn: m.end + 1,
        }));
        assert.deepEqual(correction.cliExpected, [
          { file: "Component.vue", messages, errorCount: messages.length, warningCount: 0 },
        ]);
        for (const message of correction.nativeMessages) {
          assert.equal(expected.source.slice(message.start, message.end), "$listeners");
        }
      } else {
        assert.equal(expected.kind, "setup-binding-cardinality-correction");
        assert.equal(expected.declarationCount, 4097);
        assert.equal(actual.scriptError, null);
        assert.match(actual.script.bindings.$listeners, /^setup-/);
        assert.ok(actual.template.code.includes("$setup.$listeners"));
        assert.deepEqual(expected.nativeMessages, []);
        assert.deepEqual(expected.nativeCorrection.messages, []);
        assert.equal(expected.expectedStatus, 0);
        const guard = overlay.guardRecords.find((record) => record.id === expected.id);
        assert.deepEqual(
          guard,
          {
            id: expected.id,
            source: expected.source,
            sourceSha256: expected.sourceSha256,
            nativeMessages: [],
            expectedStatus: 0,
          },
          "minimal native guard joins the exact full proof source and hash",
        );
      }
    });
  }
});
