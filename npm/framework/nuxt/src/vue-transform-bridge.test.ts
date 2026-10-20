import assert from "node:assert/strict";
import { test } from "node:test";
import { patchNuxtVueTypeGatedTransform, toNuxtVueTransformId } from "./vue-transform-bridge.ts";

void test("Vize generated modules satisfy Nuxt's script transform identity", () => {
  const isNuxtScriptTransform = (id: string): boolean => {
    const search = id.slice(id.indexOf("?"));
    const type = /[?&]setup(?:=|&|$)/.test(search)
      ? "script"
      : /[?&]type=([^&]*)/.exec(search)?.[1];
    return /[?&]vue(?:&|$)/.test(search) && type === "script";
  };
  assert.equal(isNuxtScriptTransform("/app/pages/index.vue?vue&vize"), false);
  assert.equal(
    toNuxtVueTransformId("/app/pages/index.vue?vue&vize"),
    "/app/pages/index.vue?vue&vize&type=script",
  );
  assert.equal(isNuxtScriptTransform(toNuxtVueTransformId("/app/pages/index.vue?vue&vize")), true);
  assert.equal(
    toNuxtVueTransformId("\0/app/pages/index.vue.ts"),
    "\0/app/pages/index.vue.ts?vue&type=script",
  );
  assert.equal(
    toNuxtVueTransformId("/app/pages/index.vue?vue&vize&used=true"),
    "/app/pages/index.vue?vue&vize&used=true&type=script",
  );
  for (const id of [
    "/app/pages/index.vue",
    "/app/pages/index.vue?vue&type=style&index=0",
    "/app/pages/index.vue?raw",
    "/app/pages/index.ts?vue&vize",
  ]) {
    assert.equal(toNuxtVueTransformId(id), id);
  }
});

void test("Nuxt type-gated transforms receive the compatible identity", () => {
  const calls: string[] = [];
  const plugin = {
    name: "nuxt:route-injection-plugin",
    transform(code: string, id: string) {
      calls.push(id);
      return `${id}:${code}`;
    },
  };
  patchNuxtVueTypeGatedTransform(plugin);
  assert.equal(
    plugin.transform("render", "/app/pages/index.vue?vue&vize"),
    "/app/pages/index.vue?vue&vize&type=script:render",
  );
  assert.deepEqual(calls, ["/app/pages/index.vue?vue&vize&type=script"]);

  patchNuxtVueTypeGatedTransform(plugin);
  plugin.transform("render", "/app/pages/about.vue?vue&vize");
  assert.deepEqual(calls, [
    "/app/pages/index.vue?vue&vize&type=script",
    "/app/pages/about.vue?vue&vize&type=script",
  ]);
});

void test("handler transforms and unrelated plugins preserve their contract", () => {
  const calls: string[] = [];
  const routePlugin = {
    name: "nuxt:vue-async-context",
    transform: {
      handler(_code: string, id: string) {
        calls.push(id);
        return null;
      },
    },
  };
  patchNuxtVueTypeGatedTransform(routePlugin);
  routePlugin.transform.handler("code", "/app/Page.vue?vue&vize");
  routePlugin.transform.handler("code", "/app/Page.vue?vue&type=style&index=0");
  assert.deepEqual(calls, [
    "/app/Page.vue?vue&vize&type=script",
    "/app/Page.vue?vue&type=style&index=0",
  ]);

  const unrelated = {
    name: "third-party:transform",
    transform: (_code: string, id: string) => id,
  };
  const original = unrelated.transform;
  patchNuxtVueTypeGatedTransform(unrelated);
  assert.equal(unrelated.transform, original);
});
