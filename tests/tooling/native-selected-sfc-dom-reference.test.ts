import assert from "node:assert/strict";
import fs from "node:fs";
import { after, test } from "node:test";
import {
  checkNativeMap,
  compiler,
  executeComponent,
  fromVue,
  hash,
  runtimeModuleSource,
} from "./support/native-selected-sfc-dom-runtime.ts";

const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_selected_sfc_click_vue_3_5_35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const capturePath = process.env.VIZE_NATIVE_SELECTED_SFC_DOM_CAPTURE;
const captured = capturePath ? JSON.parse(fs.readFileSync(capturePath, "utf8")) : null;
const requireCapture = process.env.VIZE_NATIVE_SELECTED_SFC_DOM_REQUIRE_CAPTURE === "1";
const runtimeCapturePath = process.env.VIZE_NATIVE_SELECTED_SFC_DOM_RUNTIME_CAPTURE;
const executions: unknown[] = [];
const ids = [
  "event-initializer",
  "original-block-comment",
  "unicode-entity-window",
  "nested-element",
  "fragment-event",
];

test("five complete original SFCs retain fresh pinned Descriptor, render, module and map references", () => {
  assert.equal(pack.schema, "vize.native-sfc.selected-click-reference");
  assert.equal(pack.version, 1);
  assert.deepEqual(pack.compiler, { name: "@vue/compiler-sfc", version: "3.5.35" });
  assert.deepEqual(pack.runtime, { name: "vue", version: "3.5.35" });
  assert.equal(compiler.version, "3.5.35");
  assert.equal(fromVue("vue/package.json").version, "3.5.35");
  assert.equal(pack.filename, "SelectedClick雪🌸.vue");
  assert.deepEqual(
    pack.fixtures.map((fixture: any) => fixture.id),
    ids,
  );
  for (const fixture of pack.fixtures) {
    assert.equal(
      fixture.source,
      `<!--SFC雪🌸:${fixture.id}-->\r\n<template>${fixture.template}</template>`,
    );
    const parsed = compiler.parse(fixture.source, { filename: pack.filename });
    assert.deepEqual(parsed.errors, []);
    const descriptor = parsed.descriptor;
    assert.equal(descriptor.source, fixture.source);
    assert.equal(descriptor.filename, pack.filename);
    assert.equal(descriptor.script, null);
    assert.equal(descriptor.scriptSetup, null);
    assert.deepEqual(descriptor.styles, []);
    assert.deepEqual(descriptor.customBlocks, []);
    assert.equal(descriptor.template.content, fixture.template);
    assert.equal(
      fixture.source.slice(
        descriptor.template.loc.start.offset,
        descriptor.template.loc.end.offset,
      ),
      fixture.template,
    );
    const measured = compiler.compileTemplate({
      source: descriptor.template.content,
      filename: descriptor.filename,
      id: fixture.id,
      sourceMap: true,
      compilerOptions: {
        mode: "module",
        hoistStatic: false,
        prefixIdentifiers: true,
        comments: true,
        bindingMetadata: {},
        cacheHandlers: false,
      },
    });
    assert.deepEqual(measured.errors, []);
    assert.equal(measured.code, fixture.referenceRender, fixture.id);
    assert.deepEqual(measured.map, fixture.referenceMap, fixture.id);
    assert.equal(
      fixture.referenceModule,
      measured.code +
        "\nconst reference_component = { render };\nexport default reference_component;\n",
    );
    assert.equal(fixture.events.length, 2);
    assert.equal(fixture.expected.length, 2);
  }
});

