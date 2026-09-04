import assert from "node:assert/strict";
import test from "node:test";
import { MUSEA_STATIC_BUILD_ENV, shouldEmitMuseaStaticGallery } from "../static-export.js";
import { shouldApplyMuseaPlugin } from "./apply.js";
import {
  assertStaticPreviewRuntimeSupported,
  resolveStaticPreviewVueVersion,
} from "./static-preview.js";
import { musea } from "./index.js";
import { readMuseaOptions } from "./options.js";

void test("musea plugin is inactive in Vite test mode", () => {
  assert.equal(shouldApplyMuseaPlugin({ command: "serve", mode: "test" }), false);
});

void test("musea plugin remains active during Vite serve outside test mode", () => {
  assert.equal(shouldApplyMuseaPlugin({ command: "serve", mode: "development" }), true);
});

void test("musea plugin remains active during production builds", () => {
  assert.equal(shouldApplyMuseaPlugin({ command: "build", mode: "production" }), true);
});

void test("musea plugin carries VRT options for the CLI config loader", () => {
  const [plugin] = musea({ vrt: { threshold: 0, comparison: { antiAliasing: false } } });

  assert.deepEqual(readMuseaOptions(plugin), {
    vrt: { threshold: 0, comparison: { antiAliasing: false } },
  });
});

void test("storybook compatibility build does not inject static gallery input by default", () => {
  const previousEnv = process.env[MUSEA_STATIC_BUILD_ENV];
  try {
    Reflect.deleteProperty(process.env, MUSEA_STATIC_BUILD_ENV);

    assert.equal(shouldEmitMuseaStaticGallery("build", true), false);
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

    assert.equal(shouldEmitMuseaStaticGallery("build", true), true);
  } finally {
    if (previousEnv === undefined) {
      Reflect.deleteProperty(process.env, MUSEA_STATIC_BUILD_ENV);
    } else {
      process.env[MUSEA_STATIC_BUILD_ENV] = previousEnv;
    }
  }
});

void test("static builds allow legacy Vue preview runtime", () => {
  assert.doesNotThrow(() => assertStaticPreviewRuntimeSupported(2, true));
  assert.doesNotThrow(() => assertStaticPreviewRuntimeSupported(2, false));
  assert.doesNotThrow(() => assertStaticPreviewRuntimeSupported(3, true));
});

void test("static preview runtime infers legacy Vue from host compiler plugins", () => {
  assert.equal(resolveStaticPreviewVueVersion(undefined, [{ name: "vite:vue2" }]), 2);
  assert.equal(resolveStaticPreviewVueVersion(undefined, [{ name: "vite:vue" }]), 3);
  assert.equal(resolveStaticPreviewVueVersion(3, [{ name: "vite:vue2" }]), 3);
});
