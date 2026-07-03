import assert from "node:assert/strict";
import test from "node:test";
import type { Plugin } from "vite";
import { MUSEA_STATIC_BUILD_ENV } from "../static-export.js";
import { shouldApplyMuseaPlugin } from "./apply.js";
import { musea } from "./index.js";
import {
  assertStaticPreviewRuntimeSupported,
  resolveStaticPreviewVueVersion,
} from "./static-preview.js";

void test("musea plugin is inactive in Vite test mode", () => {
  assert.equal(shouldApplyMuseaPlugin({ command: "serve", mode: "test" }), false);
});

void test("musea plugin remains active during Vite serve outside test mode", () => {
  assert.equal(shouldApplyMuseaPlugin({ command: "serve", mode: "development" }), true);
});

void test("musea plugin remains active during production builds", () => {
  assert.equal(shouldApplyMuseaPlugin({ command: "build", mode: "production" }), true);
});

void test("storybook compatibility build does not inject static gallery input by default", () => {
  const previousEnv = process.env[MUSEA_STATIC_BUILD_ENV];
  try {
    Reflect.deleteProperty(process.env, MUSEA_STATIC_BUILD_ENV);
    const plugin = musea({ storybookCompat: true })[0] as Plugin;
    const result = runConfigHook(plugin);

    assert.equal(Object.hasOwn(result, "build"), false);
  } finally {
    if (previousEnv === undefined) {
      Reflect.deleteProperty(process.env, MUSEA_STATIC_BUILD_ENV);
    } else {
      process.env[MUSEA_STATIC_BUILD_ENV] = previousEnv;
    }
  }
});

void test("storybook compatibility keeps explicit Musea static builds", () => {
  const previousEnv = process.env[MUSEA_STATIC_BUILD_ENV];
  try {
    process.env[MUSEA_STATIC_BUILD_ENV] = "1";
    const plugin = musea({ storybookCompat: true })[0] as Plugin;
    const result = runConfigHook(plugin);

    assert.equal(
      result.build?.rollupOptions?.input?.["musea-static-runtime"],
      "virtual:musea-static-runtime",
    );
  } finally {
    if (previousEnv === undefined) {
      Reflect.deleteProperty(process.env, MUSEA_STATIC_BUILD_ENV);
    } else {
      process.env[MUSEA_STATIC_BUILD_ENV] = previousEnv;
    }
  }
});

void test("static builds reject legacy Vue preview runtime explicitly", () => {
  assert.throws(() => assertStaticPreviewRuntimeSupported(2, true), /Vue 3 preview runtime/);
  assert.doesNotThrow(() => assertStaticPreviewRuntimeSupported(2, false));
  assert.doesNotThrow(() => assertStaticPreviewRuntimeSupported(3, true));
});

void test("static preview runtime infers legacy Vue from host compiler plugins", () => {
  assert.equal(resolveStaticPreviewVueVersion(undefined, [{ name: "vite:vue2" }]), 2);
  assert.equal(resolveStaticPreviewVueVersion(undefined, [{ name: "vite:vue" }]), 3);
  assert.equal(resolveStaticPreviewVueVersion(3, [{ name: "vite:vue2" }]), 3);
});

function runConfigHook(plugin: Plugin): {
  build?: { rollupOptions?: { input?: Record<string, string> } };
} {
  assert.equal(typeof plugin.config, "function");
  const configHook = plugin.config as (
    config: { build: { rollupOptions: Record<string, unknown> } },
    env: { command: "build"; mode: string },
  ) => unknown;
  const result = configHook(
    { build: { rollupOptions: {} } },
    { command: "build", mode: "production" },
  );
  assert.ok(result && typeof result === "object" && !("then" in result));
  return result as { build?: { rollupOptions?: { input?: Record<string, string> } } };
}
