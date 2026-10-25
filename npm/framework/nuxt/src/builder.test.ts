import assert from "node:assert/strict";
import test from "node:test";

import {
  getDetectedNuxtMajor,
  getDetectedNuxtBuilderKind,
  getNuxtBuilderKind,
  hasNuxtViteCompilerSupport,
  isViteNuxtBuilder,
  isWebpackNuxtBuilder,
} from "./builder.ts";

void test("Nuxt rolldown-vite builder is treated as Vite-compatible", () => {
  assert.equal(isViteNuxtBuilder("rolldown-vite"), true);
  assert.equal(isViteNuxtBuilder("@nuxt/rolldown-vite-builder"), true);
  assert.equal(getNuxtBuilderKind("rolldown-vite"), "vite");
});

void test("Nuxt classic Vite and webpack builder names keep their existing classification", () => {
  assert.equal(isViteNuxtBuilder("vite"), true);
  assert.equal(isViteNuxtBuilder("@nuxt/vite-builder"), true);
  assert.equal(isWebpackNuxtBuilder("webpack"), true);
  assert.equal(isWebpackNuxtBuilder("@nuxt/webpack-builder"), true);
  assert.equal(getNuxtBuilderKind("@nuxt/webpack-builder"), "webpack");
});

void test("unknown or missing Nuxt builders stay unsupported", () => {
  assert.equal(isViteNuxtBuilder("custom-builder"), false);
  assert.equal(isWebpackNuxtBuilder("custom-builder"), false);
  assert.equal(getNuxtBuilderKind("custom-builder"), "unsupported");
  assert.equal(getNuxtBuilderKind(undefined), "unsupported");
});

void test("Nuxt 2 webpack does not enable Nitro-only compiler setup from a vite option", () => {
  assert.equal(
    hasNuxtViteCompilerSupport({ options: { builder: "@nuxt/webpack-builder", vite: {} } }),
    false,
  );
  assert.equal(
    hasNuxtViteCompilerSupport({ _version: "2.17.3", options: { builder: "webpack" } }),
    false,
  );
  assert.equal(
    hasNuxtViteCompilerSupport({ options: { builder: "@nuxt/vite-builder", vite: {} } }),
    true,
  );
});

void test("the public Nuxt 2 constructor version is detected without private instance fields", () => {
  class Nuxt2 {
    static get version() {
      return "v2.17.3";
    }
  }
  assert.equal(getDetectedNuxtMajor(new Nuxt2()), 2);
  assert.equal(getDetectedNuxtMajor({ constructor: { version: "v2.17.3-development" } }), 2);
  assert.equal(getDetectedNuxtMajor({ _version: "3.19.3", constructor: Nuxt2 }), 3);
  assert.equal(getDetectedNuxtMajor({ version: "4.1.0" }), 4);
  assert.equal(getDetectedNuxtMajor({ options: { _nuxtVersion: "2.17.3" } }), 2);
  assert.equal(getDetectedNuxtMajor({ constructor: { version: "unknown" } }), null);
  assert.equal(getDetectedNuxtMajor(undefined), null);
});

void test("Nuxt 2 checker selects its actual default or Bridge builder", () => {
  class Nuxt2 {
    static version = "v2.17.3";
    options: { builder?: unknown; bridge?: false | { vite?: boolean } } = {};
  }
  const nuxt = new Nuxt2();
  assert.equal(getDetectedNuxtBuilderKind(nuxt), "webpack");
  nuxt.options.bridge = { vite: true };
  assert.equal(getDetectedNuxtBuilderKind(nuxt), "vite");
  nuxt.options.bridge = { vite: false };
  assert.equal(getDetectedNuxtBuilderKind(nuxt), "webpack");
  nuxt.options.builder = "custom-builder";
  assert.equal(getDetectedNuxtBuilderKind(nuxt), "unsupported");
  assert.equal(getDetectedNuxtBuilderKind({ options: {} }), "unsupported");
  assert.equal(getDetectedNuxtBuilderKind({ options: { builder: "vite" } }), "vite");
});
