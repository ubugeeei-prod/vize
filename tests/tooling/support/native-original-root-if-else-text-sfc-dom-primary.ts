// Complete official primary capture shared by the observation and strict replay.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { compiler as actualCompiler, hash } from "./native-selected-sfc-dom-runtime.ts";
import { diagnosticValue } from "./native-original-for-constant-sfc-dom-observation.ts";
import type {
  RootIfElseTextFixture,
  RootIfElseTextMode,
} from "./native-original-root-if-else-text-sfc-dom-runtime.ts";

const compiler = actualCompiler as typeof import("@vue/compiler-sfc");
export const typescript = createRequire(new URL("../../package.json", import.meta.url))(
  "typescript",
) as typeof import("typescript");
const ts = typescript;
type ObjectValue = Record<string, unknown>;

export function compileRootIfElseTextPrimary(
  fixture: RootIfElseTextFixture,
  attempt: Record<string, unknown>,
  filename: string,
  mode: RootIfElseTextMode,
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
  assert.equal(render.code.split("export function render").length, 2);
  const originalModule =
    script.content +
    "\n;\n" +
    render.code.replace("export function render", "function render") +
    "\n_sfc_main.render = render\nexport default _sfc_main\n";
  observed.originalModule = originalModule;
  let runtimeModule = originalModule;
  let transpilation: unknown = null;
  if (setup.lang === "ts") {
    const result = ts.transpileModule(originalModule, {
      fileName: "OriginalRootIfElseText.ts",
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
    renderErrors: render.errors,
    renderTips: render.tips,
    originalModule,
    runtimeModule,
    transpilation,
  };
}
