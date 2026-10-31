import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { after, test } from "node:test";
import { compiler, runtime, hash } from "./support/native-selected-sfc-dom-runtime.ts";
import {
  assertRootIfFixture,
  executeRootIfComponent,
  rootIfErrorDetails,
  type RootIfFixture,
} from "./support/native-original-root-if-sfc-dom-runtime.ts";

const ts = createRequire(new URL("../package.json", import.meta.url))("typescript");
const inputRaw = fs.readFileSync(
  new URL(
    "../../crates/vize_atelier_sfc/tests/fixtures/native_original_root_if_inputs_vue_3_5_35.json",
    import.meta.url,
  ),
  "utf8",
);
const pack = JSON.parse(inputRaw);
const modes = ["development", "production"] as const;
const mode = process.env.NODE_ENV ?? "development";
assert(mode === "development" || mode === "production");
assert.equal(pack.schema, "vize.native-sfc.original-root-if-inputs");
assert.equal(pack.fixtures.length, 9);
const fixtures: RootIfFixture[] = pack.fixtures;
for (const fixture of fixtures) assertRootIfFixture(fixture);
assert.equal(new Set(fixtures.map((fixture) => fixture.id)).size, 9);
const capturePath = process.env.VIZE_NATIVE_ORIGINAL_ROOT_IF_SFC_DOM_CAPTURE;
const requireCapture = process.env.VIZE_NATIVE_ORIGINAL_ROOT_IF_SFC_DOM_REQUIRE_CAPTURE === "1";
const captureRaw = capturePath ? fs.readFileSync(capturePath, "utf8") : null;
const captured = captureRaw ? JSON.parse(captureRaw) : null;
const primaryPrefix = process.env.VIZE_NATIVE_ORIGINAL_ROOT_IF_SFC_DOM_PRIMARY_CAPTURE;
const runtimePrefix = process.env.VIZE_NATIVE_ORIGINAL_ROOT_IF_SFC_DOM_RUNTIME_CAPTURE;
type NativeRow = {
  id: string;
  mode: string;
  source: string;
  code: string;
  nativeMap: Record<string, unknown>;
  nativeMapRaw: string;
};
const nativeRows = new Map<string, NativeRow>();
const primaryRows = new Map<string, ReturnType<typeof compilePrimary>>();
const primaryAttempts: unknown[] = [];
const runtimeAttempts: Record<string, unknown>[] = [];

function compilePrimary(fixture: RootIfFixture) {
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
  assert.equal(render.code.split("export function render").length, 2);
  const originalModule =
    script.content +
    "\n;\n" +
    render.code.replace("export function render", "function render") +
    "\n_sfc_main.render = render\nexport default _sfc_main\n";
  let runtimeModule = originalModule;
  let transpilation: unknown = null;
  if (parsed.descriptor.scriptSetup.lang === "ts") {
    const result = ts.transpileModule(originalModule, {
      fileName: "OriginalRootIf.ts",
      reportDiagnostics: true,
      compilerOptions: {
        target: ts.ScriptTarget.ESNext,
        module: ts.ModuleKind.ESNext,
        newLine: ts.NewLineKind.LineFeed,
      },
    });
    assert.deepEqual(result.diagnostics, []);
    assert.deepEqual(Reflect.ownKeys(result), ["outputText", "diagnostics", "sourceMapText"]);
    const { sourceMapText, ...capturedResult } = result;
    assert.equal(sourceMapText, undefined);
    runtimeModule = result.outputText;
    transpilation = { version: ts.version, ...capturedResult };
  }
  const scriptMapRaw = JSON.stringify(script.map);
  const renderMapRaw = JSON.stringify(render.map);
  assert.equal(typeof scriptMapRaw, "string");
  assert.equal(typeof renderMapRaw, "string");
  return {
    id: fixture.id,
    mode,
    source: fixture.source,
    sourceHash: hash(fixture.source),
    parseErrors: parsed.errors,
    script: script.content,
    scriptMap: JSON.parse(scriptMapRaw),
    scriptMapRaw,
    bindings: script.bindings,
    render: render.code,
    renderMap: JSON.parse(renderMapRaw),
    renderMapRaw,
    renderErrors: render.errors,
    renderTips: render.tips,
    originalModule,
    runtimeModule,
    transpilation,
  };
}

