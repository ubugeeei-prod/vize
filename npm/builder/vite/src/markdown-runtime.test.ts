// A real Vite build receives the JavaScript module emitted by Ox Content and
// then lets Vize compile it as a Vue component. Keep this at the bundler
// boundary: unit tests for the measured producer contract live beside the
// adapter, while this proves plugin ordering and DOM/SSR/Vapor selection.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { build, type Plugin, type Rollup } from "vite";

import { vize } from "./index.ts";

const require = createRequire(import.meta.url);
const vueRuntime = require.resolve("vue");
const serverRenderer = require.resolve("vue/server-renderer");
const oxContentModule = fs.readFileSync(
  path.resolve(import.meta.dirname, "test/fixtures/ox-content/Page.module.txt"),
  "utf8",
);
const authoredMarkdown = fs.readFileSync(
  path.resolve(import.meta.dirname, "test/fixtures/ox-content/Page.md"),
  "utf8",
);

interface BuiltMarkdownModule {
  code: string;
  file: string;
  root: string;
}

async function buildMarkdownModule(ssr: boolean, vapor: boolean): Promise<BuiltMarkdownModule> {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vite-markdown-"));
  const page = path.join(root, "Page.md");
  const entry = path.join(root, "entry.js");
  fs.writeFileSync(page, authoredMarkdown);
  fs.writeFileSync(entry, 'export { default, frontmatter, toc, html } from "./Page.md";\n');

  const measuredOxContentProducer: Plugin = {
    name: "test:measured-ox-content-producer",
    transform(_code, id) {
      return id === page ? { code: oxContentModule, map: null } : null;
    },
  };
  const result = (await build({
    root,
    configFile: false,
    logLevel: "silent",
    plugins: [measuredOxContentProducer, vize({ configMode: false, vapor })],
    build: {
      write: false,
      minify: false,
      ...(ssr ? { ssr: entry } : { lib: { entry, formats: ["es"], fileName: "page" } }),
      rollupOptions: { external: [/^(?:@vue\/|vue(?:\/|$))/] },
    },
  })) as Rollup.RollupOutput | Rollup.RollupOutput[];
  const chunk = (Array.isArray(result) ? result[0] : result).output.find(
    (item): item is Rollup.OutputChunk => item.type === "chunk" && item.isEntry,
  );
  assert.ok(chunk, "Markdown build should emit an entry chunk");
  const file = path.join(root, "page.mjs");
  const code = chunk.code
    .replace(
      /from ["'](?:@vue\/server-renderer|vue\/server-renderer)["']/g,
      `from ${JSON.stringify(pathToFileURL(serverRenderer).href)}`,
    )
    .replace(/from ["']vue["']/g, `from ${JSON.stringify(pathToFileURL(vueRuntime).href)}`);
  fs.writeFileSync(file, code);
  return { code, file, root };
}

async function assertRenderedMarkdown(ssr: boolean): Promise<void> {
  const built = await buildMarkdownModule(ssr, false);
  try {
    const module = (await import(pathToFileURL(built.file).href)) as {
      default: object;
      frontmatter: { title: string; published: boolean };
      html: string;
      toc: Array<{ text: string }>;
    };
    const { createSSRApp } = (await import(pathToFileURL(vueRuntime).href)) as typeof import("vue");
    const { renderToString } = (await import(
      pathToFileURL(serverRenderer).href
    )) as typeof import("vue/server-renderer");
    const html = await renderToString(createSSRApp(module.default));

    assert.deepEqual(module.frontmatter, { title: "Markdown fixture", published: true });
    assert.equal(module.toc[0]?.text, "Hello {{ frontmatter.title }}");
    assert.match(module.html, /<table>/);
    assert.match(html, /<h1 id="hello-frontmatter-title">Hello Markdown fixture<\/h1>/);
    assert.match(html, /<table>/);
    assert.match(html, /\{\{ mustStayInert \}\}/);
    assert.doesNotMatch(built.code, /_ctx\.mustStayInert/);
  } finally {
    fs.rmSync(built.root, { recursive: true, force: true });
  }
}

for (const ssr of [false, true]) {
  void test(`Vite ${ssr ? "SSR" : "DOM"} renders the measured Ox Content module`, async () => {
    await assertRenderedMarkdown(ssr);
  });
}

void test("Vite selects Vapor for the measured Ox Content module", async () => {
  const built = await buildMarkdownModule(false, true);
  try {
    assert.match(built.code, /__vapor\s*(?::|=)\s*true/);
    assert.match(built.code, /frontmatter\.title/);
    assert.doesNotMatch(built.code, /_ctx\.mustStayInert/);
  } finally {
    fs.rmSync(built.root, { recursive: true, force: true });
  }
});
