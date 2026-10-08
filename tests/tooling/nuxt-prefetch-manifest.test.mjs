import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import { joinPrefetchOwnership } from "../../tools/support/compat/nuxt/prefetch-manifest-observe.mjs";

const root = new URL("../../", import.meta.url);
const corpus = JSON.parse(
  fs.readFileSync(
    new URL(
      "tests/_fixtures/differential/compiler/nuxt-prefetch-manifest/runtime.expected.json",
      root,
    ),
  ),
);
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

await test("original Nuxt report and complete authored fixture bytes remain pinned", () => {
  const issue = fs.readFileSync(
    new URL("tests/_fixtures/differential/compiler/nuxt-prefetch-manifest/original-issue.md", root),
    "utf8",
  );
  assert.equal(sha256(issue), corpus.originalIssueSha256);
  for (const input of corpus.fixtures)
    assert.equal(sha256(fs.readFileSync(new URL(input.path, root))), input.sha256);
  const fixture = new URL("tools/support/compat/nuxt/fixtures/nuxt-prefetch-manifest/", root);
  for (const [name, language] of [
    ["nuxt.config.ts", "ts"],
    ["app/pages/reports.vue", "vue"],
  ]) {
    const section = issue.slice(issue.indexOf("`" + name + "`"));
    const block = section.match(new RegExp("```" + language + "\\n([\\s\\S]*?)```"));
    assert.ok(block, `missing original reported ${name}`);
    assert.equal(fs.readFileSync(new URL(name, fixture), "utf8"), block[1]);
  }
  assert.equal(
    fs.readFileSync(new URL("app/app.vue", fixture), "utf8"),
    "<template><NuxtPage /></template>\n",
  );
  assert.equal(
    fs.readFileSync(new URL("app/pages/index.vue", fixture), "utf8"),
    "<template><p>Home</p></template>\n",
  );
  assert.deepEqual(corpus.routes, ["/", "/reports"]);
  assert.equal(
    corpus.reporter.coauthor,
    "Co-authored-by: ubugeeei <71201308+ubugeeei@users.noreply.github.com>",
  );
});

await test("whole ordered prefetch joins retain attributes and every original graph ownership path", () => {
  const result = {
    manifest: {
      "error-404.vue": {
        src: "error-404.vue",
        file: "404.hash.js",
        imports: ["shared.hash.js"],
        css: ["404.hash.css"],
      },
      "error-500.vue": { src: "error-500.vue", file: "500.hash.js", imports: ["shared.hash.js"] },
      "shared.hash.js": { file: "shared.hash.js" },
    },
    links: [
      {
        pathname: "/_nuxt/404.hash.js",
        attributes: { rel: "prefetch", as: "script", href: "/_nuxt/404.hash.js" },
      },
      {
        pathname: "/_nuxt/shared.hash.js",
        attributes: { rel: "prefetch", as: "script", href: "/_nuxt/shared.hash.js" },
      },
      {
        pathname: "/_nuxt/404.hash.css",
        attributes: { rel: "prefetch", as: "style", href: "/_nuxt/404.hash.css" },
      },
      {
        pathname: "/_nuxt/500.hash.js",
        attributes: { rel: "prefetch", as: "script", href: "/_nuxt/500.hash.js" },
      },
    ],
  };
  const roots = ["error-404.vue", "error-500.vue"];
  const expected = [
    { attributes: { rel: "prefetch", as: "script" }, owners: ["dynamic:error-404.vue/file"] },
    {
      attributes: { rel: "prefetch", as: "script" },
      owners: [
        "dynamic:error-404.vue/imports:0:anonymous/file",
        "dynamic:error-500.vue/imports:0:anonymous/file",
      ],
    },
    { attributes: { rel: "prefetch", as: "style" }, owners: ["dynamic:error-404.vue/css:0"] },
    { attributes: { rel: "prefetch", as: "script" }, owners: ["dynamic:error-500.vue/file"] },
  ];
  assert.deepEqual(joinPrefetchOwnership(result, roots), expected);
  for (const altered of [
    { ...result, links: result.links.slice(1) },
    { ...result, links: [...result.links].reverse() },
    {
      ...result,
      links: result.links.map((row) => ({
        ...row,
        attributes: { ...row.attributes, crossorigin: "" },
      })),
    },
    {
      ...result,
      manifest: {
        ...result.manifest,
        "error-500.vue": { ...result.manifest["error-500.vue"], imports: [] },
      },
    },
  ])
    assert.notDeepEqual(joinPrefetchOwnership(altered, roots), expected);
  assert.throws(() =>
    joinPrefetchOwnership(
      { ...result, links: [{ pathname: "/_nuxt/foreign.js", attributes: { rel: "prefetch" } }] },
      roots,
    ),
  );
  assert.throws(() =>
    joinPrefetchOwnership(
      {
        ...result,
        manifest: {
          ...result.manifest,
          "error-404.vue": { ...result.manifest["error-404.vue"], imports: ["missing.js"] },
        },
      },
      roots,
    ),
  );
});
