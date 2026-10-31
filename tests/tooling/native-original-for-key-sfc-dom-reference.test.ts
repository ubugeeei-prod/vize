import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { after, test } from "node:test";
import { compiler, runtime, hash } from "./support/native-selected-sfc-dom-runtime.ts";
import {
  assertForKeyFixture,
  executeForKeyComponent,
  type ForKeyFixture,
} from "./support/native-original-for-key-sfc-dom-runtime.ts";

const ts = createRequire(new URL("../package.json", import.meta.url))("typescript");
const inputRaw = fs.readFileSync(
  new URL(
    "../../crates/vize_atelier_sfc/tests/fixtures/native_original_for_key_inputs_vue_3_5_35.json",
    import.meta.url,
  ),
  "utf8",
);
const pack = JSON.parse(inputRaw);
const goldens = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_original_for_key_sfc_vue_3_5_35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
assert.equal(goldens.schema, "vize.native-sfc.original-for-key-complete-goldens");
assert.equal(goldens.inputHash, hash(inputRaw));
assert.equal(goldens.filename, pack.filename);
const mode = process.env.NODE_ENV ?? "development";
assert(mode === "development" || mode === "production", "only the two declared Vue modes");
assert.equal(pack.schema, "vize.native-sfc.original-for-key-inputs");
assert.equal(pack.fixtures.length, 9);
const fixtures: ForKeyFixture[] = pack.fixtures;
for (const fixture of fixtures) assertForKeyFixture(fixture);
assert.equal(new Set(fixtures.map((fixture) => fixture.id)).size, 9);
const capturePath = process.env.VIZE_NATIVE_ORIGINAL_FOR_KEY_SFC_DOM_CAPTURE;
const requireCapture = process.env.VIZE_NATIVE_ORIGINAL_FOR_KEY_SFC_DOM_REQUIRE_CAPTURE === "1";
const captureRaw = capturePath ? fs.readFileSync(capturePath, "utf8") : null;
const captured = captureRaw ? JSON.parse(captureRaw) : null;
const primaryPrefix = process.env.VIZE_NATIVE_ORIGINAL_FOR_KEY_SFC_DOM_PRIMARY_CAPTURE;
const runtimePrefix = process.env.VIZE_NATIVE_ORIGINAL_FOR_KEY_SFC_DOM_RUNTIME_CAPTURE;
type NativeRow = {
  id: string;
  source: string;
  code: string;
  nativeMap: Record<string, unknown>;
  nativeMapRaw: string;
};
const nativeRows = new Map<string, NativeRow>();
const primaryRows = new Map<string, ReturnType<typeof compilePrimary>>();
const primaryAttempts: unknown[] = [];
const runtimeAttempts: unknown[] = [];
let nativeGoldenComparisonPerformed = false;
let nativeGoldensMatched = false;
const errorRecord = (error: unknown) =>
  error instanceof Error
    ? { name: error.name, message: error.message, stack: error.stack }
    : { thrown: String(error) };

function compilePrimary(fixture: ForKeyFixture) {
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
      fileName: "OriginalForKey.ts",
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
    // The pinned API owns this undefined field when no TS map was requested.
    // Verify it exactly; JSON retains every remaining result field.
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

test("nine original ForKey inputs use complete independently reviewed current oracles", () => {
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
  assert.deepEqual(goldens.compiler, pack.compiler);
  assert.deepEqual(goldens.runtime, pack.runtime);
  assert.deepEqual(goldens.typescript, pack.typescript);
  assert.equal(goldens.native.fixtures.length, fixtures.length);
  assert.equal(goldens.native.custody.length, fixtures.length);
  assert.equal(goldens.native.attempts.length, fixtures.length);
  assert.equal(goldens.primary[mode].length, fixtures.length);
  assert.equal(goldens.runtimeResults[mode].length, fixtures.length);
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
        "new native or primary bytes need real capture",
      );
    }
  }
  if (requireCapture) {
    assert(captured, "hosted acceptance requires the actual source-built Rust capture");
    assert(primaryPrefix && runtimePrefix, "hosted acceptance must retain both evidence prefixes");
  }
  if (!captured) return;
  if (requireCapture) {
    assert.equal(captured.sourceRevision, process.env.GITHUB_SHA);
  }
  assert.equal(captured.schema, "vize.native-sfc.original-for-key-capture");
  assert.equal(captured.adapter, "vize_atelier_sfc::compile_native_selected_setup_sfc_dom");
  assert.equal(captured.qualification, "current-complete-native-goldens-matched");
  assert.equal(captured.nativeGoldenComparisonPerformed, true);
  assert.deepEqual(captured.failures, []);
  nativeGoldenComparisonPerformed = true;
  assert.deepEqual(captured.fixtures, goldens.native.fixtures);
  assert.deepEqual(captured.custody, goldens.native.custody);
  assert.deepEqual(captured.attempts, goldens.native.attempts);
  nativeGoldensMatched = true;
  assert(Array.isArray(captured.fixtures));
  assert.deepEqual(
    captured.fixtures.map((row: NativeRow) => row.id),
    fixtures.map((fixture) => fixture.id),
  );
  assert(Array.isArray(captured.custody) && captured.custody.length === 9);
  assert(Array.isArray(captured.attempts) && captured.attempts.length === 9);
  for (const [index, row] of captured.fixtures.entries()) {
    assert.equal(row.source, fixtures[index].source);
    assert.equal(typeof row.code, "string");
    assert(row.code.length > 0);
    assert(row.nativeMap && typeof row.nativeMap === "object" && !Array.isArray(row.nativeMap));
    assert.equal(typeof row.nativeMapRaw, "string");
    assert.deepEqual(JSON.parse(row.nativeMapRaw), row.nativeMap);
    assert.equal(row.nativeMap.version, 3);
    assert.equal(row.nativeMap.file, pack.filename);
    assert.deepEqual(row.nativeMap.sources, [pack.filename]);
    assert.deepEqual(row.nativeMap.sourcesContent, [row.source]);
    assert(Array.isArray(row.nativeMap.names));
    assert(row.nativeMap.names.every((name: unknown) => typeof name === "string"));
    assert.equal(new Set(row.nativeMap.names).size, row.nativeMap.names.length);
    assert.equal(typeof row.nativeMap.mappings, "string");
    assert(row.nativeMap.mappings.length > 0);
    nativeRows.set(row.id, row);
  }
});

