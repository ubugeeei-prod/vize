import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  compiler,
  runtime,
  fromVue,
  hash,
  checkMap,
  execute,
} from "./support/native-sfc-setup-reference.ts";

for (const [family, count] of [
  ["const", 5],
  ["ts", 3],
] as const) {
  const prefix = family.toUpperCase();
  const pack = JSON.parse(
    fs.readFileSync(
      new URL(
        `../../crates/vize_atelier_sfc/tests/fixtures/native_sfc_${family}_setup_vue_3_5_35.json`,
        import.meta.url,
      ),
      "utf8",
    ),
  );
  const capturePath = process.env[`VIZE_NATIVE_SFC_${prefix}_SETUP_CAPTURE`];
  const captured = capturePath ? JSON.parse(fs.readFileSync(capturePath, "utf8")) : null;
  const executions: unknown[] = [];

  test(`${family} fixtures preserve actual pinned binding classes and complete maps`, () => {
    assert.equal(pack.schema, `vize.native-sfc.${family}-setup-dom-reference`);
    assert.equal(compiler.version, "3.5.35");
    assert.equal(fromVue("vue/package.json").version, "3.5.35");
    assert.equal(pack.fixtures.length, count);
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
      for (const name of fixture.bindings) {
        assert.equal(
          script.bindings[name],
          fixture.immutableBindings.includes(name) ? "literal-const" : "setup-let",
        );
      }
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
      checkMap(fixture);
    }
  });
  test(
    `${family} native capture binds all complete Rust modules/maps to original inputs`,
    { skip: !captured },
    () => {
      assert.equal(captured.schema, `vize.native-sfc.${family}-setup-capture`);
      assert.equal(captured.adapter, "vize_atelier_sfc::compile_native_sfc");
      assert.deepEqual(
        captured.fixtures,
        pack.fixtures.map(({ id, source, code, nativeMap, bindings }: any) => ({
          id,
          source,
          code,
          nativeMap,
          bindings,
        })),
      );
    },
  );

  for (const fixture of pack.fixtures) {
    test(`${fixture.id} pinned complete setup and two Vue renders`, async () => {
      const module =
        fixture.referenceScript +
        "\n;\n" +
        fixture.referenceRender.replace("export function render", "function render") +
        "\n_sfc_main.render = render\nexport default _sfc_main\n";
      assert.deepEqual(await execute(module, fixture), fixture.runtime);
    });
    test(
      `${fixture.id} captured native setup owns lexical state and two Vue renders`,
      { skip: !captured },
      async () => {
        const row = captured.fixtures.find((row: any) => row.id === fixture.id);
        assert(row);
        assert.equal(row.code, fixture.code);
        assert.equal(row.source, fixture.source);
        assert.deepEqual(await execute(row.code, fixture, true), fixture.runtime);
        executions.push({
          id: fixture.id,
          sourceSha256: hash(row.source),
          codeSha256: hash(row.code),
          nativeSetupInvocations: 1,
          nativeRenders: 2,
        });
        if (process.env[`VIZE_NATIVE_SFC_${prefix}_SETUP_RUNTIME_CAPTURE`]) {
          fs.writeFileSync(
            process.env[`VIZE_NATIVE_SFC_${prefix}_SETUP_RUNTIME_CAPTURE`],
            JSON.stringify(
              {
                schema: `vize.native-sfc.${family}-setup-runtime`,
                runtime: "vue@3.5.35",
                executions,
              },
              null,
              2,
            ) + "\n",
          );
        }
      },
    );
  }
}
