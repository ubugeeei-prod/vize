import assert from "node:assert/strict";
import test from "node:test";

import {
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