test("independent pinned import-shaped authored strings/comments survive the runtime loader byte-exactly", async () => {
  const body = `const unused="from 'vue'"; /* from 'vue' and from "vue" */ $event.count++;`;
  const source =
    '<!--loader雪🌸-->\r\n<template><button @click="' +
    body.replaceAll('"', "&quot;") +
    '"/></template>';
  const parsed = compiler.parse(source, { filename: pack.filename });
  assert.deepEqual(parsed.errors, []);
  assert.equal(parsed.descriptor.source, source);
  const measured = compiler.compileTemplate({
    source: parsed.descriptor.template.content,
    filename: pack.filename,
    id: "import-shaped-body-control",
    sourceMap: true,
    compilerOptions: {
      mode: "module",
      hoistStatic: false,
      prefixIdentifiers: true,
      comments: true,
      bindingMetadata: {},
      cacheHandlers: false,
    },
  });
  assert.deepEqual(measured.errors, []);
  assert.deepEqual(measured.map.sourcesContent, [parsed.descriptor.template.content]);
  assert(measured.code.includes(body));
  const module =
    measured.code +
    "\nconst reference_component = { render };\nexport default reference_component;\n";
  const rewritten = runtimeModuleSource(module);
  assert.notEqual(rewritten, module);
  assert.equal(rewritten.slice(rewritten.indexOf("\n\n")), module.slice(module.indexOf("\n\n")));
  assert(rewritten.includes(body));
  const result = await executeComponent(module, {
    events: [{ count: 0 }, { count: 7 }],
    expected: [{ count: 1 }, { count: 8 }],
  });
  assert.equal(result.replacedCallbacks, 1);
  assert.equal(result.unmounted, true);
});

test("complete frozen native packets retain exact source/code/maps and hosted capture is mandatory", () => {
  // Global tooling can re-execute frozen bytes, while the composite action
  // requires fresh Rust source capture and unconditional full fixture equality.
  // Missing capture is never represented as a skipped acceptance test.
  if (requireCapture) assert(captured, "hosted acceptance requires actual Rust capture");
  for (const fixture of pack.fixtures) {
    assert(fixture.nativeMap, `${fixture.id}: actual Rust map has not been frozen`);
    checkNativeMap(fixture, pack.filename, fixture.expectedCode, fixture.nativeMap);
  }
  if (captured) {
    assert.equal(captured.schema, "vize.native-sfc.selected-click-capture");
    assert.equal(captured.adapter, "vize_atelier_sfc::compile_native_selected_sfc_dom");
    assert.deepEqual(
      captured.fixtures,
      pack.fixtures.map((fixture: any) => ({
        id: fixture.id,
        source: fixture.source,
        code: fixture.expectedCode,
        nativeMap: fixture.nativeMap,
      })),
    );
  }
});

for (const fixture of pack.fixtures) {
  test(`${fixture.id}: real Vue component mount/update/click/unmount preserves all event effects`, async () => {
    if (requireCapture)
      assert(captured, "runtime acceptance requires source-built whole components");
    const actual = captured?.fixtures.find((entry: any) => entry.id === fixture.id);
    if (captured) {
      assert(actual);
      assert.equal(actual.source, fixture.source);
      assert.equal(actual.code, fixture.expectedCode);
      assert.deepEqual(actual.nativeMap, fixture.nativeMap);
    }
    const code = actual?.code ?? fixture.expectedCode;
    const nativeMap = actual?.nativeMap ?? fixture.nativeMap;
    checkNativeMap(fixture, pack.filename, code, nativeMap);
    const reference = await executeComponent(fixture.referenceModule, fixture);
    const native = await executeComponent(code, fixture);
    assert.deepEqual(native, reference);
    assert.equal(native.replacedCallbacks, 1);
    executions.push({
      id: fixture.id,
      sourceHash: hash(fixture.source),
      nativeCodeHash: hash(code),
      nativeMapHash: hash(JSON.stringify(nativeMap)),
      referenceCodeHash: hash(fixture.referenceModule),
      origin: captured ? "current-source-rust-capture" : "frozen-rust-fixture",
      native,
      reference,
    });
  });
}

after(() => {
  if (!runtimeCapturePath) return;
  assert.equal(requireCapture, true, "hosted runtime receipts require fresh Rust capture");
  assert(captured);
  assert.equal(executions.length, ids.length);
  fs.writeFileSync(
    runtimeCapturePath,
    JSON.stringify(
      {
        schema: "vize.native-sfc.selected-click-runtime-capture",
        adapter: "vize_atelier_sfc::compile_native_selected_sfc_dom",
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
