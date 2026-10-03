import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { compiler, runtime, hash } from "./support/native-sfc-setup-reference.ts";
import { checkOrdinaryMap, executeOrdinary } from "./support/native-sfc-ordinary-reference.ts";

const ts = createRequire(new URL("../package.json", import.meta.url))("typescript");
const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_sfc_ordinary_empty_vue_3_5_35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const capturePath = process.env.VIZE_NATIVE_SFC_ORDINARY_EMPTY_CAPTURE;
const captured = capturePath ? JSON.parse(fs.readFileSync(capturePath, "utf8")) : null;
const executions: unknown[] = [];

test("original ordinary JS/TS retains complete independent pinned Vue and official TS reference bytes/maps", () => {
  assert.equal(compiler.version, "3.5.35");
  assert.equal(runtime.version, "3.5.35");
  assert.equal(ts.version, "6.0.3");
  assert.equal(pack.schema, "vize.native-sfc.ordinary-empty-dom-reference");
  assert.equal(pack.fixtures.length, 3);
  for (const fixture of pack.fixtures) {
    const parsed = compiler.parse(fixture.source, { filename: "Ordinary雪🌸.vue" });
    assert.deepEqual(parsed.errors, []);
    assert.equal(parsed.descriptor.script.content, fixture.script);
    assert.equal(parsed.descriptor.template.content, fixture.template);
    assert.equal(parsed.descriptor.scriptSetup, null);
    const script = compiler.compileScript(parsed.descriptor, {
      id: fixture.id,
      genDefaultAs: "__sfc_main",
    });
    assert.equal(script.content, fixture.referenceScript);
    assert.deepEqual(script.map, fixture.referenceScriptMap);
    assert.deepEqual(script.bindings, fixture.referenceBindings);
    assert.deepEqual(script.bindings, {});
    const render = compiler.compileTemplate({
      source: fixture.template,
      filename: "Ordinary雪🌸.vue",
      id: fixture.id,
      sourceMap: true,
      compilerOptions: {
        mode: "module",
        hoistStatic: false,
        prefixIdentifiers: true,
        comments: true,
        bindingMetadata: script.bindings,
        cacheHandlers: false,
      },
    });
    assert.deepEqual(render.errors, []);
    assert.equal(render.code, fixture.referenceRender);
    assert.deepEqual(render.map, fixture.referenceRenderMap);
    const original =
      script.content +
      "\n" +
      render.code +
      "\n__sfc_main.render = render;\nexport default __sfc_main;\n";
    const result = ts.transpileModule(original, {
      fileName: "Ordinary雪🌸.vue.reference.ts",
      reportDiagnostics: true,
      compilerOptions: {
        target: ts.ScriptTarget.ESNext,
        module: ts.ModuleKind.ESNext,
        sourceMap: true,
        inlineSources: true,
        newLine: ts.NewLineKind.LineFeed,
      },
    });
    assert.deepEqual(result.diagnostics, []);
    assert.equal(result.outputText, fixture.referenceModule);
    assert.deepEqual(JSON.parse(result.sourceMapText), fixture.referenceModuleMap);
  }
});

test(
  "source-built ordinary capture contains every whole original module and map",
  { skip: !captured },
  () => {
    assert.equal(captured.schema, "vize.native-sfc.ordinary-empty-capture");
    assert.equal(captured.adapter, "vize_atelier_sfc::compile_native_sfc");
    assert.equal(captured.fixtures.length, 3);
    for (const [index, row] of captured.fixtures.entries()) {
      const fixture = pack.fixtures[index];
      assert.equal(row.id, fixture.id);
      assert.equal(row.source, fixture.source);
      assert.deepEqual(row.bindings, []);
      assert.equal(row.code, fixture.code);
      assert.deepEqual(row.nativeMap, fixture.nativeMap);
      assert.equal(typeof row.code, "string");
      assert(row.code.length > 0);
      checkOrdinaryMap({ ...fixture, code: row.code, nativeMap: row.nativeMap });
    }
  },
);

for (const fixture of pack.fixtures) {
  test(`${fixture.id} mounts and updates the actual whole pinned reference component`, async () => {
    const result = await executeOrdinary(fixture.referenceModule);
    assert.deepEqual(result.vnodes, fixture.runtime);
    assert.equal(result.defaultSetupAbsent, true);
  });
  test(
    `${fixture.id} source-built ordinary module mounts without invented setup state`,
    { skip: !captured },
    async () => {
      const row = captured.fixtures.find((row: any) => row.id === fixture.id);
      assert(row);
      assert.equal(row.source, fixture.source);
      const native = await executeOrdinary(row.code),
        reference = await executeOrdinary(fixture.referenceModule);
      assert.deepEqual(native.vnodes, fixture.runtime);
      assert.deepEqual(native.mounted, reference.mounted);
      assert.equal(native.nativeRenderInvocations, 2);
      assert.equal(native.defaultSetupAbsent, true);
      executions.push({
        id: fixture.id,
        sourceSha256: hash(row.source),
        codeSha256: hash(row.code),
        ...native,
      });
      if (process.env.VIZE_NATIVE_SFC_ORDINARY_EMPTY_RUNTIME_CAPTURE)
        fs.writeFileSync(
          process.env.VIZE_NATIVE_SFC_ORDINARY_EMPTY_RUNTIME_CAPTURE,
          JSON.stringify(
            {
              schema: "vize.native-sfc.ordinary-empty-runtime",
              runtime: "vue@3.5.35",
              executions,
            },
            null,
            2,
          ) + "\n",
        );
    },
  );
}
