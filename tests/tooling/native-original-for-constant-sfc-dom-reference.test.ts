import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { after, test } from "node:test";
import { compiler, runtime, hash } from "./support/native-selected-sfc-dom-runtime.ts";
import {
  executeConstantComponent,
  runtimeErrorDetails,
} from "./support/native-original-for-constant-sfc-dom-runtime.ts";

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
const inherited = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_original_for_constant_inherited_sfc_vue_3_5_35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const mode = process.env.NODE_ENV === "production" ? "production" : "development";
const capturePath = process.env.VIZE_NATIVE_ORIGINAL_FOR_CONSTANT_SFC_DOM_CAPTURE;
const requireCapture =
  process.env.VIZE_NATIVE_ORIGINAL_FOR_CONSTANT_SFC_DOM_REQUIRE_CAPTURE === "1";
const captured =
  capturePath && fs.existsSync(capturePath)
    ? JSON.parse(fs.readFileSync(capturePath, "utf8"))
    : null;
const fixtures = [...pack.fixtures, ...pack.rangeControls, ...inherited.fixtures];
const executions: unknown[] = [];
const attempts: unknown[] = [];

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
    bindings: script.bindings,
    script: script.content,
    scriptMap: JSON.parse(JSON.stringify(script.map)),
    render: render.code,
    renderMap: render.map,
    originalModule: original,
    runtimeModule: code,
  };
}

function assertPrimary(fixture: any, reference: any) {
  assert.deepEqual(Object.keys(reference.bindings), fixture.bindings);
  assert.equal(reference.bindings[fixture.collection], "literal-const");
  let expectedRender = fixture.expectedCode
    .slice(
      fixture.expectedCode.indexOf("function render"),
      fixture.expectedCode.indexOf("_sfc_main.render"),
    )
    .trimEnd();
  // Pinned compiler-core 3.5.35 genVNodeCall emits flag comments only in DEV.
  assert.equal(expectedRender.split(" /* STABLE_FRAGMENT */").length, 2);
  assert.equal(expectedRender.split(" /* TEXT */").length, fixture.dynamic ? 2 : 1);
  if (mode === "production") {
    expectedRender = expectedRender.replace(", 64 /* STABLE_FRAGMENT */))", ", 64))");
    if (fixture.dynamic) expectedRender = expectedRender.replace(", 1 /* TEXT */)", ", 1)");
  }
  assert.equal(
    reference.render
      .slice(reference.render.indexOf("export function render"))
      .replace("export function render", "function render"),
    expectedRender,
  );
}