for (const [index, fixture] of fixtures.entries()) {
  test(`${fixture.id}: retain the complete official Vue script, render and maps`, () => {
    try {
      const primary = compilePrimary(fixture);
      primaryRows.set(fixture.id, primary);
      const result = { id: fixture.id, state: "compiled", primary };
      primaryAttempts.push(result);
      assert.deepEqual(result, goldens.primary[mode][index]);
    } catch (error) {
      primaryAttempts.push({ id: fixture.id, state: "failed", error: errorRecord(error) });
      throw error;
    }
  });
}

for (const [index, fixture] of fixtures.entries()) {
  test(`${fixture.id}: real native and official key components match through update and unmount`, async (t) => {
    if (!captured && !requireCapture) {
      t.skip("source-only tooling mode; no native capture or qualification credit");
      return;
    }
    const attempt: Record<string, unknown> = { id: fixture.id, sourceHash: hash(fixture.source) };
    runtimeAttempts.push(attempt);
    try {
      assert(captured, "actual Rust capture is required; there is no desired-native fallback");
      const row = nativeRows.get(fixture.id);
      const primary = primaryRows.get(fixture.id);
      assert(row && primary, "both complete observed modules must be present");
      Object.assign(attempt, {
        nativeRow: row,
        nativeCodeHash: hash(row.code),
        nativeMapHash: hash(JSON.stringify(row.nativeMap)),
        nativeMapRawHash: hash(row.nativeMapRaw),
        referenceCodeHash: hash(primary.runtimeModule),
      });
      const reference = await executeForKeyComponent(primary.runtimeModule, fixture, false);
      attempt.reference = reference;
      const native = await executeForKeyComponent(row.code, fixture, true);
      attempt.native = native;
      assert.deepEqual(native, reference);
      attempt.state = "matched";
      assert.deepEqual(attempt, goldens.runtimeResults[mode][index]);
    } catch (error) {
      attempt.state = "failed";
      attempt.error = errorRecord(error);
      throw error;
    }
  });
}

after(() => {
  const primaryMatched =
    primaryAttempts.length === fixtures.length &&
    primaryAttempts.every((attempt: any) => attempt.state === "compiled");
  const runtimeMatched =
    runtimeAttempts.length === fixtures.length &&
    runtimeAttempts.every((attempt: any) => attempt.state === "matched");
  const common = {
    adapter: "vize_atelier_sfc::compile_native_selected_setup_sfc_dom",
    qualification:
      nativeGoldensMatched && primaryMatched && runtimeMatched
        ? "complete-current-oracles-matched"
        : !captured && primaryMatched
          ? "official-primary-only-native-not-executed"
          : "failed-current-complete-oracles",
    nativeGoldenComparisonPerformed,
    mode,
    compiler: pack.compiler,
    runtime: pack.runtime,
    typescript: pack.typescript,
    inputHash: hash(inputRaw),
    sourceCaptureHash: captureRaw ? hash(captureRaw) : null,
  };
  if (primaryPrefix) {
    fs.writeFileSync(
      `${primaryPrefix}.${mode}.json`,
      JSON.stringify(
        {
          ...common,
          schema: "vize.native-sfc.original-for-key-primary-capture",
          fixtures: primaryAttempts,
        },
        null,
        2,
      ) + "\n",
    );
  }
  if (runtimePrefix) {
    fs.writeFileSync(
      `${runtimePrefix}.${mode}.json`,
      JSON.stringify(
        {
          ...common,
          schema: "vize.native-sfc.original-for-key-runtime-capture",
          host: "vue-createRenderer-custom-host",
          sourceCapture: captured,
          fixtures: runtimeAttempts,
        },
        null,
        2,
      ) + "\n",
    );
  }
  assert.equal(
    primaryRows.size,
    fixtures.length,
    "all complete official primary outputs must be retained",
  );
  if (requireCapture || captured) {
    assert.equal(runtimeAttempts.length, fixtures.length);
    assert(runtimeAttempts.every((attempt: any) => attempt.state === "matched"));
  }
});
