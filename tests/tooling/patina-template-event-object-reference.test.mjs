import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { compileFunction } from "node:vm";

const root = path.resolve(import.meta.dirname, "../..");
const fixture = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-event-object-reference.json",
    ),
    "utf8",
  ),
);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const packageManifest = path.join(root, "tools/benchmarks/scripts/package.json");
const requireProvider = createRequire(
  process.env.VIZE_LEGACY_API_PROVIDER_MANIFEST ?? packageManifest,
);
const requireParser = createRequire(
  process.env.VIZE_LEGACY_API_SCRIPT_PARSER_MANIFEST ?? packageManifest,
);
const requireVue = createRequire(process.env.VIZE_LEGACY_API_VUE_MANIFEST ?? packageManifest);

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

function runtimePacket(compiler, typescript, vueRuntime, source) {
  const compiled = compilerPacket(compiler, source);
  const erase = (source) => erasePacket(typescript, source);
  const js = erase(compiled.template.code);
  const module = { exports: {} };
  compileFunction(js.outputText, ["require", "module", "exports"], { filename: "Component.js" })(
    requireVue,
    module,
    module.exports,
  );

  const warnings = [],
    errors = [],
    operations = [];
  let phase = "mount",
    button = null;
  const renderer = vueRuntime.createRenderer({
    createElement: (tag) => {
      const n = { tag, props: {}, children: [] };
      if (tag === "button") button = n;
      operations.push({ operation: "createElement", tag });
      return n;
    },
    createText: (text) => ({ text }),
    createComment: (text) => ({ comment: text }),
    insert: (node, parent) => {
      parent.children.push(node);
      operations.push({ operation: "insert", tag: node.tag ?? null });
    },
    remove: (node) => operations.push({ operation: "remove", tag: node.tag ?? null }),
    parentNode: () => null,
    nextSibling: () => null,
    setElementText: (node, text) => {
      node.text = text;
      operations.push({ operation: "setElementText", tag: node.tag, text });
    },
    setText: (node, text) => {
      node.text = text;
    },
    patchProp: (node, key, previous, next) => {
      node.props[key] = next;
      operations.push({
        operation: "patchProp",
        tag: node.tag,
        key,
        previousType: typeof previous,
        nextType: typeof next,
        functionSource: typeof next === "function" ? String(next) : null,
        value: typeof next === "function" ? null : next,
      });
    },
  });
  const component = { name: "ExistingEventObjectControl", render: module.exports.render };
  const app = renderer.createApp(component);
  app.config.warnHandler = (message, _instance, trace) => warnings.push({ phase, message, trace });
  app.config.errorHandler = (error, _instance, info) =>
    errors.push({ phase, name: error.name, message: error.message, info });
  let mounted = false,
    invoked = false,
    unmounted = false,
    eventReturn = null;
  const event = { type: "click", marker: "actual-existing-control" };
  try {
    app.mount({ children: [] });
    mounted = true;
    phase = "event";
    if (typeof button?.props.onClick !== "function")
      throw new Error("Actual onClick handler not installed");
    const value = button.props.onClick(event);
    invoked = true;
    eventReturn = { type: typeof value, value: value ?? null };
    phase = "unmount";
    app.unmount();
    unmounted = true;
  } catch (error) {
    errors.push({ phase, name: error.name, message: error.message, info: "mount/event/unmount" });
  }
  return {
    renderJavaScript: js,
    runtime: {
      instrumentation:
        "Unchanged actual pinned compiler render runs in actual pinned Vue custom renderer. Actual installed button onClick is invoked once with recorded event, then app is unmounted. No source/context/binding metadata rewrite. Handler function source and complete warnings/errors/host operations are retained.",
      event,
      mounted,
      invoked,
      unmounted,
      eventReturn,
      warnings,
      errors,
      operations,
    },
  };
}

