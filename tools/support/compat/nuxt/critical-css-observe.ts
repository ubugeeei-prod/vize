import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { createRequire } from "node:module";
import path from "node:path";
import { pathToFileURL } from "node:url";

/** Observe prerendered HTML in a browser with application JavaScript disabled. */
export async function observeCriticalCss(fixture, repoRoot) {
  const requireTests = createRequire(path.join(repoRoot, "tests/package.json"));
  const { chromium } = requireTests("@playwright/test");
  const browser = await chromium.launch();
  const publicRoot = path.join(fixture, ".output/public");
  const requests = [];
  const server = http.createServer((req, res) => {
    requests.push(req.url);
    const filename = path.join(publicRoot, req.url === "/" ? "index.html" : req.url.split("?")[0]);
    try {
      const content = fs.readFileSync(filename);
      res.setHeader("Content-Type", filename.endsWith(".css") ? "text/css" : "text/html");
      res.end(content);
    } catch {
      res.statusCode = 404;
      res.end();
    }
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const context = await browser.newContext({ javaScriptEnabled: false });
  try {
    const page = await context.newPage();
    await page.route("**/*.js", (route) => route.abort());
    await page.goto(`http://127.0.0.1:${server.address().port}/`, { waitUntil: "load" });
    const observed = await page.evaluate(() => {
      const layout = document.querySelector(".layout");
      const title = document.querySelector(".title");
      if (!layout || !title) throw new Error("both authored elements must prerender");
      const l = getComputedStyle(layout),
        t = getComputedStyle(title);
      return {
        text: title.textContent,
        display: l.display,
        gap: l.gap,
        color: t.color,
        scopes: {
          layout: layout.getAttributeNames().filter((n) => n.startsWith("data-v-")),
          title: title.getAttributeNames().filter((n) => n.startsWith("data-v-")),
        },
        styles: [...document.querySelectorAll("head style")].map((e) => e.textContent),
        stylesheetLinks: [...document.querySelectorAll("head link[rel=stylesheet]")].map(
          (e) => e.href,
        ),
        styleLinks: [...document.querySelectorAll("head link[as=style]")].map((e) => e.outerHTML),
        cssRules: [...document.styleSheets].flatMap((sheet) =>
          [...sheet.cssRules].map((rule) => rule.cssText),
        ),
      };
    });
    const { default: styles } = await import(
      pathToFileURL(path.join(fixture, ".output/server/chunks/virtual/styles.mjs")).href
    );
    const styleEntries = {};
    for (const key of Object.keys(styles).sort()) styleEntries[key] = await styles[key]();
    const { default: manifest } = await import(
      pathToFileURL(path.join(fixture, ".nuxt/dist/server/client.manifest.mjs")).href
    );
    return {
      browser: browser.version(),
      javaScriptEnabled: false,
      requests,
      observed,
      styleEntries,
      manifest,
    };
  } finally {
    await context.close();
    await browser.close();
    await new Promise((resolve) => server.close(resolve));
  }
}

export function assertCriticalCss(result, broken) {
  const { observed, styleEntries, manifest } = result;
  assert.equal(result.javaScriptEnabled, false);
  assert.equal(observed.text, "hello");
  assert.deepEqual(observed.stylesheetLinks, []);
  if (broken) {
    assert.deepEqual(styleEntries, {});
    assert.deepEqual(observed.styles, []);
    assert.deepEqual(
      [observed.display, observed.gap, observed.color],
      ["block", "normal", "rgb(0, 0, 0)"],
    );
    assert.equal(observed.styleLinks.length, 4);
    return;
  }
  assert.deepEqual(
    [observed.display, observed.gap, observed.color],
    ["grid", "32px", "rgb(102, 51, 153)"],
  );
  assert.deepEqual(observed.styleLinks, []);
  assert.equal(observed.styles.length, 2);
  const layoutScope = observed.scopes.layout[0];
  const titleScope = observed.scopes.title.find((n) => !n.endsWith("-s"));
  assert.deepEqual(observed.styles, [
    `.layout[${layoutScope}]{gap:2rem;display:grid}`,
    `.title[${titleScope}]{color:#639}`,
  ]);
  assert.deepEqual(styleEntries["layouts/default.vue"], [observed.styles[0]]);
  assert.deepEqual(styleEntries["pages/index.vue"], [observed.styles[1]]);
  assert.equal(Object.keys(styleEntries).length, 4);
  for (const basename of ["error-404.vue", "error-500.vue"]) {
    const keys = Object.keys(styleEntries).filter((key) => key.endsWith(`/components/${basename}`));
    assert.equal(keys.length, 1);
    assert.equal(styleEntries[keys[0]].length, 1);
    assert.equal(styleEntries[keys[0]][0].length > 0, true);
  }
  // Every CSS-bearing component manifest row is cleared after successful inline collection.
  const components = Object.entries(manifest).filter(([key]) =>
    /(?:^|\/)(?:pages\/index|layouts\/default|error-404|error-500)\.vue(?:\?|$)/.test(key),
  );
  assert.equal(components.length, 4);
  for (const [, entry] of components) assert.deepEqual(entry.css ?? [], []);
}
