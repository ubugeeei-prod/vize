import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import {
  compiler,
  runtime,
  hash,
  checkMap,
  execute,
} from "./support/native-sfc-setup-reference.ts";

const ts = createRequire(new URL("../package.json", import.meta.url))("typescript");
const pack = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../crates/vize_atelier_sfc/tests/fixtures/native_sfc_ts_annotation_setup_vue_3_5_35.json",
      import.meta.url,
    ),
    "utf8",
  ),
);
const capturePath = process.env.VIZE_NATIVE_SFC_TS_ANNOTATION_SETUP_CAPTURE;
const captured = capturePath ? JSON.parse(fs.readFileSync(capturePath, "utf8")) : null;
const executions: unknown[] = [];

test("original annotated TS retains independent pinned Vue and official TypeScript whole reference bytes/maps", () => {
  assert.equal(compiler.version, "3.5.35");
  assert.equal(runtime.version, "3.5.35");
  assert.equal(ts.version, "6.0.3");
  assert.equal(pack.schema, "vize.native-sfc.ts_annotation-setup-dom-reference");
  assert.equal(pack.fixtures.length, 3);
  for (const fixture of pack.fixtures) {
    const parsed = compiler.parse(fixture.source, { filename: "Setup雪🌸.vue" });
    assert.deepEqual(parsed.errors, []);
    const script = compiler.compileScript(parsed.descriptor, {
      id: fixture.id,
      genDefaultAs: "_sfc_main",
    });
    assert.equal(script.content, fixture.referenceScript);
    assert.deepEqual(script.bindings, fixture.referenceBindings);
    assert.deepEqual(Object.keys(script.bindings), fixture.bindings);
    for (const name of fixture.bindings)
      assert.equal(
        script.bindings[name],
        fixture.immutableBindings.includes(name) ? "literal-const" : "setup-let",
      );
    const render = compiler.compileTemplate({
      source: fixture.template,
      filename: "Setup雪🌸.vue",
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
    const original =
      script.content +
      "\n;\n" +
      render.code.replace("export function render", "function render") +
      "\n_sfc_main.render = render\nexport default _sfc_main\n";
    const result = ts.transpileModule(original, {
      fileName: "Setup雪🌸.ts",
      reportDiagnostics: true,
      compilerOptions: {
        target: ts.ScriptTarget.ESNext,
        module: ts.ModuleKind.ESNext,
        sourceMap: true,
        newLine: ts.NewLineKind.LineFeed,
      },
    });
    assert.deepEqual(result.diagnostics, []);
    assert.equal(result.outputText, fixture.referenceModule);
    assert.deepEqual(JSON.parse(result.sourceMapText), fixture.referenceModuleMap);
  }
});

test(
  "source-built annotation capture retains every complete original source, module and map",
  { skip: !captured },
  () => {
    assert.equal(captured.schema, "vize.native-sfc.ts_annotation-setup-capture");
    assert.equal(captured.adapter, "vize_atelier_sfc::compile_native_sfc");
    assert.equal(captured.fixtures.length, 3);
    for (const [index, row] of captured.fixtures.entries()) {
      const fixture = pack.fixtures[index];
      assert.equal(row.id, fixture.id);
      assert.equal(row.source, fixture.source);
      assert.deepEqual(row.bindings, fixture.bindings);
      assert.equal(row.code, fixture.code);
      assert.deepEqual(row.nativeMap, fixture.nativeMap);
      checkMap({ ...fixture, code: row.code, nativeMap: row.nativeMap });
    }
  },
);

for (const fixture of pack.fixtures) {
  test(`${fixture.id} real Vue renders the official whole reference module`, async () => {
    assert.deepEqual(await execute(fixture.referenceModule, fixture), fixture.runtime);
  });
  test(
    `${fixture.id} source-built native module owns real lexical state and two Vue renders`,
    { skip: !captured },
    async () => {
      const row = captured.fixtures.find((row: any) => row.id === fixture.id);
      assert(row);
      assert.equal(row.source, fixture.source);
      assert.deepEqual(await execute(row.code, fixture, true), fixture.runtime);
      executions.push({
        id: fixture.id,
        sourceSha256: hash(row.source),
        codeSha256: hash(row.code),
        nativeSetupInvocations: 1,
        nativeRenders: 2,
      });
      if (process.env.VIZE_NATIVE_SFC_TS_ANNOTATION_SETUP_RUNTIME_CAPTURE)
        fs.writeFileSync(
          process.env.VIZE_NATIVE_SFC_TS_ANNOTATION_SETUP_RUNTIME_CAPTURE,
          JSON.stringify(
            {
              schema: "vize.native-sfc.ts_annotation-setup-runtime",
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
