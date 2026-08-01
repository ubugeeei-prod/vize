import assert from "node:assert/strict";
import test from "node:test";

void test("Nuxt 2 musea options do not load the ESM-only Musea Vite plugin", async () => {
  const { default: nuxtModule } = await import(new URL("../dist/index.mjs", import.meta.url).href);
  const hookNames: string[] = [];
  const nuxt = {
    _version: "2.17.3",
    options: {
      rootDir: process.cwd(),
      build: { publicPath: "/_nuxt/" },
      router: { base: "/" },
      modules: [],
      buildDir: ".nuxt",
      dev: false,
      vite: {},
    },
    hook(name: string, callback: (...args: unknown[]) => unknown) {
      assert.equal(typeof callback, "function");
      hookNames.push(name);
    },
  };

  await nuxtModule(
    {
      compiler: false,
      lint: false,
      musea: {
        include: ["stories/**/*.art.vue"],
        basePath: "/__musea__",
      },
      compatibility: {
        nuxtVersion: 2,
        vueVersion: "2.7",
        hostCompiler: true,
        webpackVersion: 4,
      },
    },
    nuxt,
  );

  assert.deepEqual(hookNames, ["close", "builder:prepared", "build:templates"]);
  assert.deepEqual(nuxt.options.vite, {});
});