test("nine root If inputs pin providers and eighteen actual mode-specific Rust rows", () => {
  assert.equal(compiler.version, "3.5.35");
  assert.equal(runtime.version, "3.5.35");
  assert.equal(ts.version, "6.0.3");
  assert.deepEqual(pack.compiler, { version: compiler.version });
  assert.deepEqual(pack.runtime, { version: runtime.version });
  assert.deepEqual(pack.typescript, { version: ts.version });
  assert.equal(
    pack.qualification,
    "input-only; no native code, source-map or primary capture credit",
  );
  for (const fixture of fixtures) {
    for (const field of [
      "expectedCode",
      "nativeMap",
      "nativeMapRaw",
      "referenceScript",
      "referenceRender",
    ]) {
      assert.equal(
        Object.hasOwn(fixture, field),
        false,
        "native or primary goldens require genuine capture",
      );
    }
  }
  if (requireCapture) {
    assert(captured, "hosted acceptance requires genuine source-built Rust output");
    assert(primaryPrefix && runtimePrefix, "both complete evidence prefixes are required");
    assert.equal(captured.sourceRevision, process.env.GITHUB_SHA);
  }
  if (!captured) return;
  assert.equal(captured.schema, "vize.native-sfc.original-root-if-capture");
  assert.equal(
    captured.adapter,
    "vize_atelier_sfc::compile_native_selected_setup_sfc_dom_with_mode",
  );
  assert.equal(captured.qualification, "observed-unqualified-current-oracles-pending");
  assert.equal(captured.nativeGoldenComparisonPerformed, false);
  const expected = modes.flatMap((current) =>
    fixtures.map((fixture) => ({ id: fixture.id, mode: current })),
  );
  for (const field of ["fixtures", "custody", "attempts"]) {
    assert(Array.isArray(captured[field]));
    assert.deepEqual(
      captured[field].map((row: { id: string; mode: string }) => ({ id: row.id, mode: row.mode })),
      expected,
      `whole ordered mode-major ${field}`,
    );
  }
  for (const [index, row] of captured.fixtures.entries()) {
    const fixture = fixtures[index % fixtures.length];
    assert.equal(row.source, fixture.source);
    assert.equal(typeof row.code, "string");
    assert(row.code.length > 0);
    assert(row.nativeMap && typeof row.nativeMap === "object" && !Array.isArray(row.nativeMap));
    assert.equal(typeof row.nativeMapRaw, "string");
    assert.deepEqual(JSON.parse(row.nativeMapRaw), row.nativeMap);
    assert.equal(row.nativeMap.version, 3);
    assert.equal(row.nativeMap.file, pack.filename);
    assert.deepEqual(row.nativeMap.sources, [pack.filename]);
    assert.deepEqual(row.nativeMap.sourcesContent, [fixture.source]);
    assert(Array.isArray(row.nativeMap.names));
    assert(row.nativeMap.names.every((name: unknown) => typeof name === "string"));
    assert.equal(new Set(row.nativeMap.names).size, row.nativeMap.names.length);
    assert.equal(typeof row.nativeMap.mappings, "string");
    assert(row.nativeMap.mappings.length > 0);
    if (row.mode === mode) nativeRows.set(row.id, row);
  }
  assert.deepEqual(
    [...nativeRows.keys()],
    fixtures.map((fixture) => fixture.id),
  );
});

