import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import path from "node:path";
import {
  DEFAULT_MARKDOWN_EXTENSIONS,
  generateOgImages,
  resolveOgImageOptions,
} from "@ox-content/vite-plugin";
import { chromium } from "playwright";
import { resolvePuppeteerExecutablePath } from "../browser-path.js";
import {
  applyOpenGraphMetadata,
  assertPngDimensions,
  ogHeight,
  ogWidth,
  pageMetadata,
  pageRoute,
  readDocumentMetadata,
} from "../theme/open-graph.mjs";
import { buildOgTemplate } from "./og-template.mjs";
import { CATALOGUE_SOURCES } from "./materialize-content.mjs";

const docsRoot = path.resolve(import.meta.dirname, "..");
// The native staging step preserves authored routes. Enumerate their authority,
// then read metadata from the actual SSG output rather than a stale staging tree.
const content = path.join(docsRoot, "content");
const dist = path.join(docsRoot, "dist");
const require = createRequire(import.meta.url);
const providerRequire = createRequire(require.resolve("@ox-content/vite-plugin"));
const providerPlaywright = providerRequire("playwright");
if (!existsSync(providerPlaywright.chromium.executablePath())) {
  const cli = path.join(path.dirname(providerRequire.resolve("playwright/package.json")), "cli.js");
  const result = spawnSync(process.execPath, [cli, "install", "chromium"], { stdio: "inherit" });
  if (result.status !== 0) throw new Error("Cannot install the OG provider's Chromium");
}
assert(
  existsSync(providerPlaywright.chromium.executablePath()),
  "OG provider Chromium is required",
);

async function markdownFiles(directory, prefix = "") {
  const result = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const relative = path.posix.join(prefix, entry.name);
    if (entry.isDirectory())
      result.push(...(await markdownFiles(path.join(directory, entry.name), relative)));
    else if (
      entry.isFile() &&
      DEFAULT_MARKDOWN_EXTENSIONS.includes(path.extname(entry.name)) &&
      !CATALOGUE_SOURCES.includes(relative)
    )
      result.push(relative);
  }
  return result.sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
}

const { template, assetFingerprint } = await buildOgTemplate(docsRoot);
const pages = [];
const routes = new Set();
const browser = await chromium.launch({
  executablePath: resolvePuppeteerExecutablePath(),
  headless: true,
});
try {
  const parser = await browser.newPage();
  for (const file of await markdownFiles(content)) {
    const route = pageRoute(file);
    assert(!routes.has(route), `${file}: duplicate generated route ${route}`);
    routes.add(route);
    const htmlPath = path.join(dist, route, "index.html");
    const html = await readFile(htmlPath, "utf8");
    const document = await parser.evaluate(readDocumentMetadata, html);
    const metadata = pageMetadata(route, document, assetFingerprint);
    const imagePath = path.join(dist, new URL(metadata.image).pathname);
    await mkdir(path.dirname(imagePath), { recursive: true });
    pages.push({ ...metadata, htmlPath, imagePath, originalHtml: html });
  }
} finally {
  await browser.close();
}
assert(pages.length > 0, "No documentation pages were generated");

const results = await generateOgImages(
  pages.map((page) => ({ props: page.props, outputPath: page.imagePath })),
  resolveOgImageOptions({
    template,
    width: ogWidth,
    height: ogHeight,
    concurrency: 6,
    cache: false,
  }),
  docsRoot,
);
assert.equal(results.length, pages.length, "OG provider must return every page result");
const resultsByPath = new Map(results.map((result) => [result.outputPath, result]));
for (const page of pages) {
  const result = resultsByPath.get(page.imagePath);
  assert(result && !result.error, `${page.route}: ${result?.error ?? "missing image result"}`);
  assertPngDimensions(await readFile(page.imagePath), page.route);
  await writeFile(page.htmlPath, applyOpenGraphMetadata(page.originalHtml, page));
}
await writeFile(
  path.join(dist, "_og", "manifest.json"),
  `${JSON.stringify(
    {
      version: 1,
      width: ogWidth,
      height: ogHeight,
      assetFingerprint,
      sourceSha: spawnSync("git", ["rev-parse", "HEAD"], {
        cwd: docsRoot,
        encoding: "utf8",
      }).stdout.trim(),
      pages: pages.map(
        ({
          htmlPath: _htmlPath,
          imagePath: _imagePath,
          originalHtml: _originalHtml,
          ...metadata
        }) => metadata,
      ),
    },
    null,
    2,
  )}\n`,
);
console.log(`Generated and verified ${pages.length} page-specific OG images with native Vue SSR`);
