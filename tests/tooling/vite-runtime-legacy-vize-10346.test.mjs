import assert from "node:assert/strict";
import { test } from "node:test";
import { resolveViteConfigExport } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";

const legacyVize = {
  lint: "lint.vize is no longer supported; pass { lint: false } as defineConfig's integration options to disable the Vite+ native lint task while keeping CLI lint enabled, and put shared lint settings in vue.linter",
  fmt: "fmt.vize: configuration is no longer supported; move Vue settings to vue: in vite.config.*",
  pack: "pack.vize: configuration is no longer supported; move Vue settings to vue: in vite.config.*",
};

test("retired lint.vize, fmt.vize, and pack.vize match the Vite+ migration text", async () => {
  for (const key of ["lint", "fmt", "pack"]) {
    await assert.rejects(resolveViteConfigExport({ [key]: { vize: false } }), {
      name: "Error",
      message: legacyVize[key],
    });
    assert.equal(legacyVize[key].includes(`${key}.vue`), false);
  }
});

test("lint.vue false still disables the linter", async () => {
  assert.deepEqual(await resolveViteConfigExport({ lint: { vue: false } }), {
    typeChecker: { jsxTypecheck: true },
    linter: { enabled: false },
  });
});
