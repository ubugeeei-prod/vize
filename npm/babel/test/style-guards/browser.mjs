import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import path from "node:path";
import { createRequire } from "node:module";
import { json } from "../runtime/environment.mjs";
import { wholeError } from "../hosts/contracts.mjs";

export async function browserCases(input, directory, cases) {
  const requireRuntime = createRequire(path.join(input.runtime.root, "package.json"));
  const { chromium } = requireRuntime("playwright");
  const files = new Map([
    ["/runtime.js", input.runtime.browserEntry],
    ["/page.mjs", path.join(import.meta.dirname, "page.mjs")],
  ]);
  for (const name of fs.readdirSync(directory))
    if (name.endsWith(".mjs")) files.set("/generated/" + name, path.join(directory, name));
  let selected;
  const requests = [],
    rows = [];
  const server = http.createServer((request, response) => {
    const pathname = new URL(request.url, "http://127.0.0.1").pathname;
    requests.push({ pathname, method: request.method });
    if (pathname === "/favicon.ico") {
      response.writeHead(204);
      response.end();
      return;
    }
    if (pathname === "/") {
      response.writeHead(200, { "content-type": "text/html;charset=utf-8" });
      response.end(
        '<!doctype html><meta charset="utf-8"><script type="importmap">{"imports":{"vue":"/runtime.js"}}</script><div id="app"></div><script>globalThis.__styleGuardInput=' +
          JSON.stringify(selected).replaceAll("<", "\\u003c") +
          ';</script><script type="module" src="/page.mjs"></script>',
      );
      return;
    }
    const file = files.get(pathname);
    if (!file) {
      response.writeHead(404);
      response.end("unknown guard route");
      return;
    }
    response.writeHead(200, { "content-type": "text/javascript;charset=utf-8" });
    response.end(fs.readFileSync(file));
  });
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  let browser;
  try {
    browser = await chromium.launch({ headless: true });
    for (const item of cases) {
      const observations = [];
      const row = {
        id: item.id,
        role: item.spelling
          ? "html-source-refusal"
          : item.fold
            ? "admitted-literal"
            : "cssom-lexical-refusal",
        observations,
        error: null,
      };
      rows.push(row);
      for (const kind of item.fold
        ? ["candidate", "dynamic", "static"]
        : ["candidate", "dynamic"]) {
        const observation = {
          kind,
          snapshots: [],
          console: [],
          pageErrors: [],
          requestFailures: [],
          dispose: null,
          error: null,
        };
        observations.push(observation);
        const page = await browser.newPage({
          viewport: { width: 640, height: 240 },
          deviceScaleFactor: 1,
        });
        page.setDefaultTimeout(10000);
        page.on("console", (message) =>
          observation.console.push({ type: message.type(), text: message.text() }),
        );
        page.on("pageerror", (error) => observation.pageErrors.push(wholeError(error)));
        page.on("requestfailed", (request) =>
          observation.requestFailures.push({ url: request.url(), failure: request.failure() }),
        );
        try {
          selected = { module: "/generated/" + item.id + "-" + kind + ".mjs" };
          await page.goto("http://127.0.0.1:" + server.address().port + "/");
          await page.waitForFunction(
            () => globalThis.__styleGuard?.ready || globalThis.__styleGuardFailure,
          );
          assert.equal(await page.evaluate(() => globalThis.__styleGuardFailure ?? null), null);
          const snapshot = async (name) => {
            observation.snapshots.push({
              name,
              ...(await page.evaluate(() => globalThis.__styleGuard.snapshot())),
            });
            await page.screenshot({
              path: path.join(directory, item.id + "-" + kind + "-" + name + ".png"),
              fullPage: true,
            });
          };
          await snapshot("initial");
          await page.locator("button").click();
          await snapshot("clicked");
          await page.evaluate(() => globalThis.__styleGuard.restore());
          await snapshot("restored");
          observation.dispose = await page.evaluate(() => globalThis.__styleGuard.dispose());
        } catch (error) {
          observation.error = wholeError(error);
        } finally {
          await page.close();
          json(path.join(directory, item.id + "-" + kind + "-browser.json"), observation);
        }
      }
      try {
        for (const observation of observations) {
          assert.equal(observation.error, null);
          assert.deepEqual(observation.pageErrors, []);
          assert.deepEqual(observation.requestFailures, []);
          assert.deepEqual(
            observation.console.filter((message) => ["warning", "error"].includes(message.type)),
            [],
          );
          assert.deepEqual(
            observation.snapshots.map((snapshot) => snapshot.name),
            ["initial", "clicked", "restored"],
          );
          for (const [index, snapshot] of observation.snapshots.entries()) {
            assert.equal(snapshot.sameNodes, true);
            assert.deepEqual(snapshot.diagnostics, { warnings: [], errors: [] });
            assert.equal(snapshot.metrics.setup, 1);
            assert.equal(snapshot.metrics.clicks, index === 0 ? 0 : 1);
            assert.deepEqual(
              snapshot.metrics.events,
              index === 0 ? [] : [{ type: "click", trusted: true }],
            );
          }
          assert.deepEqual(observation.dispose, {
            html: "",
            diagnostics: { warnings: [], errors: [] },
          });
        }
        const actual = observations[0],
          dynamic = observations[1];
        if (item.fold) {
          assert.deepEqual(
            actual.snapshots,
            observations[2].snapshots,
            "admitted literal retains the genuine static-template raw DOM/effects",
          );
          assert.deepEqual(
            actual.snapshots.map((row) => row.styles),
            dynamic.snapshots.map((row) => row.styles),
            "independent CSSOM computed/priority equivalence; all raw strings are retained separately",
          );
        } else
          assert.deepEqual(
            actual.snapshots,
            dynamic.snapshots,
            "refused source retains complete dynamic raw DOM/style/effects",
          );
      } catch (error) {
        row.error = wholeError(error);
      }
      json(path.join(directory, item.id + "-browser.json"), row);
    }
  } finally {
    await browser?.close();
    await new Promise((resolve) => server.close(resolve));
  }
  const result = {
    runtime: input.runtime,
    chromiumVersion: browser?.version(),
    scope: "additive CSR-only guards; separate from original supported8 SSR/hydration",
    rows,
    requests,
  };
  json(path.join(directory, "browser.json"), result);
  assert.equal(rows.length, 12);
  assert(
    rows.every((row) => row.error === null),
    "complete guard rows must pass; inspect retained raw counterparts",
  );
  return result;
}
