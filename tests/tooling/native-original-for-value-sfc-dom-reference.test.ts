import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { after, test } from "node:test";
import { compiler, runtime, hash } from "./support/native-selected-sfc-dom-runtime.ts";
import { executeSetupComponent } from "./support/native-selected-setup-sfc-dom-runtime.ts";

const ts = createRequire(new URL("../package.json", import.meta.url))("typescript");
const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_original_for_value_sfc_vue_3_5_35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const capturePath = process.env.VIZE_NATIVE_ORIGINAL_FOR_VALUE_SFC_DOM_CAPTURE;
const requireCapture = process.env.VIZE_NATIVE_ORIGINAL_FOR_VALUE_SFC_DOM_REQUIRE_CAPTURE === "1";
const captured = capturePath ? JSON.parse(fs.readFileSync(capturePath, "utf8")) : null;
const executions: unknown[] = [];

test("six original mutable For value components retain pinned Vue whole JS/TS module decisions", () => {
  assert.equal(compiler.version, "3.5.35");
  assert.equal(runtime.version, "3.5.35");
  assert.equal(ts.version, "6.0.3");
  assert.equal(pack.schema, "vize.native-sfc.original-for-value-reference");
  assert.equal(pack.fixtures.length, 6);
  assert.equal(new Set(pack.fixtures.map((f: any) => f.id)).size, 6);
  if (requireCapture)
    assert(captured, "hosted acceptance requires the actual source-built Rust components");
  if (captured) {
    assert.equal(captured.schema, "vize.native-sfc.original-for-value-capture");
    assert.equal(captured.adapter, "vize_atelier_sfc::compile_native_selected_setup_sfc_dom");
    assert.deepEqual(
      captured.fixtures,
      pack.fixtures.map((f: any) => ({
        id: f.id,
        source: f.source,
        code: f.expectedCode,
        nativeMap: f.nativeMap,
        nativeMapRaw: f.nativeMapRaw,
      })),
    );
  }
  for (const fixture of pack.fixtures) {
    const parsed = compiler.parse(fixture.source, { filename: pack.filename });
    assert.deepEqual(parsed.errors, []);
    assert.equal(parsed.descriptor.script, null);
    assert.equal(parsed.descriptor.scriptSetup.content, fixture.script);
    assert.equal(parsed.descriptor.template.content, fixture.template);
    const script = compiler.compileScript(parsed.descriptor, {
      id: fixture.id,
      genDefaultAs: "_sfc_main",
    });
    assert.equal(script.content, fixture.referenceScript);
    assert.deepEqual(JSON.parse(JSON.stringify(script.map)), fixture.referenceScriptMap);
    assert.deepEqual(Object.keys(script.bindings), fixture.bindings);
    const render = compiler.compileTemplate({
      source: fixture.template,
      filename: pack.filename,
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
    assert.deepEqual(render.map, fixture.referenceMap);
    assert.deepEqual(fixture.nativeMap.sourcesContent, [fixture.source]);
    assert.deepEqual(fixture.nativeMap.names, [
      ...new Set([...fixture.bindings, /v-for='([^ ]+) in/.exec(fixture.template)![1]]),
    ]);
    assert.equal(fixture.nativeMap.version, 3);
    assert.equal(fixture.nativeMapRaw, JSON.stringify(fixture.nativeMap));
    assert.deepEqual(JSON.parse(fixture.nativeMapRaw), fixture.nativeMap);
  }
});

test("the added display helper alias preserves its complete original primary and earliest lower refusal", () => {
  assert.equal(pack.refusals.length, 1);
  const fixture = pack.refusals[0];
  assert.equal(fixture.expectedRefusal, "ReservedAlias");
  assert.equal(fixture.template, "<i v-for='_toDisplayString in count'>{{_toDisplayString}}</i>");
  const parsed = compiler.parse(fixture.source, { filename: pack.filename });
  assert.deepEqual(parsed.errors, []);
  const script = compiler.compileScript(parsed.descriptor, {
    id: fixture.id,
    genDefaultAs: "_sfc_main",
  });
  assert.equal(script.content, fixture.referenceScript);
  assert.deepEqual(JSON.parse(JSON.stringify(script.map)), fixture.referenceScriptMap);
  const render = compiler.compileTemplate({
    source: fixture.template,
    filename: pack.filename,
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
  assert.deepEqual(render.map, fixture.referenceMap);
});

for (const fixture of pack.fixtures) {
  test(`${fixture.id}: original callback values initialize, update and unmounts in actual Vue`, async () => {
    if (requireCapture) assert(captured);
    const row = captured?.fixtures.find((r: any) => r.id === fixture.id);
    if (captured) assert(row);
    const code = row?.code ?? fixture.expectedCode;
    assert.equal(code, fixture.expectedCode);
    const original =
      fixture.referenceScript +
      "\n;\n" +
      fixture.referenceRender.replace("export function render", "function render") +
      "\n_sfc_main.render = render\nexport default _sfc_main\n";
    let referenceCode = original;
    if (fixture.source.includes('lang="ts"')) {
      const result = ts.transpileModule(original, {
        fileName: "SelectedSetup.ts",
        reportDiagnostics: true,
        compilerOptions: {
          target: ts.ScriptTarget.ESNext,
          module: ts.ModuleKind.ESNext,
          newLine: ts.NewLineKind.LineFeed,
        },
      });
      assert.deepEqual(result.diagnostics, []);
      referenceCode = result.outputText;
    }
    const reference = await executeSetupComponent(referenceCode, fixture, false);
    const native = await executeSetupComponent(code, fixture, true);
    assert.deepEqual(native, reference);
    for (const [state, values] of [
      [native.initial, fixture.forValueRuntime.initial],
      [native.updated, fixture.forValueRuntime.updated],
    ] as const) {
      assert.equal(state.type, "fragment");
      assert.equal(state.patchFlag, 256);
      assert.deepEqual(
        state.children,
        values.map((value: string) => ({
          type: fixture.forValueRuntime.tag,
          props: {},
          children: value,
          patchFlag: 1,
        })),
      );
    }
    executions.push({
      id: fixture.id,
      sourceHash: hash(fixture.source),
      nativeCodeHash: hash(code),
      nativeMapHash: hash(JSON.stringify(row?.nativeMap ?? fixture.nativeMap)),
      nativeMapRawHash: hash(row?.nativeMapRaw ?? fixture.nativeMapRaw),
      referenceCodeHash: hash(referenceCode),
      origin: captured ? "current-source-rust-capture" : "independent-desired-module",
      native,
      reference,
    });
  });
}

after(() => {
  const path = process.env.VIZE_NATIVE_ORIGINAL_FOR_VALUE_SFC_DOM_RUNTIME_CAPTURE;
  if (!path) return;
  assert(requireCapture && captured);
  assert.equal(executions.length, pack.fixtures.length);
  fs.writeFileSync(
    path,
    JSON.stringify(
      {
        schema: "vize.native-sfc.original-for-value-runtime-capture",
        adapter: "vize_atelier_sfc::compile_native_selected_setup_sfc_dom",
        runtime: pack.runtime,
        host: "vue-createRenderer-custom-host",
        sourceCaptureHash: hash(fs.readFileSync(capturePath!, "utf8")),
        fixtures: executions,
      },
      null,
      2,
    ) + "\n",
  );
});
