import assert from "node:assert/strict";
import { test } from "node:test";

import { MISSKEY_HMR_TARGETS } from "../app/dev/misskey-hmr-targets.ts";
import {
  isVizeClientCompletion,
  isVizeClientModuleUrl,
  NUXT_UI_HMR_MODULE_PATH,
} from "../app/dev/vize-client-module.ts";

const origin = "http://localhost:5173";
const suffixes = [
  ...MISSKEY_HMR_TARGETS.map((target) => target.moduleSuffix),
  NUXT_UI_HMR_MODULE_PATH,
];

test("App HMR observers consume the actual Vite-visible module producer", async () => {
  const { toPluginVisibleVirtualId } = await import("../../npm/builder/vite/src/virtual.ts");
  for (const suffix of suffixes) {
    const moduleId = toPluginVisibleVirtualId(`/authored${suffix}`);
    assert.equal(moduleId, `/authored${suffix}?vue&vize`);
    assert.equal(isVizeClientModuleUrl(`${origin}${moduleId}`, suffix, origin), true);
    const hotId = toPluginVisibleVirtualId(`/authored${suffix}`, false, "?t=123");
    assert.equal(hotId, `/authored${suffix}?vue&vize&t=123`);
    assert.equal(isVizeClientModuleUrl(`${origin}${hotId}`, suffix, origin, true), true);
  }
});

test("App HMR observers reject stale virtual TypeScript suffixes", () => {
  for (const suffix of suffixes) {
    assert.equal(isVizeClientModuleUrl(`${origin}${suffix}.ts?vue&vize`, suffix, origin), false);
  }
});

test("App HMR observers reject raw, SSR, and incomplete query identities", () => {
  for (const suffix of suffixes) {
    for (const query of ["", "?vue", "?vize", "?vue&vize-ssr", "?vue&vize&vize-ssr"]) {
      assert.equal(isVizeClientModuleUrl(`${origin}${suffix}${query}`, suffix, origin), false);
    }
  }
});

test("App HMR observers reject foreign origins and wrong authored files", () => {
  for (const suffix of suffixes) {
    assert.equal(
      isVizeClientModuleUrl(`https://foreign.invalid${suffix}?vue&vize`, suffix, origin),
      false,
    );
    assert.equal(isVizeClientModuleUrl(`${origin}${suffix}Other?vue&vize`, suffix, origin), false);
    assert.equal(isVizeClientModuleUrl(`${origin}/other.vue?vue&vize`, suffix, origin), false);
    assert.equal(isVizeClientModuleUrl("not a URL", suffix, origin), false);
  }
});

test("App HMR observers require a fresh timestamp only for hot requests", () => {
  for (const suffix of suffixes) {
    const url = `${origin}${suffix}?vue&vize`;
    assert.equal(isVizeClientModuleUrl(url, suffix, origin), true);
    assert.equal(isVizeClientModuleUrl(url, suffix, origin, true), false);
    assert.equal(isVizeClientModuleUrl(`${url}&t=123`, suffix, origin, true), true);
  }
});

test("Vite completion identity keeps component and imported CSS owners separate", () => {
  const base = `${origin}/vite/`;
  const component = "/src/ui/visitor.vue?vue&vize";
  const style = "/src/ui/visitor.vue.__vize_style_0.css?vue=&type=style&index=0&lang=css";
  // Authored combined update control, not a recovered failed-run payload.
  const combined = {
    type: "update",
    updates: [
      { type: "js-update", path: component, acceptedPath: component },
      { type: "js-update", path: style, acceptedPath: style },
    ],
  };
  const messages = combined.updates.map(
    ({ acceptedPath }) => `[vite] hot updated: ${acceptedPath}`,
  );
  assert.deepEqual(
    messages.filter((message) =>
      isVizeClientCompletion(message, `${origin}/vite${component}`, base),
    ),
    [`[vite] hot updated: ${component}`],
  );
  assert.deepEqual(messages, [`[vite] hot updated: ${component}`, `[vite] hot updated: ${style}`]);
});

test("two actual component completions still violate the unchanged once contract", () => {
  const base = `${origin}/vite/`;
  const initial = `${origin}/vite/src/ui/visitor.vue?vue&vize`;
  const messages = [
    "[vite] hot updated: /src/ui/visitor.vue?vue&vize",
    "[vite] hot updated: /src/ui/visitor.vue?vue&vize via /src/importer.ts",
  ];
  const completed = messages.filter((message) => isVizeClientCompletion(message, initial, base));
  assert.deepEqual(completed, messages);
  assert.throws(() => assert.equal(completed.length, 1), {
    code: "ERR_ASSERTION",
    actual: 2,
    expected: 1,
    operator: "strictEqual",
  });
});

test("completion matching preserves the full initial URL, query flags and origin", () => {
  const base = `${origin}/vite/`;
  const initial = `${origin}/vite/src/ui/visitor.vue?vue&vize`;
  const candidates = [
    ["/src/ui/visitor.vue?vize=&vue=", true],
    ["/src/ui/visitor.vue", false],
    ["/src/ui/visitor.vue?vue", false],
    ["/src/ui/visitor.vue?vize", false],
    ["/src/ui/visitor.vue?vue&vize&vize-ssr", false],
    ["/src/ui/visitor.vue?vue&vize&type=style", false],
    ["/src/ui/visitor.vue?vue&vize&t=123", false],
    ["/other/src/ui/visitor.vue?vue&vize", false],
    ["/vite/src/ui/visitor.vue?vue&vize", false],
    ["/src/ui/visitor.vue.__vize_style_0.css?vue&vize", false],
    ["/src/ui/visitor.vue.ts?vue&vize", false],
    ["https://foreign.invalid/src/ui/visitor.vue?vue&vize", false],
    ["/src/ui/visitor.vue?vue&vize#different", false],
  ] as const;
  assert.deepEqual(
    candidates.map(([path]) =>
      isVizeClientCompletion(`[vite] hot updated: ${path}`, initial, base),
    ),
    candidates.map(([, expected]) => expected),
  );
  assert.equal(isVizeClientCompletion(`[vite] css hot updated: ${initial}`, initial, base), false);
  assert.equal(isVizeClientCompletion(`[vite] hot updated: ${initial}`, "not a URL", base), false);
  const message = "[vite] hot updated: /src/ui/visitor.vue?vue&vize";
  for (const invalidBase of [origin + "/", origin + "/wrong/", "https://foreign.invalid/vite/"])
    assert.equal(isVizeClientCompletion(message, initial, invalidBase), false);
});

test("completion URLs follow the configured Vite base at root, non-root and escaped bases", () => {
  const path = "src/ui/visitor.vue?vue&vize";
  const message = `[vite] hot updated: /${path}`;
  for (const base of ["/", "/vite/", "/preview%20space/"]) {
    const baseUrl = origin + base;
    assert.equal(isVizeClientCompletion(message, origin + base + path, baseUrl), true);
    assert.equal(
      isVizeClientCompletion(message, origin + base + path, origin + "/different/"),
      false,
    );
  }
});
