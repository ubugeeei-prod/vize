import { test } from "node:test";
import { vizeUnplugin } from "./unplugin.ts";
import type { VizeUnpluginOptions } from "./types.ts";

type BunHooks = {
  loader(this: void, code: string, id: string): "css" | "js";
  setup(this: void, build: { config: { define?: Record<string, string> } }): void;
};

function bunHooks(options: VizeUnpluginOptions): BunHooks {
  const plugin = vizeUnplugin.raw(options, {
    framework: "bun",
    versions: { bun: "1.3.14", unplugin: "3.4.0" },
  });
  if (!plugin.bun) throw new Error("expected Bun hook object");
  return plugin.bun as BunHooks;
}

void test("Bun hooks select JavaScript for compiled SFCs and CSS for style modules", (t) => {
  const { loader } = bunHooks({ isProduction: true });
  t.assert.equal(loader("", "/project/App.vue"), "js");
  t.assert.equal(loader("", "/project/Component.tsx"), "js");
  t.assert.equal(loader("", "/project/App.vue?vue&type=style&index=0&lang.css"), "css");
});

void test("Bun hooks install Vue feature flags without replacing user definitions", (t) => {
  const { setup } = bunHooks({ isProduction: true });
  const build = {
    config: {
      define: {
        __VUE_OPTIONS_API__: JSON.stringify(false),
        CUSTOM_FLAG: JSON.stringify("kept"),
      },
    },
  };
  setup(build);
  t.assert.deepEqual(build.config.define, {
    __VUE_OPTIONS_API__: JSON.stringify(false),
    __VUE_PROD_DEVTOOLS__: JSON.stringify(false),
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: JSON.stringify(false),
    CUSTOM_FLAG: JSON.stringify("kept"),
  });
});

void test("Bun hooks remain non-invasive with the host compiler", (t) => {
  const { setup } = bunHooks({ compatibility: { hostCompiler: true } });
  const build: { config: { define?: Record<string, string> } } = { config: {} };
  setup(build);
  t.assert.equal(build.config.define, undefined);
});
