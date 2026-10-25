import assert from "node:assert/strict";
import { test } from "node:test";
import { resolveViteConfigExport } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";

test("Vitest boolean project inheritance remains host-owned alongside Vite options", async () => {
  for (const inherited of [true, false]) {
    for (const host of [
      { resolve: { alias: { "#x": "/dev/null" } } },
      { css: {} },
      { define: { __UNIT__: true } },
      {},
    ]) {
      const source = {
        extends: inherited,
        ...host,
        test: { name: "unit", environment: "node", include: ["src/**/*.spec.ts"] },
        vue: { compiler: { vapor: false }, typeChecker: { enabled: false } },
      };
      const before = structuredClone(source);
      const expected = {
        typeChecker: { jsxTypecheck: true, enabled: false },
        compiler: { vapor: false },
      };
      assert.deepEqual(await resolveViteConfigExport(source), expected);
      assert.deepEqual(await resolveViteConfigExport(async () => source), expected);
      assert.deepEqual(source, before);
    }
  }
});

test("host inheritance does not admit retired native extends or unknown Vue keys", async () => {
  for (const inherited of [{}, "./base.ts", [], null, undefined]) {
    await assert.rejects(resolveViteConfigExport({ extends: inherited }), /Custom config extends/);
  }
  await assert.rejects(
    resolveViteConfigExport({ extends: true, vue: { extends: true } }),
    /vue: unknown option "extends"/,
  );
  await assert.rejects(
    resolveViteConfigExport({ extends: false, vize: {} }),
    /vize: configuration is no longer supported/,
  );
});
