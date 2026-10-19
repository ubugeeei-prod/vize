import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import path from "node:path";
import { asRecord, at } from "../../json-value.ts";
import { importDefault, loadChromium } from "./host-types.ts";
import type { BrowserGlobals } from "./host-types.ts";

export interface CriticalCssObservation {
  text: string | null;
  display: string;
  gap: string;
  color: string;
  scopes: { layout: string[]; title: string[] };
  styles: (string | null)[];
  stylesheetLinks: (string | undefined)[];
  styleLinks: string[];
  cssRules: string[];
}
export interface CriticalCssResult {
  browser: string;
  javaScriptEnabled: boolean;
  requests: (string | undefined)[];
  observed: CriticalCssObservation;
  styleEntries: Record<string, unknown>;
  manifest: unknown;
}

/** Observe prerendered HTML in a browser with application JavaScript disabled. */
export async function observeCriticalCss(
  fixture: string,
  repoRoot: string,
): Promise<CriticalCssResult> {
  const browser = await loadChromium(repoRoot).launch();
  const publicRoot = path.join(fixture, ".output/public");
  const requests: (string | undefined)[] = [];
  const server = http.createServer((req, res) => {
    requests.push(req.url);
    const url = req.url ?? "/";
    const filename = path.join(publicRoot, url === "/" ? "index.html" : (url.split("?")[0] ?? ""));
    try {
      const content = fs.readFileSync(filename);
      res.setHeader("Content-Type", filename.endsWith(".css") ? "text/css" : "text/html");
      res.end(content);
    } catch {
      res.statusCode = 404;
      res.end();
    }
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  assert.ok(address && typeof address === "object");
  const context = await browser.newContext({ javaScriptEnabled: false });
  try {
    const page = await context.newPage();
    await page.route("**/*.js", (route) => route.abort());
    await page.goto(`http://127.0.0.1:${address.port}/`, { waitUntil: "load" });
    const observed = await page.evaluate((): CriticalCssObservation => {
      const { document, getComputedStyle } = globalThis as unknown as BrowserGlobals;
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
    const styles = asRecord(
      await importDefault(path.join(fixture, ".output/server/chunks/virtual/styles.mjs")),
      "Nitro styles",
    );
    const styleEntries: Record<string, unknown> = {};
    for (const key of Object.keys(styles).sort()) {
      const load = styles[key];
      assert.equal(typeof load, "function", `style entry ${key}`);
      styleEntries[key] = await (load as () => Promise<unknown>)();
    }
    const manifest = await importDefault(
      path.join(fixture, ".nuxt/dist/server/client.manifest.mjs"),
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

function styleList(entries: Record<string, unknown>, key: string | undefined): unknown[] {
  const value = key === undefined ? undefined : entries[key];
  assert.ok(Array.isArray(value), `style entry ${key ?? "(missing)"}`);
  return value;
}

export function assertCriticalCss(result: CriticalCssResult, broken: boolean): void {
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
    const entry = styleList(styleEntries, keys[0]);
    assert.equal(entry.length, 1);
    assert.equal(String(entry[0]).length > 0, true);
  }
  // Every CSS-bearing component manifest row is cleared after successful inline collection.
  const components = Object.entries(asRecord(manifest, "client manifest")).filter(([key]) =>
    /(?:^|\/)(?:pages\/index|layouts\/default|error-404|error-500)\.vue(?:\?|$)/.test(key),
  );
  assert.equal(components.length, 4);
  for (const [, entry] of components) assert.deepEqual(at(entry, "css") ?? [], []);
}