await test("the existing event object expression preserves whole independent proof and approved native correction", () => {
  const proof = fixture.proof;
  assert.equal(fixture.schema, 1);
  assert.equal(proof.id, "template-event-object-expression");
  const scriptParser = requireParser("@typescript-eslint/parser");
  for (const [name, key] of [
    ["eslint", "eslint"],
    ["eslint-plugin-vue", "plugin"],
    ["vue-eslint-parser", "parser"],
  ]) {
    assert.equal(requireProvider(`${name}/package.json`).version, proof.provider[key]);
  }
  assert.equal(scriptParser.version, proof.provider.scriptParser);
  const plugin = requireProvider("eslint-plugin-vue");
  const directory = path.dirname(requireProvider.resolve("eslint-plugin-vue/package.json"));
  for (const source of [
    ...Object.values(proof.provider.ruleSources),
    proof.provider.utilitySource,
  ]) {
    assert.equal(hash(fs.readFileSync(path.join(directory, source.path))), source.sha256);
  }
  assert.equal(requireVue("vue/package.json").version, proof.compilerProvider.vue);
  assert.equal(
    requireVue("@vue/compiler-sfc/package.json").version,
    proof.compilerProvider.compiler,
  );
  for (const source of proof.compilerProvider.sources) {
    const directory = path.dirname(requireVue.resolve(`${source.packageName}/package.json`));
    assert.equal(hash(fs.readFileSync(path.join(directory, source.path))), source.sha256);
  }
  const compiler = requireVue("@vue/compiler-sfc");
  const typescript = requireParser("typescript");
  assert.equal(typescript.version, proof.typescriptProvider.version);
  assert.equal(
    hash(fs.readFileSync(requireParser.resolve("typescript"))),
    proof.typescriptProvider.source.sha256,
  );
  const vueRuntime = requireVue("vue");
  assert.equal(vueRuntime.version, proof.runtimeProvider.vue);
  const requireDom = createRequire(requireVue.resolve("@vue/runtime-dom/package.json"));
  const runtimeDirectory = path.dirname(requireDom.resolve("@vue/runtime-core/package.json"));
  assert.equal(
    hash(fs.readFileSync(path.join(runtimeDirectory, proof.runtimeProvider.source.path))),
    proof.runtimeProvider.source.sha256,
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
  assert.equal(fixture.frozenInputs.length, 3);
  const originals = [];
  for (const [index, input] of fixture.frozenInputs.entries()) {
    const bytes = fs.readFileSync(path.join(fixtureDirectory, input.path));
    assert.equal(bytes.length, input.bytes);
    assert.equal(hash(bytes), input.sha256);
    const packet = JSON.parse(bytes);
    const original = (packet.records ?? packet.cases).find((r) => r.id === proof.id);
    assert.deepEqual(
      original,
      proof.inputs[index].originalRecord,
      "the full historical zero packet is immutable",
    );
    assert.deepEqual(input, {
      path: proof.inputs[index].path,
      bytes: proof.inputs[index].bytes,
      sha256: proof.inputs[index].sha256,
    });
    originals.push(original);
  }
  assert.equal(hash(proof.source), proof.sourceSha256);
  assert.equal(proof.source, originals[0].source);
  assert.equal(proof.sourceSha256, originals[0].sourceSha256);
  assert.deepEqual(
    linter.verify(proof.source, config, { filename: "Component.vue" }),
    proof.referenceMessages,
    "whole unfiltered provider packet",
  );
  assert.deepEqual(proof.referenceMessages, [], "preserve the actual provider blind spot");
  assert.deepEqual(originals[1].messages, []);
  assert.deepEqual(originals[2].expected, [
    { file: "Component.vue", messages: [], errorCount: 0, warningCount: 0 },
  ]);
  const compiled = compilerPacket(compiler, proof.source);
  assert.deepEqual(
    compiled,
    proof.compiler,
    "whole compiler packet including generated code/errors/maps",
  );
  assert.deepEqual(compiled.template.errors, []);
  assert.ok(compiled.template.code.includes("$event => ({ ..._ctx.$listeners })"));
  const actual = JSON.parse(
    JSON.stringify(runtimePacket(compiler, typescript, vueRuntime, proof.source)),
  );
  assert.deepEqual(
    actual,
    { renderJavaScript: proof.renderJavaScript, runtime: proof.runtime },
    "whole actual runtime event invocation and host packet",
  );
  assert.equal(actual.runtime.mounted, true);
  assert.equal(actual.runtime.invoked, true);
  assert.equal(actual.runtime.unmounted, true);
  assert.deepEqual(actual.runtime.eventReturn, { type: "object", value: {} });
  assert.deepEqual(actual.runtime.errors, []);
  assert.deepEqual(actual.runtime.warnings, []);
  assert.equal(fixture.originalCorrections.length, 1);
  const correction = fixture.originalCorrections[0];
  assert.equal(correction.id, proof.id);
  assert.equal(correction.sourceSha256, proof.sourceSha256);
  assert.equal(
    correction.qualification,
    "product-accuracy-vue3-event-expression-provider-blindspot",
  );
  assert.equal(correction.expectedStatus, 1);
  assert.equal(correction.nativeMessages.length, 1);
  const native = correction.nativeMessages[0];
  assert.equal(native.start, 31);
  assert.equal(native.end, 41);
  assert.equal(proof.source.slice(native.start, native.end), "$listeners");
  assert.deepEqual(correction.cliExpected, [
    {
      file: "Component.vue",
      messages: [
        {
          ruleId: native.ruleId,
          ruleDocsPath: "docs/content/rules/type-and-script.md",
          severity: 2,
          message: `[vize:${native.ruleId}] ${native.message}`,
          line: 1,
          column: 32,
          endLine: 1,
          endColumn: 42,
          help: native.help,
        },
      ],
      errorCount: 1,
      warningCount: 0,
    },
  ]);
});
