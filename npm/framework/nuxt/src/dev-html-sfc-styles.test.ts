import assert from "node:assert/strict";
import { test } from "node:test";
import { sanitizeNuxtDevStylesheetLinks } from "./dev-html.ts";
import { createStyleVirtualId } from "../../../builder/vite/src/plugin/style-request.ts";

// Whole literal-H HTTP 200 component response retained this exact style import.
const actual =
  "/_nuxt/components/heading/VFHeading.vue.__vize_style_0.css?vue=&type=style&index=0&scoped=data-v-98cff51b&lang=css&vize-file=%2Fhome%2Frunner%2F_work%2Fvize%2Fvize%2Ftests%2F_fixtures%2F_projects%2F_git-worktrees%2Fci-app-e2e-full-vrt-vuefes%2Fvrt-dev-candidate%2Fvuefes-2025%2Fapp%2Fcomponents%2Fheading%2FVFHeading%2Evue";
const link = (href: string) =>
  `<link rel="stylesheet" href="${href.replaceAll("&", "&amp;")}" crossorigin>`;

void test("the actual nested SFC style import survives in complete HTML and original order", () => {
  const html = `<!doctype html><html><head>${link("/_nuxt/assets/styles/main.css")}${link(actual)}</head><body><h2 data-v-98cff51b>complete heading</h2></body></html>`;
  assert.equal(sanitizeNuxtDevStylesheetLinks(html), html);
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(html.replace(link(actual), link(actual) + link(actual))),
    html,
  );
});

void test("producer-generated nested and encoded SFC CSS entries honor custom asset bases", () => {
  for (const filename of ["components/heading/VFHeading.vue", "components/日本語 Heading.vue"]) {
    const id = createStyleVirtualId(`${filename}?vue=&type=style&index=2&lang=css`);
    const tag = link(`/docs/_nuxt/${id}`);
    assert.equal(sanitizeNuxtDevStylesheetLinks(tag, "/docs/_nuxt/"), tag);
  }
});

void test("generated-style names preserve the existing invalid-path rejection", () => {
  for (const href of [
    "/_nuxt/components/%2e%2e/Heading.vue.__vize_style_0.css",
    "/_nuxt/components/%00/Heading.vue.__vize_style_0.css",
    "/_nuxt/components%5Cheading%5CHeading.vue.__vize_style_0.css",
    "/_nuxt/components/Heading.vue.__vize_style_-1.css",
    "/_nuxt/components/Heading.vue.__vize_style_0.css.js",
    "/_nuxt/pkg/style.css",
  ])
    assert.equal(sanitizeNuxtDevStylesheetLinks(link(href)), "");
});
