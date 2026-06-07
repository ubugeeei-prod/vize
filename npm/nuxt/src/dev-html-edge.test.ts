import assert from "node:assert/strict";
import test from "node:test";

import { sanitizeNuxtDevStylesheetLinks } from "./dev-html.ts";

void test("HTML without any <link> tags is returned unchanged", () => {
  const html = `<!DOCTYPE html><html><head><title>x</title></head><body>hi</body></html>`;
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(html),
    html,
    "markup without stylesheet links should pass through untouched",
  );
});

void test("non-stylesheet <link> tags are never processed", () => {
  // Only rel="stylesheet" links are matched by the sanitizer, so even an
  // obviously unsafe href on a preload/icon link is left exactly as-is.
  const preload = `<link rel="preload" href="/_nuxt/../etc.js">`;
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(preload),
    preload,
    "rel=preload links should be ignored entirely",
  );

  const icon = `<link rel="icon" href="/_nuxt/../favicon.ico">`;
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(icon),
    icon,
    "rel=icon links should be ignored entirely",
  );
});

void test("single and double quotes on rel/href are both handled", () => {
  const single = `<link rel='stylesheet' href='/_nuxt/assets/main.css'>`;
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(single),
    single,
    "single-quoted valid stylesheet links should be kept verbatim",
  );

  const double = `<link rel="stylesheet" href="/_nuxt/assets/main.css">`;
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(double),
    double,
    "double-quoted valid stylesheet links should be kept verbatim",
  );
});

void test("allow-listed dev stylesheet paths are kept", () => {
  const cases = [
    `<link rel="stylesheet" href="/_nuxt/assets/main.css">`,
    `<link rel="stylesheet" href="/_nuxt/@fs/Users/me/project/node_modules/pkg/style.css">`,
    `<link rel="stylesheet" href="/_nuxt/@id/foo.css">`,
    `<link rel="stylesheet" href="/_nuxt/virtual:uno.css">`,
    `<link rel="stylesheet" href="/_nuxt/__uno.css">`,
    `<link rel="stylesheet" href="/_nuxt/style.css">`,
  ];
  for (const tag of cases) {
    assert.equal(
      sanitizeNuxtDevStylesheetLinks(tag),
      tag,
      `allow-listed href should be preserved: ${tag}`,
    );
  }
});

void test("non-allow-listed sub-path under the assets dir is stripped", () => {
  // A nested, non-css-named path under /_nuxt/ matches none of the allow-list
  // prefixes (@fs/, @id/, assets/, virtual:) nor the single-segment *.css rule.
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(`<link rel="stylesheet" href="/_nuxt/pkg/style.css">`),
    "",
    "an unrecognized nested path under the assets dir should be removed",
  );
});

void test("duplicate identical hrefs are de-duplicated", () => {
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(
      `<link rel="stylesheet" href="/_nuxt/assets/main.css"><link rel="stylesheet" href="/_nuxt/assets/main.css">`,
    ),
    `<link rel="stylesheet" href="/_nuxt/assets/main.css">`,
    "a repeated stylesheet href should only survive once",
  );

  // Dedupe is keyed on the raw href and applies even to external links that
  // are otherwise always kept.
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(
      `<link rel="stylesheet" href="https://cdn.example.com/x.css"><link rel="stylesheet" href="https://cdn.example.com/x.css">`,
    ),
    `<link rel="stylesheet" href="https://cdn.example.com/x.css">`,
    "duplicate external stylesheet hrefs are also collapsed",
  );
});

void test("'..' traversal segments under the assets dir are stripped", () => {
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(`<link rel="stylesheet" href="/_nuxt/assets/../secret.css">`),
    "",
    "a '..' segment in a sub-path should be rejected",
  );
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(`<link rel="stylesheet" href="/_nuxt/../etc.css">`),
    "",
    "a '..' segment directly after the assets dir should be rejected",
  );
});

void test("URL-encoded traversal and encoded null bytes are decoded then rejected", () => {
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(
      `<link rel="stylesheet" href="/_nuxt/assets/%2e%2e/server.css">`,
    ),
    "",
    "percent-encoded '..' should be decoded and rejected as traversal",
  );
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(`<link rel="stylesheet" href="/_nuxt/assets/%00secret.css">`),
    "",
    "a decoded null byte should be rejected",
  );
});

void test("a fragment-only suffix is ignored when validating the path", () => {
  // The path part is validated after stripping ?query and #fragment, so a
  // valid css href with a fragment is kept and the fragment stays in the tag.
  const tag = `<link rel="stylesheet" href="/_nuxt/assets/main.css#x">`;
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(tag),
    tag,
    "a fragment should not change keep/strip outcome and should be preserved",
  );
});

void test("custom buildAssetsDir is filtered while paths under other dirs pass through", () => {
  // Only hrefs under the (normalized) custom dir are validated. A non-allow-listed
  // sub-path under /_app/ is stripped; an allow-listed one is kept.
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(
      `<link rel="stylesheet" href="/_app/pkg/style.css"><link rel="stylesheet" href="/_app/assets/main.css">`,
      "/_app/",
    ),
    `<link rel="stylesheet" href="/_app/assets/main.css">`,
    "custom buildAssetsDir should gate sub-paths under that dir",
  );

  // SURPRISING: an href under the *default* /_nuxt/ dir is NOT under the custom
  // /_app/ prefix, so it is treated as "external" and kept untouched rather than
  // being stripped. The sanitizer only validates hrefs under the configured dir.
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(
      `<link rel="stylesheet" href="/_nuxt/assets/main.css">`,
      "/_app/",
    ),
    `<link rel="stylesheet" href="/_nuxt/assets/main.css">`,
    "an href outside the configured assets dir is left as-is, even if it looks like a Nuxt path",
  );
});

void test("empty href is kept (treated as not under the assets dir)", () => {
  // SURPRISING: href="" does not start with the normalized assets dir, so
  // shouldKeepHref returns true and the empty stylesheet link survives.
  const tag = `<link rel="stylesheet" href="">`;
  assert.equal(
    sanitizeNuxtDevStylesheetLinks(tag),
    tag,
    "an empty href is not stripped by the current implementation",
  );
});
