import assert from "node:assert/strict";
import { test } from "node:test";

import {
  appendSsrModuleRegistration,
  ssrModuleRegistrationCode,
  toManifestModuleId,
} from "./ssr-modules.ts";

const ROOT = "/project";
const PAGE = "/project/app/pages/index.vue";

void test("the manifest key is the root-relative path with POSIX separators", () => {
  assert.equal(toManifestModuleId(PAGE, ROOT), "app/pages/index.vue");
  assert.equal(toManifestModuleId("/project/App.vue", ROOT), "App.vue");
});

void test("a path outside the root keeps its absolute form instead of a ../ chain", () => {
  // `vue-bundle-renderer` looks the key up in the client manifest; a `../`
  // chain matches nothing there and would silently drop the stylesheet.
  const outside = toManifestModuleId("/elsewhere/lib/Widget.vue", ROOT);
  assert.equal(outside, "/elsewhere/lib/Widget.vue");
  assert.ok(!outside.startsWith(".."));
});

void test("a root-relative filename beginning with .. is still keyed relative to the root", () => {
  // `path.relative` yields "..widget.vue" here: no parent step, so the manifest
  // key stays root-relative rather than falling back to the absolute path.
  assert.equal(toManifestModuleId("/project/..widget.vue", ROOT), "..widget.vue");
  assert.equal(toManifestModuleId("/project/app/..widget.vue", ROOT), "app/..widget.vue");
});

void test("the registration wraps setup and adds the module to ssrContext", () => {
  const code = ssrModuleRegistrationCode(PAGE, ROOT);

  assert.match(code, /import \{ useSSRContext as __vize_useSSRContext \} from "vue";/);
  assert.match(code, /const __vize_sfc_setup = _sfc_main\.setup;/);
  // `useSSRContext()` returns undefined outside a request render. Dereferencing
  // it there is a TypeError inside setup, which fails the whole prerender with a
  // bare 500 rather than just losing a stylesheet link.
  assert.match(code, /if \(ssrContext\) \{/);
  assert.match(
    code,
    /\(ssrContext\.modules \|\| \(ssrContext\.modules = new Set\(\)\)\)\.add\("app\/pages\/index\.vue"\);/,
  );
  // `useSSRContext()` is undefined outside a request render; dereferencing it
  // there is a TypeError that fails the whole prerender.
  assert.match(code, /if \(ssrContext\) \{/);
  // An SFC without its own `setup` must still render.
  assert.match(code, /return __vize_sfc_setup \? __vize_sfc_setup\(props, ctx\) : undefined;/);
});

void test("appending leaves the emitted module intact and adds the registration once", () => {
  const emitted = 'const _sfc_main = { name: "Index" };\nexport default _sfc_main;';

  const once = appendSsrModuleRegistration(emitted, PAGE, ROOT, true);
  assert.ok(once.startsWith(emitted), "the emitted module must be preserved verbatim");
  assert.equal(once.match(/__vize_useSSRContext/g)?.length, 2, "one import, one call");

  const twice = appendSsrModuleRegistration(once, PAGE, ROOT, true);
  assert.equal(twice, once, "appending again must be a no-op");
});

void test("a client build is a no-op", () => {
  const emitted = "const _sfc_main = {};\nexport default _sfc_main;";
  assert.equal(appendSsrModuleRegistration(emitted, PAGE, ROOT, false), emitted);
});

void test("a module with no component object is left untouched", () => {
  // Render-function-only output and boundary placeholders have no `_sfc_main`
  // to wrap; wrapping a missing binding would be a ReferenceError at runtime.
  for (const emitted of [
    "export function render(_ctx, _cache) { return null }",
    'import { defineComponent } from "vue";\nexport default defineComponent({});',
    // `_sfc_main` occurs only as text, so there is still nothing to wrap.
    'export function render(_ctx) { return _ctx.h("pre", "_sfc_main") }',
    "// _sfc_main is attached by the client output, not here\nexport function render() {}",
  ]) {
    assert.equal(appendSsrModuleRegistration(emitted, PAGE, ROOT, true), emitted);
  }
});

void test("an SFC that already uses the helper names still registers, with fresh names", () => {
  // Keying idempotency on `__vize_useSSRContext` would skip this component and
  // cost it its initial stylesheet; re-declaring the name would be a SyntaxError.
  const emitted = [
    'import { useSSRContext as __vize_useSSRContext } from "vue";',
    "const __vize_sfc_setup = 1;",
    'const _sfc_main = { name: "Index" };',
    "export default _sfc_main;",
  ].join("\n");

  const once = appendSsrModuleRegistration(emitted, PAGE, ROOT, true);
  assert.ok(once.startsWith(emitted), "the emitted module must be preserved verbatim");
  assert.match(once, /import \{ useSSRContext as __vize_useSSRContext2 \} from "vue";/);
  assert.match(once, /const __vize_sfc_setup2 = _sfc_main\.setup;/);
  assert.match(once, /\(ssrContext\.modules \|\| \(ssrContext\.modules = new Set\(\)\)\)\.add\(/);
  assert.equal(
    once.match(/const __vize_sfc_setup\b/g)?.length,
    1,
    "the existing binding must not be re-declared",
  );

  assert.equal(
    appendSsrModuleRegistration(once, PAGE, ROOT, true),
    once,
    "appending again is a no-op",
  );
});
