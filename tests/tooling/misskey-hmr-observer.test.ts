import assert from "node:assert/strict";
import { test } from "node:test";
import { isMisskeyAuthoredHotUpdate } from "../app/dev/misskey-hmr-observer.ts";
import { MISSKEY_HMR_TARGETS } from "../app/dev/misskey-hmr-targets.ts";

const origin = "http://127.0.0.1:5173";

test("Misskey counts one SFC completion per edit when its CSS module also updates", () => {
  // These are the two module identities in the failed Actions update. Vite's
  // HMRClient logs a completion for each accepted module independently.
  const component = "[vite] hot updated: /src/ui/visitor.vue?vue&vize";
  const style =
    "[vite] hot updated: /src/ui/visitor.vue.__vize_style_0.css?vue=&type=style&index=0&lang=css&vize-file=%2Fsrc%2Fui%2Fvisitor%2Evue";
  const target = MISSKEY_HMR_TARGETS[0];
  const completed: string[] = [];
  const observe = (messages: readonly string[]) => {
    for (const message of messages) {
      if (isMisskeyAuthoredHotUpdate(message, target.moduleSuffix, origin)) completed.push(message);
    }
  };
  observe([component, style]);
  assert.deepEqual(completed, [component]);
  observe([style, component]);
  assert.deepEqual(completed, [component, component]);
  // A duplicate real SFC completion must remain visible to the exact-count gate.
  observe([component]);
  assert.deepEqual(completed, [component, component, component]);
});

test("Misskey matches accepted authored modules, including Vite's via log form", () => {
  for (const { moduleSuffix } of MISSKEY_HMR_TARGETS) {
    for (const moduleId of [
      `${moduleSuffix}?vue&vize`,
      `/vite${moduleSuffix}?vue&vize`,
      `/@fs/authored${moduleSuffix}?vue&vize`,
      `${origin}${moduleSuffix}?vize&vue`,
    ]) {
      for (const via of ["", " via /src/parent.ts"]) {
        assert.equal(
          isMisskeyAuthoredHotUpdate(`[vite] hot updated: ${moduleId}${via}`, moduleSuffix, origin),
          true,
        );
      }
    }
  }
});

test("Misskey rejects style, raw, stale, SSR, foreign and importer-only identities", () => {
  for (const { moduleSuffix } of MISSKEY_HMR_TARGETS) {
    for (const moduleId of [
      `${moduleSuffix}.__vize_style_0.css?vue&vize`,
      `${moduleSuffix}.ts?vue&vize`,
      moduleSuffix,
      `${moduleSuffix}?vue`,
      `${moduleSuffix}?vize`,
      `${moduleSuffix}?vue&vize&vize-ssr`,
      `https://foreign.invalid${moduleSuffix}?vue&vize`,
      `/other.vue?vue&vize&owner=${moduleSuffix}`,
      `/other.vue?vue&vize via ${moduleSuffix}?vue&vize`,
    ]) {
      assert.equal(
        isMisskeyAuthoredHotUpdate(`[vite] hot updated: ${moduleId}`, moduleSuffix, origin),
        false,
      );
    }
    for (const message of [
      `application log: [vite] hot updated: ${moduleSuffix}?vue&vize`,
      `[vite] css hot updated: ${moduleSuffix}?vue&vize`,
      `[vite] hot updated: ${moduleSuffix}?vue&vize trailing text`,
      "[vite] hot updated: http://[invalid",
      "[vite] connected.",
    ]) {
      assert.equal(isMisskeyAuthoredHotUpdate(message, moduleSuffix, origin), false);
    }
  }
});
