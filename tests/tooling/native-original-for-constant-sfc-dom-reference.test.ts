import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { after, test } from "node:test";
import { compiler, runtime, hash } from "./support/native-selected-sfc-dom-runtime.ts";
import { executeConstantComponent } from "./support/native-original-for-constant-sfc-dom-runtime.ts";

const ts = createRequire(new URL("../package.json", import.meta.url))("typescript");
const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_original_for_constant_sfc_vue_3_5_35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const mode = process.env.NODE_ENV === "production" ? "production" : "development";
const capturePath = process.env.VIZE_NATIVE_ORIGINAL_FOR_CONSTANT_SFC_DOM_CAPTURE;
const requireCapture =
  process.env.VIZE_NATIVE_ORIGINAL_FOR_CONSTANT_SFC_DOM_REQUIRE_CAPTURE === "1";
const captured = capturePath ? JSON.parse(fs.readFileSync(capturePath, "utf8")) : null;
const fixtures = [...pack.fixtures, ...pack.rangeControls];
const executions: unknown[] = [];

function primary(fixture: any) {
  const parsed = compiler.parse(fixture.source, { filename: pack.filename });
  assert.deepEqual(parsed.errors, []);
  assert.equal(parsed.descriptor.script, null);
  assert.equal(parsed.descriptor.scriptSetup.content, fixture.script);
  assert.equal(parsed.descriptor.template.content, fixture.template);
  const script = compiler.compileScript(parsed.descriptor, {
    id: fixture.id,
    genDefaultAs: "_sfc_main",
  });
  assert.deepEqual(Object.keys(script.bindings), fixture.bindings);
  assert.equal(script.bindings[fixture.collection], "literal-const");
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
  const expectedRender = fixture.expectedCode
    .slice(
      fixture.expectedCode.indexOf("function render"),
      fixture.expectedCode.indexOf("_sfc_main.render"),
    )
    .trimEnd();
  assert.equal(
    render.code
      .slice(render.code.indexOf("export function render"))
      .replace("export function render", "function render"),
    expectedRender,
  );
  assert.deepEqual(render.map.sourcesContent, [fixture.template]);
  const original =
    script.content +
    "\n;\n" +
    render.code.replace("export function render", "function render") +
    "\n_sfc_main.render = render\nexport default _sfc_main\n";
  let code = original;
  if (fixture.source.includes('lang="ts"')) {
    const transformed = ts.transpileModule(original, {
      fileName: pack.filename + ".ts",
      reportDiagnostics: true,
      compilerOptions: {
        target: ts.ScriptTarget.ESNext,
        module: ts.ModuleKind.ESNext,
        newLine: ts.NewLineKind.LineFeed,
      },
    });
    assert.deepEqual(transformed.diagnostics, []);
    code = transformed.outputText;
  }
  return {
    script: script.content,
    scriptMap: JSON.parse(JSON.stringify(script.map)),
    render: render.code,
    renderMap: render.map,
    originalModule: original,
    runtimeModule: code,
  };
}

test("constant whole code/object/raw maps require the current source-built capture", () => {
  assert.equal(compiler.version, "3.5.35");
  assert.equal(runtime.version, "3.5.35");
  assert.equal(ts.version, "6.0.3");
  assert.equal(pack.nativeExecutions, 0, "desired fixture model never executes native code");
  assert.equal(pack.fixtures.length, 10);
  assert.equal(pack.rangeControls.length, 1);
  assert.equal(new Set(fixtures.map((f: any) => f.id)).size, 11);
  if (requireCapture) assert(captured, "mandatory actual source-built Rust modules");
  if (captured) {
    assert.equal(captured.schema, "vize.native-sfc.original-for-constant-capture");
    assert.equal(captured.adapter, "vize_atelier_sfc::compile_native_selected_setup_sfc_dom");
    assert.deepEqual(
      captured.fixtures,
      fixtures.map((f: any) => ({
        id: f.id,
        source: f.source,
        code: f.expectedCode,
        nativeMap: f.nativeMap,
        nativeMapRaw: f.nativeMapRaw,
      })),
    );
  }
  for (const fixture of fixtures) {
    assert.equal(fixture.nativeMapRaw, JSON.stringify(fixture.nativeMap));
    assert.deepEqual(JSON.parse(fixture.nativeMapRaw), fixture.nativeMap);
    assert.deepEqual(fixture.nativeMap.sourcesContent, [fixture.source]);
    assert.deepEqual(fixture.nativeMap.names, [...new Set([fixture.collection, fixture.alias])]);
  }
});

for (const fixture of fixtures) {
  test(`${fixture.id}: ${mode} original constant list cold mount, force-update and unmount`, async () => {
    if (requireCapture) assert(captured);
    const row = captured?.fixtures.find((r: any) => r.id === fixture.id);
    if (captured) assert(row);
    const code = row?.code ?? fixture.expectedCode;
    assert.equal(code, fixture.expectedCode);
    const reference = primary(fixture);
    const range = pack.rangeControls.includes(fixture);
    const native = await executeConstantComponent(code, fixture, true, mode, range);
    const original = await executeConstantComponent(
      reference.runtimeModule,
      fixture,
      false,
      mode,
      range,
    );
    assert.deepEqual(native, original);
    executions.push({
      id: fixture.id,
      sourceHash: hash(fixture.source),
      nativeCodeHash: hash(code),
      nativeMapHash: hash(JSON.stringify(row?.nativeMap ?? fixture.nativeMap)),
      nativeMapRawHash: hash(row?.nativeMapRaw ?? fixture.nativeMapRaw),
      referenceModuleHash: hash(reference.originalModule),
      referenceRuntimeHash: hash(reference.runtimeModule),
      origin: captured ? "current-source-rust-capture" : "independent-desired-module",
      reference,
      native,
      primary: original,
    });
  });
}

after(() => {
  const path = process.env.VIZE_NATIVE_ORIGINAL_FOR_CONSTANT_SFC_DOM_RUNTIME_CAPTURE;
  if (!path) return;
  assert(requireCapture && captured);
  assert.equal(executions.length, fixtures.length);
  fs.writeFileSync(
    path,
    JSON.stringify(
      {
        schema: "vize.native-sfc.original-for-constant-runtime-capture",
        adapter: "vize_atelier_sfc::compile_native_selected_setup_sfc_dom",
        runtime: pack.runtime,
        mode,
        host: "vue-createRenderer-custom-host",
        sourceCaptureHash: hash(fs.readFileSync(capturePath!, "utf8")),
        fixtures: executions,
      },
      null,
      2,
    ) + "\n",
  );
});
