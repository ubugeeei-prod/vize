import assert from "node:assert/strict";
import { test } from "node:test";
import { normalizeStyleVirtualId } from "./style-request.ts";

// The fresh native source fixture returned HTTP 404 for this actual request.
// Vite prepends `direct` to browser stylesheet requests before plugin loading.
const actual =
  "/VFHeading.vue.__vize_style_0.css?direct&vue=&type=style&index=0&scoped=data-v-ea49ded1&lang=css&vize-file=%2Fhome%2Frunner%2F_work%2Fvize%2Fvize%2Ftarget%2Fvuefes-ssr-style-dependencies%2Fproject-1Rp9Ub%2FVFHeading%2Evue";
const source =
  "/home/runner/_work/vize/vize/target/vuefes-ssr-style-dependencies/project-1Rp9Ub/VFHeading.vue";

void test("real browser direct CSS requests retain the authored source and entire query", () => {
  for (const id of [actual, `\0${actual}`, actual.replace("?direct&vue", "?vue")]) {
    assert.equal(normalizeStyleVirtualId(id), `${source}?${id.slice(id.indexOf("?") + 1)}`);
  }
});

void test("lookalike query names cannot opt into Vue style normalization", () => {
  for (const id of [
    actual.replace("&vue=", "&vueish="),
    actual.replace("&vue=", "&notvue="),
    actual.replace("&vue=", "&x=vue&"),
    "/VFHeading.vue.__vize_style_0.css",
  ])
    assert.equal(normalizeStyleVirtualId(id), id);
});
