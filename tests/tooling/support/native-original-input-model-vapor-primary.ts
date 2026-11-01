// Actual pinned compiler-sfc inline Vapor client/SSR graphs, no Vue runtime load.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import {
  fromVaporVue,
  hash,
  type ModelVaporFixture,
  type ModelVaporMode,
  type ObjectValue,
} from "./native-original-input-model-vapor-boundary.ts";
import { diagnosticValue } from "./native-original-for-constant-sfc-dom-observation.ts";

const compiler = fromVaporVue("vue/compiler-sfc") as typeof import("@vue/compiler-sfc");
assert.equal(compiler.version, "3.6.0-rc.9");
export const typescript = createRequire(new URL("../../package.json", import.meta.url))(
  "typescript",
) as typeof import("typescript");
assert.equal(typescript.version, "6.0.3");

function compile(
  fixture: ModelVaporFixture,
  filename: string,
  mode: ModelVaporMode,
  ssr: boolean,
  observations: ObjectValue,
) {
  const parsed = compiler.parse(fixture.source, { filename });
  Object.assign(observations, {
    parseErrors: parsed.errors,
    originalDescriptorDiagnosticOnly: diagnosticValue(parsed.descriptor),
  });
  assert.deepEqual(parsed.errors, []);
  assert.equal(parsed.descriptor.source, fixture.source);
  assert.equal(parsed.descriptor.script, null);
  assert.deepEqual(parsed.descriptor.styles, []);
  assert.deepEqual(parsed.descriptor.cssVars, []);
  const setup = parsed.descriptor.scriptSetup;
  const template = parsed.descriptor.template;
  assert(setup && template);
  assert.equal(setup.content, fixture.script);
  assert.equal(template.content, fixture.template);
  // Actual target selection on the real parse result. All original SFC bytes,
  // offsets, script/template bodies and ASTs stay owned by that descriptor.
  Object.assign(parsed.descriptor, { vapor: true });
  observations.selectedVaporDescriptorDiagnosticOnly = diagnosticValue(parsed.descriptor);
  const options = {
    id: fixture.id,
    inlineTemplate: true,
    isProd: mode === "production",
    sourceMap: true,
    genDefaultAs: "_sfc_main",
    templateOptions: {
      ssr,
      compilerOptions: { comments: true, sourceMap: true },
    },
  };
  observations.compileScriptOptions = options;
  const script = compiler.compileScript(parsed.descriptor, options);
  // Retain every returned field diagnostically as well as complete executable
  // code/raw maps/bindings/imports. Undefined AST/deps fields are not silently
  // replaced by defaults or promoted into another target's contract.
  observations.completeCompilerResultDiagnosticOnly = diagnosticValue(script);
  observations.code = script.content;
  const mapRaw = JSON.stringify(script.map);
  observations.mapRaw = mapRaw;
  assert(typeof mapRaw === "string" && mapRaw.length > 0);
  assert.deepEqual(Object.keys(script.bindings ?? {}), fixture.bindings);
  const originalModule = script.content + "\n;\nexport default _sfc_main\n";
  observations.originalModule = originalModule;
  let runtimeModule = originalModule;
  let transpilation: unknown = null;
  if (setup.lang === "ts") {
    const result = typescript.transpileModule(originalModule, {
      fileName: filename,
      reportDiagnostics: true,
      compilerOptions: {
        target: typescript.ScriptTarget.ESNext,
        module: typescript.ModuleKind.ESNext,
        newLine: typescript.NewLineKind.LineFeed,
      },
    });
    observations.transpilationOwnKeys = Reflect.ownKeys(result);
    observations.transpilationDiagnosticOnly = diagnosticValue(result);
    assert.deepEqual(Reflect.ownKeys(result), ["outputText", "diagnostics", "sourceMapText"]);
    assert.equal(result.sourceMapText, undefined);
    assert.deepEqual(result.diagnostics, []);
    const { sourceMapText, ...capturedResult } = result;
    assert.equal(sourceMapText, undefined);
    transpilation = { version: typescript.version, ...capturedResult };
    runtimeModule = result.outputText;
  }
  return {
    source: fixture.source,
    filename,
    mode,
    ssr,
    targetSelection: "actual parsed descriptor.vapor=true; original source unchanged",
    inlineTemplate: true,
    code: script.content,
    map: JSON.parse(mapRaw) as unknown,
    mapRaw,
    mapDiagnosticOnly: diagnosticValue(script.map),
    bindings: script.bindings,
    importsDiagnosticOnly: diagnosticValue(script.imports),
    completeCompilerResultDiagnosticOnly: diagnosticValue(script),
    originalModule,
    runtimeModule,
    transpilation,
  };
}

export function compileInputModelVaporPrimary(
  fixture: ModelVaporFixture,
  attempt: ObjectValue,
  filename: string,
  mode: ModelVaporMode,
) {
  const clientObservations: ObjectValue = {};
  const ssrObservations: ObjectValue = {};
  attempt.compilerObservations = { client: clientObservations, ssr: ssrObservations };
  const client = compile(fixture, filename, mode, false, clientObservations);
  const ssr = compile(fixture, filename, mode, true, ssrObservations);
  return {
    id: fixture.id,
    mode,
    source: fixture.source,
    sourceHash: hash(fixture.source),
    compiler: `@vue/compiler-sfc@${compiler.version}`,
    runtime: "vue@3.6.0-rc.9",
    typescript: typescript.version,
    client,
    ssr,
    runtimeModule: client.runtimeModule,
  };
}