test("constant whole code/object/raw maps require the current source-built capture", () => {
  assert.equal(compiler.version, "3.5.35");
  assert.equal(runtime.version, "3.5.35");
  assert.equal(ts.version, "6.0.3");
  assert.equal(pack.nativeExecutions, 0, "desired fixture model never executes native code");
  assert.equal(pack.fixtures.length, 10);
  assert.equal(pack.rangeControls.length, 1);
  assert.equal(inherited.nativeExecutions, 0);
  assert.equal(inherited.fixtures.length, 1);
  assert.equal(inherited.rangeControls.length, 0);
  assert.equal(inherited.filename, pack.filename);
  assert.equal(new Set(fixtures.map((f: any) => f.id)).size, 12);
  if (requireCapture) assert(captured, "mandatory actual source-built Rust modules");
  if (captured) {
    assert.equal(captured.schema, "vize.native-sfc.original-for-constant-capture");
    assert.equal(captured.adapter, "vize_atelier_sfc::compile_native_selected_setup_sfc_dom");
    assert.deepEqual(
      captured.fixtures,
      [...pack.fixtures, ...pack.rangeControls].map((f: any) => ({
        id: f.id,
        source: f.source,
        code: f.expectedCode,
        nativeMap: f.nativeMap,
        nativeMapRaw: f.nativeMapRaw,
      })),
    );
    assert.deepEqual(
      captured.inheritedControls,
      inherited.fixtures.map((f: any) => ({
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
    const attempt: any = {
      id: fixture.id,
      source: fixture.source,
      sourceHash: hash(fixture.source),
      qualified: false,
      nativeCapture: null,
      primary: null,
      native: null,
      primaryObservations: [],
      nativeObservations: [],
      primaryFailureEvents: [],
      nativeFailureEvents: [],
      error: null,
    };
    try {
      // Each actual module is attempted once, even if its peer attempt throws.
      // Missing capture never executes the desired native module as a substitute.
      const range = pack.rangeControls.includes(fixture);
      const attemptErrors: { phase: string; error: unknown }[] = [];
      let reference: any, original: any, native: any, row: any;
      try {
        reference = primary(fixture);
        attempt.reference = reference;
        original = await executeConstantComponent(
          reference.runtimeModule,
          fixture,
          false,
          mode,
          range,
          (observation) => attempt.primaryObservations.push(observation),
          attempt.primaryFailureEvents,
        );
        attempt.primary = original;
      } catch (error) {
        attemptErrors.push({ phase: "primary-attempt", error });
      }
      try {
        if (!captured) {
          assert(!requireCapture, "mandatory actual source-built native capture is absent");
        } else {
          row = [...captured.fixtures, ...captured.inheritedControls].find(
            (r: any) => r.id === fixture.id,
          );
          attempt.nativeCapture = row ?? null;
          assert(row, "actual native row is absent");
          assert.equal(row.source, fixture.source);
          assert.equal(typeof row.code, "string");
          native = await executeConstantComponent(
            row.code,
            fixture,
            true,
            mode,
            range,
            (observation) => attempt.nativeObservations.push(observation),
            attempt.nativeFailureEvents,
          );
          attempt.native = native;
        }
      } catch (error) {
        attemptErrors.push({ phase: "native-attempt", error });
      }
      attempt.attemptErrors = attemptErrors.map(({ phase, error }) => ({
        phase,
        error: runtimeErrorDetails(error),
      }));
      if (attemptErrors.length) throw attemptErrors[0].error;
      // Whole expected comparisons follow both authenticated actual attempts.
      assertPrimary(fixture, reference);
      if (!captured) return;
      const code = row.code;
      assert.equal(code, fixture.expectedCode);
      assert.deepEqual(row.nativeMap, fixture.nativeMap);
      assert.equal(row.nativeMapRaw, fixture.nativeMapRaw);
      assert.deepEqual(native, original);
      attempt.qualified = true;
      executions.push({
        id: fixture.id,
        sourceHash: hash(fixture.source),
        nativeCodeHash: hash(code),
        nativeMapHash: hash(JSON.stringify(row.nativeMap)),
        nativeMapRawHash: hash(row.nativeMapRaw),
        referenceModuleHash: hash(reference.originalModule),
        referenceRuntimeHash: hash(reference.runtimeModule),
        origin: "current-source-rust-capture",
        reference,
        native,
        primary: original,
      });
    } catch (error: any) {
      attempt.error = runtimeErrorDetails(error);
      throw error;
    } finally {
      for (const key of ["primaryFailureEvents", "nativeFailureEvents"]) {
        attempt[key] = attempt[key].map((event: any) => ({
          phase: event.phase,
          error: runtimeErrorDetails(event.error),
        }));
      }
      attempts.push(attempt);
    }
  });
}

after(() => {
  const rawPath = process.env.VIZE_NATIVE_ORIGINAL_FOR_CONSTANT_SFC_DOM_RAW_RUNTIME_CAPTURE;
  if (rawPath) {
    fs.writeFileSync(
      rawPath,
      JSON.stringify(
        {
          schema: "vize.native-sfc.original-for-constant-unqualified-runtime-attempts",
          qualified: false,
          mode,
          sourceCaptureHash: captured ? hash(fs.readFileSync(capturePath!, "utf8")) : null,
          attempts,
        },
        null,
        2,
      ) + "\n",
    );
  }
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
