import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import { createHash } from "node:crypto";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { pathToFileURL } from "node:url";
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
} from "../theme/open-graph.ts";
import { buildOgTemplate } from "./og-template.ts";
import { CATALOGUE_SOURCES } from "./materialize-content.ts";

type GeneratedPage = ReturnType<typeof pageMetadata> & {
  htmlPath: string;
  imagePath: string;
  originalHtml: string;
};

function providerChromiumPath(provider: unknown): string {
  assert(typeof provider === "object" && provider !== null && "chromium" in provider);
  const { chromium: providerChromium } = provider;
  assert(
    typeof providerChromium === "object" &&
      providerChromium !== null &&
      "executablePath" in providerChromium &&
      typeof providerChromium.executablePath === "function",
    "OG provider must expose its Chromium executable path",
  );
  const executablePath: unknown = providerChromium.executablePath();
  assert(typeof executablePath === "string" && executablePath.length > 0);
  return executablePath;
}

const docsRoot = path.resolve(import.meta.dirname, "..");
// The native staging step preserves authored routes. Enumerate their authority,
// then read metadata from the actual SSG output rather than a stale staging tree.
const content = path.join(docsRoot, "content");
const dist = path.join(docsRoot, "dist");
const require = createRequire(import.meta.url);
const providerRequire = createRequire(require.resolve("@ox-content/vite-plugin"));
const providerModule: unknown = await import(
  pathToFileURL(providerRequire.resolve("playwright")).href
);
assert(
  typeof providerModule === "object" && providerModule !== null && "default" in providerModule,
  "OG provider Playwright module must expose its CommonJS default export",
);
const providerPlaywright = providerModule.default;
if (!existsSync(providerChromiumPath(providerPlaywright))) {
  const cli = path.join(path.dirname(providerRequire.resolve("playwright/package.json")), "cli.js");
  const result = spawnSync(process.execPath, [cli, "install", "chromium"], { stdio: "inherit" });
  if (result.status !== 0) throw new Error("Cannot install the OG provider's Chromium");
}
assert(existsSync(providerChromiumPath(providerPlaywright)), "OG provider Chromium is required");

async function markdownFiles(directory: string, prefix = ""): Promise<string[]> {
  const result: string[] = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const relative = path.posix.join(prefix, entry.name);
    if (entry.isDirectory())
      result.push(...(await markdownFiles(path.join(directory, entry.name), relative)));
    else if (
      entry.isFile() &&
      DEFAULT_MARKDOWN_EXTENSIONS.some((extension) => extension === path.extname(entry.name)) &&
      !CATALOGUE_SOURCES.some((source: string) => source === relative)
    )
      result.push(relative);
  }
  return result.sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
}

const { template, assetFingerprint } = await buildOgTemplate(docsRoot);
const pages: GeneratedPage[] = [];
const routes = new Set<string>();
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
const imageHashes = new Map<string, string>();
for (const page of pages) {
  const result = resultsByPath.get(page.imagePath);
  assert(result && !result.error, `${page.route}: ${result?.error ?? "missing image result"}`);
  const png = await readFile(page.imagePath);
  assertPngDimensions(png, page.route);
  imageHashes.set(page.imagePath, createHash("sha256").update(png).digest("hex"));
  await writeFile(page.htmlPath, applyOpenGraphMetadata(page.originalHtml, page));
}
const source = spawnSync("git", ["rev-parse", "HEAD"], {
  cwd: docsRoot,
  encoding: "utf8",
});
assert.equal(source.status, 0, "OG manifest must identify the actual Git source");
const sourceSha = source.stdout.trim();
assert(/^[a-f\d]{40}$/u.test(sourceSha), "OG manifest must contain a complete source SHA");
await writeFile(
  path.join(dist, "_og", "manifest.json"),
  `${JSON.stringify(
    {
      version: 1,
      width: ogWidth,
      height: ogHeight,
      assetFingerprint,
      sourceSha,
      pages: pages.map(
        ({ htmlPath: _htmlPath, imagePath, originalHtml: _originalHtml, ...metadata }) => {
          const imageSha256 = imageHashes.get(imagePath);
          assert(imageSha256, `${metadata.route}: successful image must retain its byte custody`);
          return { ...metadata, imageSha256 };
        },
      ),
    },
    null,
    2,
  )}\n`,
);
console.log(`Generated and verified ${pages.length} page-specific OG images with native Vue SSR`);
