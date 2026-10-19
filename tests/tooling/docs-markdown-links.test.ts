import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { test } from "node:test";

const root = resolve(import.meta.dirname, "../..");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
const { Window } = createRequire(new URL("../package.json", import.meta.url))("happy-dom");
const supportUrl = "https://github.com/ubugeeei-prod/vize/blob/main/SUPPORT.md";
const repositoryDocumentUrl =
  "https://github.com/ubugeeei-prod/vize/blob/main/CONTRIBUTING.md#pull-requests";

void test("each authored testing guide retains its external Support destination in native SSG", () => {
  for (const locale of ["", "ja/", "zh-CN/", "pt-BR/", "fr/"]) {
    const sourcePath = resolve(root, `docs/content/${locale}guide/testing.md`);
    const source = readFileSync(sourcePath, "utf8");
    assert.ok(source.includes(`](${supportUrl})`), "the original authored URL is exercised");
    const rendered = native.transform(source, { convertMdLinks: true, baseUrl: "/", sourcePath });
    assert.deepEqual(rendered.errors, [], sourcePath);
    const document = new Window().document;
    document.body.innerHTML = rendered.html;
    const link = document.querySelector(`a[href="${supportUrl}"]`);
    assert.ok(link, `${locale || "en"}: Support remains an external Markdown document`);
    assert.equal(link.getAttribute("target"), "_blank");
    assert.equal(link.getAttribute("rel"), "noopener noreferrer");
    assert.doesNotMatch(rendered.html, /href="[^"\s]*\/https?:\/\//);
  }
});

void test("native SSG distinguishes external Markdown URLs from generated local routes", () => {
  const cases = [
    [repositoryDocumentUrl, repositoryDocumentUrl],
    [
      "https://example.com/docs/README.md?plain=1#support",
      "https://example.com/docs/README.md?plain=1#support",
    ],
    ["http://example.com/docs/README.md", "http://example.com/docs/README.md"],
    ["//example.com/docs/README.md", "//example.com/docs/README.md"],
    ["git+ssh://example.com/docs/README.md", "git+ssh://example.com/docs/README.md"],
    ["./testing.md?plain=1#support", "../testing/index.html?plain=1#support"],
    ["../getting-started.md#install", "../../getting-started/index.html#install"],
    ["/guide/configuration.md", "/docs/guide/configuration/index.html"],
    ["./index.md", "./index.html"],
    ["#support", "#support"],
    ["./image.svg", "./image.svg"],
  ];
  for (const [destination, expected] of cases) {
    for (const source of [`[link](${destination})`, `<a href="${destination}">link</a>`]) {
      const rendered = native.transform(source, {
        convertMdLinks: true,
        baseUrl: "/docs/",
        sourcePath: "/site/content/ja/guide/testing.md",
      });
      assert.deepEqual(rendered.errors, [], source);
      const document = new Window().document;
      document.body.innerHTML = rendered.html;
      assert.equal(document.querySelector("a").getAttribute("href"), expected, source);
    }
  }
});
