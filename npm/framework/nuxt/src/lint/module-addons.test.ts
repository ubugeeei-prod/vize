import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

type Hook = (...args: unknown[]) => unknown;

void test("Nuxt module regenerates lint globals from live registries before later addons", async (t) => {
  const { default: nuxtModule } = await import(
    new URL("../../dist/index.mjs", import.meta.url).href
  );
  const rootDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-module-lint-"));
  t.after(() => fs.rmSync(rootDir, { recursive: true, force: true }));
  const hooks = new Map<string, Hook[]>();
  const nuxt = {
    _version: "4.0.0",
    options: {
      rootDir,
      srcDir: path.join(rootDir, "app"),
      buildDir: path.join(rootDir, ".nuxt"),
      builder: "vite",
      modules: [],
      dir: {},
      _layers: [{ config: { srcDir: path.join(rootDir, "app") } }],
      dev: false,
    },
    hook(name: string, callback: Hook) {
      hooks.set(name, [...(hooks.get(name) ?? []), callback]);
    },
    async callHook(name: string, ...args: unknown[]) {
      for (const callback of hooks.get(name) ?? []) await callback(...args);
    },
  };
  nuxt.hook("vize:lint:config:addons", (value) => {
    const addons = value as Array<{
      name: string;
      getConfigs(): Array<{ name: string; globals: Record<string, "readonly"> }>;
    }>;
    addons.push({
      name: "later-module",
      getConfigs: () => [{ name: "nuxt/later-module", globals: { afterImports: "readonly" } }],
    });
  });

  await nuxtModule({ compiler: false, lint: { autoInit: false }, musea: false }, nuxt);
  const generated = path.join(rootDir, ".nuxt", "oxlint.config.json");
  const initial = JSON.parse(fs.readFileSync(generated, "utf8")) as {
    globals: Record<string, string>;
  };
  assert.deepEqual(Object.keys(initial.globals), ["$fetch", "afterImports"]);

  await nuxt.callHook("imports:context", {
    getImports: async () => [
      { from: "vue", name: "watch", as: "observe" },
      { from: "#app", name: "useRoute" },
      { from: "#app", name: "alpha", as: "constructor" },
    ],
  });
  await nuxt.callHook("nitro:init", {
    unimport: {
      getImports: async () => [{ from: "#app", name: "beta", as: "__proto__" }],
    },
  });
  await nuxt.callHook("builder:generateApp");

  const regenerated = JSON.parse(fs.readFileSync(generated, "utf8")) as {
    globals: Record<string, string>;
  };
  assert.deepEqual(Object.keys(regenerated.globals), [
    "$fetch",
    "constructor",
    "__proto__",
    "useRoute",
    "observe",
    "afterImports",
  ]);
  assert.deepEqual(new Set(Object.values(regenerated.globals)), new Set(["readonly"]));
});
