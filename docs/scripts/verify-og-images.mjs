import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { parseArgs } from "node:util";
import { chromium } from "playwright";
import { DEFAULT_MARKDOWN_EXTENSIONS } from "@ox-content/vite-plugin";
import { resolvePuppeteerExecutablePath } from "../browser-path.js";
import { CATALOGUE_SOURCES } from "./materialize-content.mjs";
import {
  assertPngDimensions,
  docsSiteUrl,
  ogHeight,
  ogWidth,
  pageRoute,
} from "../theme/open-graph.mjs";

const { values } = parseArgs({
  options: {
    dir: { type: "string", default: "docs/dist" },
    output: { type: "string", default: "docs-render-evidence/og-images" },
    site: { type: "string" },
    "expected-sha": { type: "string" },
  },
});
const dist = path.resolve(values.dir);
const output = path.resolve(values.output);
const docsRoot = path.resolve(import.meta.dirname, "..");
if (values.site)
  assert(values["expected-sha"], "Live verification requires the expected deployed source SHA");
const expectedSha =
  values["expected-sha"] ??
  process.env.GITHUB_SHA ??
  spawnSync("git", ["rev-parse", "HEAD"], { cwd: docsRoot, encoding: "utf8" }).stdout.trim();
async function readAsset(relative, text = false) {
  if (!values.site) return readFile(path.join(dist, relative), text ? "utf8" : undefined);
  const response = await fetch(new URL(relative, values.site));
  assert.equal(response.status, 200, relative);
  assert.match(
    response.headers.get("content-type") ?? "",
    text ? /(?:text\/html|application\/json)/u : /image\/png/u,
    relative,
  );
  return text ? response.text() : Buffer.from(await response.arrayBuffer());
}
const manifest = JSON.parse(await readAsset("_og/manifest.json", true));
assert.equal(manifest.width, ogWidth);
assert.equal(manifest.height, ogHeight);
assert.match(manifest.sourceSha, /^[0-9a-f]{40}$/u, "Manifest must identify its actual source");
assert.equal(
  manifest.sourceSha,
  expectedSha,
  "OG manifest differs from the expected source revision",
);
const contentDir = path.join(docsRoot, "content");
const expectedRoutes = (await readdir(contentDir, { recursive: true, withFileTypes: true }))
  .filter((file) => file.isFile() && DEFAULT_MARKDOWN_EXTENSIONS.includes(path.extname(file.name)))
  .map((file) =>
    path.relative(contentDir, path.join(file.parentPath, file.name)).replaceAll("\\", "/"),
  )
  .filter((file) => !CATALOGUE_SOURCES.includes(file))
  .map(pageRoute)
  .sort();