for (const fixture of fixtures) {
  test(`${fixture.id}: preserve complete official primary output for ${mode}`, () => {
    try {
      const primary = compilePrimary(fixture);
      primaryRows.set(fixture.id, primary);
      primaryAttempts.push({ id: fixture.id, mode, state: "compiled", primary });
    } catch (error) {
      primaryAttempts.push({
        id: fixture.id,
        mode,
        state: "failed",
        error: rootIfErrorDetails(error),
      });
      throw error;
    }
  });
}

for (const fixture of fixtures) {
  test(`${fixture.id}: whole native and reference branches update and unmount in ${mode}`, async (t) => {
    if (!captured && !requireCapture) {
      t.skip("official primary-only tooling; native was not executed and receives no credit");
      return;
    }
    const referenceObservations: unknown[] = [];
    const nativeObservations: unknown[] = [];
    const attempt: Record<string, unknown> = {
      id: fixture.id,
      mode,
      sourceHash: hash(fixture.source),
      referenceObservations,
      nativeObservations,
    };
    runtimeAttempts.push(attempt);
    try {
      assert(captured, "there is no desired-native fallback");
      const row = nativeRows.get(fixture.id);
      const primary = primaryRows.get(fixture.id);
      assert(row && primary, "both whole observed modules must be present");
      Object.assign(attempt, {
        nativeRow: row,
        nativeCodeHash: hash(row.code),
        nativeMapHash: hash(JSON.stringify(row.nativeMap)),
        nativeMapRawHash: hash(row.nativeMapRaw),
        referenceCodeHash: hash(primary.runtimeModule),
      });
      const failures: { phase: string; error: unknown }[] = [];
      let reference: unknown;
      let native: unknown;
      try {
        reference = await executeRootIfComponent(
          primary.runtimeModule,
          fixture,
          false,
          mode,
          (observation) => referenceObservations.push(observation),
        );
        attempt.reference = reference;
      } catch (error) {
        failures.push({ phase: "reference-attempt", error });
      }
      try {
        native = await executeRootIfComponent(row.code, fixture, true, mode, (observation) =>
          nativeObservations.push(observation),
        );
        attempt.native = native;
      } catch (error) {
        failures.push({ phase: "native-attempt", error });
      }
      attempt.failures = failures.map(({ phase, error }) => ({
        phase,
        error: rootIfErrorDetails(error),
      }));
      if (failures.length) throw failures[0].error;
      assert.deepEqual(native, reference);
      attempt.state = "matched";
    } catch (error) {
      attempt.state = "failed";
      attempt.error = rootIfErrorDetails(error);
      throw error;
    }
  });
}

after(() => {
  const common = {
    adapter: "vize_atelier_sfc::compile_native_selected_setup_sfc_dom_with_mode",
    qualification: "observed-unqualified-current-oracles-pending",
    nativeGoldenComparisonPerformed: false,
    mode,
    compiler: pack.compiler,
    runtime: pack.runtime,
    typescript: pack.typescript,
    inputHash: hash(inputRaw),
    sourceCaptureHash: captureRaw ? hash(captureRaw) : null,
  };
  if (primaryPrefix)
    fs.writeFileSync(
      `${primaryPrefix}.${mode}.json`,
      JSON.stringify(
        {
          ...common,
          schema: "vize.native-sfc.original-root-if-primary-capture",
          fixtures: primaryAttempts,
        },
        null,
        2,
      ) + "\n",
    );
  if (runtimePrefix)
    fs.writeFileSync(
      `${runtimePrefix}.${mode}.json`,
      JSON.stringify(
        {
          ...common,
          schema: "vize.native-sfc.original-root-if-runtime-capture",
          host: "vue-createRenderer-custom-host",
          sourceCapture: captured,
          fixtures: runtimeAttempts,
        },
        null,
        2,
      ) + "\n",
    );
  assert.equal(
    primaryRows.size,
    fixtures.length,
    "all full official primary results must be retained",
  );
  if (requireCapture || captured) {
    assert.equal(runtimeAttempts.length, fixtures.length);
    assert(runtimeAttempts.every((attempt) => attempt.state === "matched"));
  }
});
