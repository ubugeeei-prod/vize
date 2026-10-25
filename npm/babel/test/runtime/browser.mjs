import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { createRequire } from "node:module";
import path from "node:path";
import { here, json } from "./environment.mjs";
import { wholeError } from "../hosts/contracts.mjs";

export async function observeBrowser(input, directory, serverReport) {
  const requireRuntime = createRequire(path.join(input.runtime.root, "package.json"));
  const { chromium } = requireRuntime("playwright");
  assert.equal(typeof chromium?.launch, "function", "the actual Playwright host exposes Chromium");
  const routes = new Map([
    ["/runtime.js", input.runtime.browserEntry],
    ["/browser-page.mjs", path.join(here, "browser-page.mjs")],
  ]);
  for (const name of fs.readdirSync(directory))
    if (name.endsWith(".mjs")) routes.set("/generated/" + name, path.join(directory, name));
  let current;
  const requests = [];
  const server = http.createServer((request, response) => {
    const url = new URL(request.url, "http://127.0.0.1/");
    requests.push({ pathname: url.pathname, method: request.method });
    if (url.pathname === "/favicon.ico") {
      response.writeHead(204);
      response.end();
      return;
    }
    if (url.pathname === "/") {
      const state = JSON.stringify(current).replaceAll("<", "\\u003c");
      response.writeHead(200, { "content-type": "text/html; charset=utf-8" });
      response.end(
        '<!doctype html><html><head><meta charset="utf-8"><script type="importmap">{"imports":{"vue":"/runtime.js"}}</script></head><body><div id="app"></div><script>globalThis.__vizeRuntimeInput=' +
          state +
          ';</script><script type="module" src="/browser-page.mjs"></script></body></html>',
      );
      return;
    }
    const file = routes.get(url.pathname);
    if (!file) {
      response.writeHead(404);
      response.end("unknown fixture route");
      return;
    }
    response.writeHead(200, { "content-type": "text/javascript; charset=utf-8" });
    response.end(fs.readFileSync(file));
  });
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  const origin = "http://127.0.0.1:" + address.port;
  const observations = [];
  let browser;
  try {
    browser = await chromium.launch({ headless: true });
    for (const mode of ["mount", "hydrate"])
      for (const kind of ["candidate", "reference"]) {
        const row = {
          kind,
          mode,
          snapshots: [],
          console: [],
          pageErrors: [],
          requestFailures: [],
          hydration: null,
          dispose: null,
          failure: null,
        };
        observations.push(row);
        const page = await browser.newPage({
          viewport: { width: 640, height: 360 },
          deviceScaleFactor: 1,
        });
        page.setDefaultTimeout(10000);
        page.on("console", (message) =>
          row.console.push({
            type: message.type(),
            text: message.text(),
            location: message.location(),
          }),
        );
        page.on("pageerror", (error) => row.pageErrors.push(wholeError(error)));
        page.on("requestfailed", (request) =>
          row.requestFailures.push({ url: request.url(), failure: request.failure() }),
        );
        try {
          const server = serverReport.observations.find((row) => row.kind === kind);
          assert(server && server.failure === null && typeof server.html === "string");
          current = {
            mode,
            kind,
            version: input.runtime.version,
            target: input.runtime.target,
            module: "/generated/" + (kind === "candidate" ? "client.mjs" : "reference-client.mjs"),
            html: server.html,
          };
          await page.goto(origin + "/");
          await page.waitForFunction(
            () => globalThis.__vizeRuntime?.ready || globalThis.__vizeRuntimeFailure,
          );
          row.hydration = await page.evaluate(() => globalThis.__vizeHydrationObservation ?? null);
          assert.equal(
            await page.evaluate(() => globalThis.__vizeRuntimeFailure ?? null),
            null,
            "whole browser bootstrap succeeds",
          );
          const snapshot = async (name) => {
            const packet = await page.evaluate(() => globalThis.__vizeRuntime.snapshot());
            row.snapshots.push({ name, ...packet });
            await page.screenshot({
              path: path.join(directory, kind + "-" + mode + "-" + name + ".png"),
              fullPage: true,
            });
          };
          await snapshot("initial");
          await page.evaluate(() => globalThis.__vizeRuntime.update());
          await snapshot("updated");
          assert.equal(await page.locator("button").count(), 1);
          await page.locator("button").click();
          await snapshot("clicked");
          await page.evaluate(() => globalThis.__vizeRuntime.restore());
          await snapshot("restored");
          row.dispose = await page.evaluate(() => globalThis.__vizeRuntime.dispose());
        } catch (error) {
          row.failure = wholeError(error);
        } finally {
          await page.close();
          json(path.join(directory, "browser-" + kind + "-" + mode + ".json"), row);
        }
      }
  } finally {
    await browser?.close();
    await new Promise((resolve) => server.close(resolve));
  }
  const report = {
    chromiumVersion: browser?.version(),
    runtime: input.runtime,
    observations,
    requests,
  };
  json(path.join(directory, "browser.json"), report);
  return report;
}
