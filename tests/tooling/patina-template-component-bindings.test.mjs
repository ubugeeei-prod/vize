import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import {
  assertLegacyInstanceCompilerSources,
  legacyInstanceVueRequire,
} from "./support/patina-legacy-instance-provider.ts";

const root = path.resolve(import.meta.dirname, "../..");
const fixture = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-component-bindings.json",
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
const requireVue = legacyInstanceVueRequire();

await test("the pinned compiler refuses benchmark Vue and altered source receipts", () => {
  const sources = fixture.compilerProvider.sources;
  assert.throws(
    () =>
      assertLegacyInstanceCompilerSources(
        createRequire(packageManifest),
        "@vue/compiler-sfc",
        sources,
      ),
    /the pinned compiler must own its dependencies/,
  );
  for (const packageName of ["@vue/compiler-sfc", "@vue/compiler-core"]) {
    const altered = sources.map((source) =>
      source.packageName === packageName ? { ...source, sha256: "0".repeat(64) } : source,
    );
    assert.throws(
      () => assertLegacyInstanceCompilerSources(requireVue, "@vue/compiler-sfc", altered),
      { code: "ERR_ASSERTION" },
    );
  }
});

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

await test("component-owned reserved spellings retain complete pinned compiler and reference observations", async () => {
  assert.equal(fixture.schema, 1);
  assert.equal(fixture.records.length, 30);
  assert.equal(new Set(fixture.records.map((record) => record.id)).size, 30);
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
  assert.equal(requireVue("vue/package.json").version, fixture.compilerProvider.vue);
  assert.equal(
    requireVue("@vue/compiler-sfc/package.json").version,
    fixture.compilerProvider.compiler,
  );
  assertLegacyInstanceCompilerSources(
    requireVue,
    "@vue/compiler-sfc",
    fixture.compilerProvider.sources,
  );
  const compiler = requireVue("@vue/compiler-sfc");
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
        expected.surface === "$listeners" ? "primary-listeners" : "unrepaired-scopedSlots-research",
      );
      if (expected.ownerExpectation === "setup-owned") {
        assert.match(actual.script.bindings[expected.surface], /^setup-/);
        assert.ok(actual.template.code.includes(`$setup.${expected.surface}`));
        assert.equal(expected.referenceMessages.length, 1, "preserve the reference blind spot");
      } else if (expected.ownerExpectation === "component-context") {
        assert.equal(actual.script.bindings[expected.surface], undefined);
        assert.ok(actual.template.code.includes(`_ctx.${expected.surface}`));
      }
      // Alias/factory research packets deliberately carry no suppression or
      // product-acceptance claim merely because compiler metadata is absent.
    });
  }
});
