import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { loadFrozenTemplateCompiler } from "./support/patina-template-provider.ts";
import { compileFunction } from "node:vm";

const root = path.resolve(import.meta.dirname, "../..");
const fixture = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-component-declaration-bindings.json",
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

function runtimePacket(compiler, typescript, vueRuntime, source, name, requireRuntime) {
  const erase = (source) => erasePacket(typescript, source);
  const compiled = compilerPacket(compiler, source);
  const rewritten = compiler.rewriteDefault(compiled.script.content, "__component", ["typescript"]);
  const script = erase(`${rewritten}\nmodule.exports = __component;`);
  const template = erase(compiled.template.code);
  const run = (code) => {
    const module = { exports: {} };
    compileFunction(code, ["require", "module", "exports"], { filename: "Component.js" })(
      requireRuntime,
      module,
      module.exports,
    );
    return module.exports;
  };
  const component = run(script.outputText);
  component.render = run(template.outputText).render;
  const warnings = [],
    errors = [];
  let instanceObservation = null,
    childObservation = null;
  const Child = {
    created() {
      const props = vueRuntime.getCurrentInstance().vnode.props ?? {};
      childObservation = {
        keys: Object.keys(props),
        onProbeType: typeof props.onProbe,
        probeResult: typeof props.onProbe === "function" ? props.onProbe() : null,
      };
    },
    render: () => null,
  };
  component.components = { ...component.components, Child };
  component.created = function () {
    const instance = vueRuntime.getCurrentInstance();
    instanceObservation = {
      publicValueType: typeof this[name],
      setupValueType: typeof instance.setupState[name],
      setupKeys: Object.keys(instance.setupState),
      scriptSetupFlag: instance.setupState.__isScriptSetup === true,
    };
  };
  const renderer = vueRuntime.createRenderer({
    createComment: (text) => ({ text }),
    insert: (node, parent) => {
      parent.child = node;
    },
    remove: () => {},
    parentNode: () => null,
    nextSibling: () => null,
  });
  const app = renderer.createApp(component);
  app.config.warnHandler = (message, _instance, trace) => warnings.push({ message, trace });
  app.config.errorHandler = (error, _instance, info) =>
    errors.push({ name: error.name, message: error.message, info });
  let mounted = false,
    unmounted = false;
  try {
    app.mount({});
    mounted = true;
    app.unmount();
    unmounted = true;
  } catch (error) {
    errors.push({ name: error.name, message: error.message, info: "mount/unmount" });
  }
  return {
    compiler: compiled,
    scriptJavaScript: script,
    renderJavaScript: template,
    instrumentation:
      "Actual compiled script and unchanged generated render run in pinned Vue custom renderer. Added created hooks observe public proxy/setupState and actual Child vnode props; no source or bindingMetadata rewrite.",
    observation: {
      mounted,
      unmounted,
      instance: instanceObservation,
      child: childObservation,
      warnings,
      errors,
    },
  };
}

await test("declaration bindings retain complete pinned provider, compiler, erasure and actual Vue runtime observations", async () => {
  assert.equal(fixture.schema, 1);
  assert.equal(fixture.records.length, 40);
  assert.equal(new Set(fixture.records.map((record) => record.id)).size, 40);
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
    "template-component-declaration-bindings.json",
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
  for (const expected of fixture.records) {
    await test(expected.id, () => {
      assert.equal(hash(expected.source), expected.sourceSha256);
      assert.deepEqual(
        linter.verify(expected.source, config, { filename: "Component.vue" }),
        expected.referenceMessages,
        "whole unfiltered reference packet",
      );
      const actual = compilerPacket(compiler, expected.source);
      assert.deepEqual(actual, expected.compiler, "whole compiler observation packet");
      assert.deepEqual(actual.parseErrors, []);
      assert.equal(actual.scriptError, null);
      assert.deepEqual(actual.template.errors, []);
      assert.equal(
        expected.productScope,
        expected.ownerExpectation === "bare-setup-owned-template-this-unqualified"
          ? "unqualified-template-this-frontier"
          : expected.surface === "$listeners"
            ? "primary-listeners"
            : "unrepaired-scopedSlots-research",
      );
      if (expected.originalScriptErasure) {
        const originalScript = compiler.parse(expected.source, { filename: "Component.vue" })
          .descriptor.scriptSetup.content;
        assert.deepEqual(
          erasePacket(typescript, originalScript),
          expected.originalScriptErasure,
          "entire original script erasure packet",
        );
      }
      if (
        expected.ownerExpectation === "setup-owned" ||
        expected.ownerExpectation === "setup-owned-template-script-member-unchanged" ||
        expected.ownerExpectation === "bare-setup-owned-template-this-unqualified"
      ) {
        assert.match(actual.script.bindings[expected.surface], /^setup-/);
        assert.ok(actual.template.code.includes(`$setup.${expected.surface}`));
        if (expected.ownerExpectation === "setup-owned") {
          assert.equal(expected.referenceMessages.length, 1, "preserve the reference blind spot");
        }
      } else if (
        expected.ownerExpectation === "component-context" ||
        expected.ownerExpectation === "type-only-erasure"
      ) {
        assert.equal(actual.script.bindings[expected.surface], undefined);
        assert.ok(actual.template.code.includes(`_ctx.${expected.surface}`));
      }
      // Template this and scopedSlots remain research, independent of these
      // complete provider/compiler observations. No acceptance is inferred.
    });
  }
  assert.equal(fixture.runtime.length, 8);
  assert.equal(new Set(fixture.runtime.map((record) => record.id)).size, 8);
  for (const expected of fixture.runtime) {
    await test(expected.id, () => {
      assert.equal(hash(expected.source), expected.sourceSha256);
      const actual = JSON.parse(
        JSON.stringify(
          runtimePacket(
            compiler,
            typescript,
            vueRuntime,
            expected.source,
            expected.surface,
            requireVuePackage,
          ),
        ),
      );
      const { id, surface, form, access, qualification, source, sourceSha256, ...packet } =
        expected;
      assert.deepEqual(actual, packet, "whole compiler/transpile/runtime observation packet");
      assert.equal(actual.observation.mounted, true);
      assert.equal(actual.observation.unmounted, true);
      assert.deepEqual(actual.observation.errors, []);
      assert.equal(actual.observation.instance.publicValueType, "undefined");
      assert.equal(actual.observation.instance.setupValueType, "object");
      if (access === "bare") {
        assert.equal(qualification, "positive-setup-value-runtime");
        assert.equal(actual.observation.child.onProbeType, "function");
        assert.equal(actual.observation.child.probeResult, "local");
      } else {
        assert.equal(qualification, "unqualified-template-this-runtime-contrast");
        assert.equal(actual.observation.child.onProbeType, "undefined");
        assert.equal(actual.observation.child.probeResult, null);
      }
      assert.ok(["$listeners", "$scopedSlots"].includes(surface));
      assert.ok(["script-setup", "explicit-setup"].includes(form));
      assert.equal(hash(source), sourceSha256);
      assert.equal(typeof id, "string");
    });
  }
});