assert.deepEqual(
  manifest.pages.map((page) => page.route).sort(),
  expectedRoutes,
  "OG routes must cover the independent authored/generated source tree",
);
assert(manifest.pages.length > 0, "Empty OG manifest");
assert.equal(
  new Set(manifest.pages.map((page) => page.route)).size,
  manifest.pages.length,
  "Duplicate routes",
);
assert.equal(
  new Set(manifest.pages.map((page) => page.image)).size,
  manifest.pages.length,
  "Duplicate image identities",
);
const representativeRoutes = new Set([
  "/",
  "/ja/",
  "/zh-CN/",
  "/guide/cli/",
  "/getting-started/",
  "/ja/getting-started/",
  "/rules/all/",
  "/ja/rules/all/",
  "/rules/reference/vue-component-name-in-template-casing/",
  "/ja/rules/reference/vue-component-name-in-template-casing/",
]);
const longestTitle = [...manifest.pages].sort(
  (a, b) => b.props.title.length - a.props.title.length,
)[0];
representativeRoutes.add(longestTitle.route);
await mkdir(output, { recursive: true });
const browser = await chromium.launch({
  executablePath: resolvePuppeteerExecutablePath(),
  headless: true,
});
const checked = [];
const representativeHashes = new Map();
try {
  const parser = await browser.newPage();
  for (const entry of manifest.pages) {
    const html = await readAsset(`${entry.route.slice(1)}index.html`, true);
    const imagePath = new URL(entry.image).pathname.slice(1);
    assert.equal(new URL(entry.image).origin, docsSiteUrl, entry.route);
    const png = await readAsset(imagePath);
    assertPngDimensions(png, entry.route);
    const actual = await parser.evaluate(
      async ({ html, image }) => {
        const document = new DOMParser().parseFromString(html, "text/html");
        const tags = [...document.head.querySelectorAll("meta[property], meta[name]")].map(
          (tag) => ({
            key: tag.getAttribute("property") || tag.getAttribute("name"),
            attribute: tag.hasAttribute("property") ? "property" : "name",
            content: tag.getAttribute("content"),
          }),
        );
        const bitmap = await createImageBitmap(
          await (await fetch(`data:image/png;base64,${image}`)).blob(),
        );
        const size = { width: bitmap.width, height: bitmap.height };
        bitmap.close();
        return { language: document.documentElement.lang, tags, size };
      },
      { html, image: png.toString("base64") },
    );
    assert.equal(actual.language, entry.props.locale, entry.route);
    assert.deepEqual(actual.size, { width: ogWidth, height: ogHeight }, entry.route);
    const expected = {
      "og:title": entry.title,
      "og:description": entry.description,
      "og:url": entry.url,
      "og:locale": entry.locale,
      "og:site_name": "Vize",
      "og:type": entry.type,
      "og:image": entry.image,
      "og:image:type": "image/png",
      "og:image:width": String(ogWidth),
      "og:image:height": String(ogHeight),
      "og:image:alt": `${entry.props.title} · ${entry.props.category} · Vize`,
      description: entry.description,
      "twitter:card": "summary_large_image",
      "twitter:title": entry.title,
      "twitter:description": entry.description,
      "twitter:image": entry.image,
      "twitter:image:alt": `${entry.props.title} · ${entry.props.category} · Vize`,
    };
    for (const [key, content] of Object.entries(expected)) {
      const tags = actual.tags.filter((tag) => tag.key === key);
      assert.equal(tags.length, 1, `${entry.route}: ${key} count`);
      assert.equal(
        tags[0].attribute,
        key.startsWith("og:") ? "property" : "name",
        `${entry.route}: ${key} attribute`,
      );
      assert.equal(tags[0].content, content, `${entry.route}: ${key}`);
      assert(content, `${entry.route}: empty ${key}`);
    }
    if (representativeRoutes.has(entry.route)) {
      representativeHashes.set(entry.route, createHash("sha256").update(png).digest("hex"));
      await writeFile(
        path.join(output, `${entry.props.locale}-${entry.route.replaceAll("/", "_")}.png`),
        png,
      );
    }
    checked.push({
      route: entry.route,
      title: entry.title,
      descriptionOrigin: entry.descriptionOrigin,
      language: actual.language,
      image: entry.image,
      ...actual.size,
    });
  }
} finally {
  await browser.close();
}
for (const route of representativeRoutes)
  assert(
    checked.some((entry) => entry.route === route),
    `Missing representative ${route}`,
  );
// These authored titles intentionally differ from the first visible heading.
// Check the independent literal contract, rather than certifying the manifest.
for (const [route, title] of [
  ["/guide/cli/", "CLI"],
  ["/zh-CN/", "维泽"],
]) {
  assert.equal(manifest.pages.find((entry) => entry.route === route)?.props.title, title, route);
}
assert.equal(
  new Set(representativeHashes.values()).size,
  representativeHashes.size,
  "Representative pages must render distinct image bytes; a reused homepage image is invalid",
);
for (const route of ["/getting-started/", "/ja/getting-started/"]) {
  assert.equal(checked.find((entry) => entry.route === route).descriptionOrigin, "authored", route);
}
for (const route of [
  "/rules/reference/vue-component-name-in-template-casing/",
  "/ja/rules/reference/vue-component-name-in-template-casing/",
]) {
  assert.equal(checked.find((entry) => entry.route === route).descriptionOrigin, "content", route);
}
if (values.site) {
  assert.equal(
    JSON.parse(await readAsset("_og/manifest.json", true)).sourceSha,
    expectedSha,
    "Public deployment changed during whole-site verification",
  );
}
await writeFile(
  path.join(output, "metadata.json"),
  `${JSON.stringify({ source: values.site ?? dist, sourceSha: manifest.sourceSha, assetFingerprint: manifest.assetFingerprint, checked }, null, 2)}\n`,
);
console.log(
  `Verified ${checked.length} actual page metadata frames and decoded 1200×630 PNGs; retained ${representativeRoutes.size} representative images`,
);
