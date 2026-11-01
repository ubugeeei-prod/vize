// Complete official primary capture shared by the observation and strict replay.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import {
  fromSsrVue,
  hash,
  type ModelSsrFixture,
  type ModelSsrMode,
} from "./native-original-input-model-sfc-ssr-boundary.ts";
import { diagnosticValue } from "./native-original-for-constant-sfc-dom-observation.ts";

const compiler = fromSsrVue("@vue/compiler-sfc") as typeof import("@vue/compiler-sfc");
assert.equal(compiler.version, "3.5.35");
export const typescript = createRequire(new URL("../../package.json", import.meta.url))(
  "typescript",
) as typeof import("typescript");
const ts = typescript;
type ObjectValue = Record<string, unknown>;

export function compileInputModelSsrPrimary(
  fixture: ModelSsrFixture,
  attempt: Record<string, unknown>,
  filename: string,
  mode: ModelSsrMode,
) {
  const observed: ObjectValue = {};
  attempt.compilerObservations = observed;
  const parsed = compiler.parse(fixture.source, { filename });
  Object.assign(observed, {
    parseErrors: parsed.errors,
    descriptorDiagnosticOnly: diagnosticValue(parsed.descriptor),
  });
  assert.deepEqual(parsed.errors, []);
  assert.equal(parsed.descriptor.script, null);
  const setup = parsed.descriptor.scriptSetup;
  const template = parsed.descriptor.template;
  assert(setup && template);
  assert.equal(setup.content, fixture.script);
  assert.equal(template.content, fixture.template);
  const script = compiler.compileScript(parsed.descriptor, {
    id: fixture.id,
    isProd: mode === "production",
    genDefaultAs: "_sfc_main",
  });
  // Keep the complete returned code/map/bindings before any comparison.
  Object.assign(observed, {
    script: script.content,
    scriptMap: script.map,
    bindings: script.bindings,
  });
  assert.deepEqual(Object.keys(script.bindings ?? {}), fixture.bindings);
  const render = compiler.compileTemplate({
    source: fixture.template,
    filename,
    id: fixture.id,
    isProd: mode === "production",
    ssr: true,
    compilerOptions: {
      sourceMap: true,
      mode: "module",
      hoistStatic: false,
      prefixIdentifiers: true,
      comments: true,
      bindingMetadata: script.bindings,
      cacheHandlers: false,
    },
  });
  Object.assign(observed, {
    render: render.code,
    renderMap: render.map,
    renderErrors: render.errors,
    renderTips: render.tips,
  });
  assert.deepEqual(render.errors, []);
  assert.equal(render.code.split("export function ssrRender").length, 2);
  const originalModule =
    script.content +
    "\n;\n" +
    render.code +
    "\n_sfc_main.ssrRender = ssrRender\nexport default _sfc_main\n";
  observed.originalModule = originalModule;
  let runtimeModule = originalModule;
  let transpilation: unknown = null;
  if (setup.lang === "ts") {
    const result = ts.transpileModule(originalModule, {
      fileName: "OriginalInputModelSsr.ts",
      reportDiagnostics: true,
      compilerOptions: {
        target: ts.ScriptTarget.ESNext,
        module: ts.ModuleKind.ESNext,
        newLine: ts.NewLineKind.LineFeed,
      },
    });
    // TS6's exact three-field result drops only its undefined sourceMapText.
    observed.transpilationOwnKeys = Reflect.ownKeys(result);
    const { sourceMapText, ...capturedResult } = result;
    transpilation = { version: ts.version, ...capturedResult };
    observed.transpilation = transpilation;
    assert.deepEqual(result.diagnostics, []);
    assert.deepEqual(Reflect.ownKeys(result), ["outputText", "diagnostics", "sourceMapText"]);
    assert.equal(sourceMapText, undefined);
    runtimeModule = result.outputText;
  }
  const scriptMapRaw = JSON.stringify(script.map);
  const renderMapRaw = JSON.stringify(render.map);
  assert(typeof scriptMapRaw === "string");
  assert(typeof renderMapRaw === "string");
  // MagicString's SourceMap owns exactly these six fields; its public JSON
  // omits only the verified undefined file. Keep the raw map bytes above.
  assert(script.map);
  assert.deepEqual(Reflect.ownKeys(script.map), [
    "version",
    "file",
    "sources",
    "sourcesContent",
    "names",
    "mappings",
  ]);
  const { file: scriptMapFile, ...scriptMapWire } = script.map;
  assert.equal(scriptMapFile, undefined);
  observed.scriptMap = scriptMapWire;
  return {
    id: fixture.id,
    mode,
    source: fixture.source,
    sourceHash: hash(fixture.source),
    parseErrors: parsed.errors,
    script: script.content,
    scriptMap: JSON.parse(scriptMapRaw) as unknown,
    scriptMapRaw,
    bindings: script.bindings,
    render: render.code,
    renderMap: JSON.parse(renderMapRaw) as unknown,
    renderMapRaw,
    renderMapOwnKeys: render.map ? Reflect.ownKeys(render.map) : null,
    renderMapDiagnosticOnly: diagnosticValue(render.map),
    renderMapDescriptorsDiagnosticOnly: render.map
      ? diagnosticValue(Object.getOwnPropertyDescriptors(render.map))
      : null,
    renderErrors: render.errors,
    renderTips: render.tips,
    originalModule,
    runtimeModule,
    transpilation,
  };
}
