import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { createRequire } from "node:module";
import path from "node:path";
import { pathToFileURL } from "node:url";

export async function observePrefetchManifest(fixture, root, artifacts) {
  const requireTests = createRequire(path.join(root, "tests/package.json"));
  const { chromium } = requireTests("@playwright/test");
  const manifestPath = path.join(fixture, ".nuxt/dist/server/client.manifest.mjs");
  const { default: manifest } = await import(
    pathToFileURL(manifestPath).href + `?arm=${path.basename(artifacts)}`
  );
  fs.copyFileSync(manifestPath, path.join(artifacts, "client.manifest.mjs"));
  fs.copyFileSync(
    path.join(fixture, ".nuxt/dist/server/client.precomputed.mjs"),
    path.join(artifacts, "client.precomputed.mjs"),
  );
  const publicRoot = path.join(fixture, ".output/public");
  const html = fs.readFileSync(path.join(publicRoot, "index.html"), "utf8");
  fs.writeFileSync(path.join(artifacts, "index.html"), html);
  const serverRequests = [];
  const server = http.createServer((req, res) => {
    serverRequests.push({ url: req.url, method: req.method, headers: req.headers });
    const pathname = new URL(req.url, "http://fixture.local").pathname;
    const filename =
      pathname === "/" || pathname === "/reports"
        ? path.join(publicRoot, "index.html")
        : path.join(publicRoot, pathname);
    try {
      const bytes = fs.readFileSync(filename);
      res.setHeader(
        "Content-Type",
        filename.endsWith(".js")
          ? "text/javascript"
          : filename.endsWith(".css")
            ? "text/css"
            : filename.endsWith(".json")
              ? "application/json"
              : "text/html",
      );
      res.end(bytes);
    } catch {
      res.statusCode = 404;
      res.end();
    }
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  let browser;
  try {
    browser = await chromium.launch();
  } catch (error) {
    fs.writeFileSync(path.join(artifacts, "browser-launch-error.txt"), String(error));
    await new Promise((resolve) => server.close(resolve));
    throw error;
  }
  const errors = [],
    responses = [],
    failed = [];
  let phase = "home";
  try {
    const page = await browser.newPage();
    page.on("pageerror", (error) => errors.push(error.message));
    page.on("response", (response) =>
      responses.push({ phase, url: response.url(), status: response.status() }),
    );
    page.on("requestfailed", (request) =>
      failed.push({ phase, url: request.url(), failure: request.failure() }),
    );
    const origin = `http://127.0.0.1:${server.address().port}`;
    await page.goto(origin, { waitUntil: "networkidle" });
    await page.getByText("Home", { exact: true }).waitFor();
    assert.equal(await page.locator(".report").count(), 0);
    const homeRequests = serverRequests.slice();
    const links = await page.locator("head link").evaluateAll((nodes) =>
      nodes.map((node) => ({
        attributes: Object.fromEntries(
          [...node.attributes].map((attribute) => [attribute.name, attribute.value]),
        ),
        pathname: new URL(node.href).pathname,
      })),
    );
    fs.writeFileSync(path.join(artifacts, "home-dom.html"), await page.content());
    phase = "reports";
    await page.goto(`${origin}/reports`, { waitUntil: "networkidle" });
    await page.getByText("Reports", { exact: true }).waitFor();
    const report = await page.locator(".report").evaluate((node) => ({
      text: node.textContent,
      color: getComputedStyle(node).color,
      attributes: Object.fromEntries(
        [...node.attributes].map((attribute) => [attribute.name, attribute.value]),
      ),
    }));
    assert.equal(report.color, "rgb(255, 0, 0)");
    assert.ok(Object.keys(report.attributes).some((name) => /^data-v-[a-f\d]+$/.test(name)));
    fs.writeFileSync(path.join(artifacts, "reports-dom.html"), await page.content());
    assert.deepEqual(errors, []);
    assert.deepEqual(failed, []);
    assert.deepEqual(
      responses.filter((response) => response.status >= 400),
      [],
    );
    return {
      browser: browser.version(),
      origin,
      manifest,
      html,
      links,
      homeRequests,
      serverRequests,
      responses,
      failed,
      errors,
      report,
    };
  } finally {
    fs.writeFileSync(
      path.join(artifacts, "browser-raw.json"),
      JSON.stringify({ phase, serverRequests, responses, failed, errors }, null, 2) + "\n",
    );
    try {
      await browser.close();
    } finally {
      await new Promise((resolve) => server.close(resolve));
    }
  }
}

export function assertPrefetchManifest(result, broken) {
  const { manifest } = result;
  const entries = Object.entries(manifest).filter(([, chunk]) => chunk.isEntry);
  assert.equal(entries.length, 1, "the complete manifest must have one SPA entry");
  const dynamicImports = entries[0][1].dynamicImports ?? [];
  const suffix = broken ? "?vue&vize" : "";
  for (const name of ["pages/index.vue", "pages/reports.vue"]) {
    assert.equal(manifest[name + suffix]?.src, name + suffix);
    assert.equal(dynamicImports.includes(name + suffix), broken);
  }
  const reportKey = "pages/reports.vue" + suffix;
  const report = manifest[reportKey];
  const css = new Set();
  const seen = new Set();
  function visit(key) {
    if (seen.has(key)) return;
    seen.add(key);
    assert.ok(Object.hasOwn(manifest, key), `missing original manifest dependency ${key}`);
    const chunk = manifest[key];
    for (const file of chunk.css ?? []) css.add(file);
    for (const imported of chunk.imports ?? []) visit(imported);
  }
  visit(reportKey);
  assert.ok(css.size > 0, "original ReportTable scoped CSS must be emitted");
  const reportAssets = [report.file, ...css].map((file) => `/_nuxt/${file}`);
  const prefetch = result.links
    .filter((link) => link.attributes.rel === "prefetch")
    .map((link) => link.pathname);
  for (const asset of reportAssets) {
    assert.equal(prefetch.includes(asset), broken, `home prefetch ownership for ${asset}`);
    assert.equal(
      result.homeRequests.some((request) => request.url.split("?")[0] === asset),
      broken,
      `actual home request ownership for ${asset}`,
    );
    assert.ok(
      result.serverRequests.some((request) => request.url.split("?")[0] === asset),
      `reports route must eventually load ${asset}`,
    );
  }
  return {
    dynamicImports,
    prefetch,
    reportAssets,
    prefetchOwnership: joinPrefetchOwnership(result, dynamicImports),
  };
}

/** Compare graph ownership, retaining hashed output filenames in the raw vector. */
export function joinPrefetchOwnership(result, dynamicImports) {
  const owners = new Map();
  function add(file, owner) {
    const pathname = `/_nuxt/${file}`;
    const paths = owners.get(pathname) ?? new Set();
    paths.add(owner);
    owners.set(pathname, paths);
  }
  function visit(key, owner, ancestors) {
    assert.ok(Object.hasOwn(result.manifest, key), `missing prefetch graph row ${key}`);
    if (ancestors.has(key)) return;
    const chunk = result.manifest[key];
    const seen = new Set([...ancestors, key]);
    add(chunk.file, owner + "/file");
    for (const field of ["css", "assets"]) {
      for (const [index, file] of (chunk[field] ?? []).entries())
        add(file, `${owner}/${field}:${index}`);
    }
    for (const [index, imported] of (chunk.imports ?? []).entries()) {
      const source = result.manifest[imported]?.src;
      visit(imported, `${owner}/imports:${index}:${source ?? "anonymous"}`, seen);
    }
  }
  for (const key of dynamicImports) visit(key, `dynamic:${key}`, new Set());
  return result.links
    .filter((link) => link.attributes.rel === "prefetch")
    .map((link) => {
      assert.ok(
        owners.has(link.pathname),
        `prefetch has no original manifest graph owner ${link.pathname}`,
      );
      const { href: _href, ...attributes } = link.attributes;
      return {
        attributes,
        owners: [...owners.get(link.pathname)].sort((left, right) =>
          left < right ? -1 : Number(left > right),
        ),
      };
    });
}
