import assert from "node:assert/strict";
import test from "node:test";
import { shouldApplyMuseaPlugin } from "./apply.js";
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
