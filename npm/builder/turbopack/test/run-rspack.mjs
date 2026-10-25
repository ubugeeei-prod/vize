import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { once } from "node:events";
import { eachMapping, originalPositionFor, TraceMap } from "@jridgewell/trace-mapping";
import { consumer, output, sha256 } from "./prepare-consumer.mjs";
import { buildRspack, verifyDebugAdvisories } from "./rspack-authority.mjs";

/** Exercise the packed existing adapter after sharing its module/map helpers. */
export async function checkRspackConsumer(browser, measurements) {
  const require = createRequire(path.join(consumer, "package.json"));
  const { rspack } = require("@rspack/core");
  const { VizePlugin } = require("@vizejs/rspack-plugin");
  const root = path.join(consumer, "rspack");
  const filename = "组件 space/RegressionCounter.vue";
  const original = await fs.readFile(path.join(root, filename), "utf8");
  const packets = [];
  for (const mode of ["development", "production-unminified-debug-map", "production"]) {
    const debug = mode !== "production";
    const target = path.join(root, mode);
    const start = performance.now();
    const report = await buildRspack(
      rspack,
      {
        context: root,
        mode: mode === "development" ? "development" : "production",
        entry: "./entry.js",
        devtool: "source-map",
        output: { path: target, filename: "bundle.js", cssFilename: "styles.css" },
        ...(debug ? { optimization: { minimize: false } } : {}),
        experiments: { css: true },
        resolve: { alias: { vue: require.resolve("vue/dist/vue.runtime.esm-bundler.js") } },
        module: {
          rules: [
            {
              test: /\.vue$/,
              loader: require.resolve("@vizejs/rspack-plugin/loader"),
              options: { sourceMap: true },
            },
          ],
        },
        plugins: [new VizePlugin({ typescript: false, css: { native: true } })],
      },
      `rspack-${mode}`,
    );
    measurements.push({ phase: `rspack-${mode}-build`, ms: performance.now() - start });

    const bundle = await fs.readFile(path.join(target, "bundle.js"), "utf8");
    if (mode === "production-unminified-debug-map")
      await verifyDebugAdvisories(rspack, root, bundle, report.warnings);
    else assert.deepEqual(report.warnings, []);
    const mapBytes = await fs.readFile(path.join(target, "bundle.js.map"));
    const map = JSON.parse(mapBytes);
    const traced = new TraceMap(map);
    // The host also maps generated CSS module exports, and disambiguates shared
    // resource paths with fallback queries. Keep those full identities intact.
    const authored = new Set(
      map.sources.flatMap((source, index) =>
        decodeURI(source).split("?", 1)[0].endsWith(filename) &&
        map.sourcesContent[index] === original
          ? [traced.resolvedSources[index]]
          : [],
      ),
    );
    assert.ok(authored.size, "the complete original SFC must own an authored source identity");
    const prefix = original.slice(0, original.indexOf(debug ? "const initial" : "const emoji"));
    const line = prefix.split("\n").length;
    const matches = [];
    eachMapping(traced, (mapping) => {
      if (authored.has(mapping.source) && mapping.originalLine === line) {
        matches.push(mapping);
      }
    });
    if (debug)
      assert.ok(matches.length, "Rspack must retain the script statement's authored position");
    const generated = bundle.split("\n");
    if (debug)
      assert.ok(
        matches.some((mapping) =>
          /^(?:const|let|var) [\w$]+ = 2;/.test(
            generated[mapping.generatedLine - 1].slice(mapping.generatedColumn),
          ),
        ),
        "the composed map points at the surviving statement after Unicode text",
      );
    else {
      const marker = /🐱|\\u[dD]83[dD]\\u[dD][cC]31|\\u\{1[fF]431\}/.exec(bundle);
      assert.ok(marker, "optimized code retains the visible authored string");
      const position = bundle.slice(0, marker.index);
      const origin = originalPositionFor(traced, {
        line: position.split("\n").length,
        column: position.length - position.lastIndexOf("\n") - 1,
      });
      assert.ok(
        authored.has(origin.source),
        "the surviving token must belong to authored SFC content",
      );
      assert.equal(
        origin.line,
        line,
        "optimized surviving string points at its authored declaration",
      );
    }
    await fs.mkdir(path.join(output, "maps"), { recursive: true });
    await fs.writeFile(path.join(output, "maps", `rspack-${mode}.map`), mapBytes);
    await fs.writeFile(path.join(output, "maps", `rspack-${mode}.js`), bundle);

    const server = createServer(async (request, response) => {
      if (request.url === "/") {
        response.setHeader("Content-Type", "text/html; charset=utf-8");
        response.end(
          '<!doctype html><html><head><meta charset="utf-8"><link rel="icon" href="data:,"><link rel="stylesheet" href="/styles.css"></head><body><div id="app"></div><script src="/bundle.js"></script></body></html>',
        );
        return;
      }
      const files = {
        "/bundle.js": "application/javascript",
        "/styles.css": "text/css",
        "/bundle.js.map": "application/json",
        "/styles.css.map": "application/json",
      };
      if (!Object.hasOwn(files, request.url)) {
        response.writeHead(404).end();
        return;
      }
      try {
        response.setHeader("Content-Type", `${files[request.url]}; charset=utf-8`);
        response.end(await fs.readFile(path.join(target, request.url.slice(1))));
      } catch (error) {
        response.writeHead(500).end(String(error));
      }
    });
    server.listen(0, "127.0.0.1");
    await once(server, "listening");
    const page = await browser.newPage();
    const errors = [];
    const warnings = [];
    page.on("pageerror", (error) => errors.push(error.stack ?? error.message));
    page.on("console", (message) => {
      if (message.type() === "error") errors.push(message.text());
      if (message.type() === "warning") warnings.push(message.text());
    });
    try {
      await page.goto(`http://127.0.0.1:${server.address().port}`);
      await page.locator("#regression-counter").waitFor();
      assert.equal((await page.locator("#regression-counter").textContent()).trim(), "🐱 2");
      await page.locator("#regression-counter").click();
      await page.waitForFunction(
        () => document.querySelector("#regression-counter").textContent.trim() === "🐱 3",
      );
      const packet = await page.evaluate(() => {
        const element = document.querySelector("#regression-counter");
        const style = getComputedStyle(element);
        return {
          html: document.documentElement.outerHTML,
          color: style.color,
          padding: style.paddingLeft,
        };
      });
      assert.equal(packet.color, "rgb(12, 34, 56)");
      assert.equal(packet.padding, "7px");
      assert.deepEqual(errors, []);
      assert.deepEqual(warnings, []);
      packets.push({
        mode,
        ...packet,
        authoredSources: [...authored],
        mapSha256: sha256(mapBytes),
      });
    } finally {
      await fs.writeFile(
        path.join(output, `rspack-${mode}-browser-errors.json`),
        JSON.stringify({ errors, warnings }, null, 2) + "\n",
      );
      await page.close();
      server.closeAllConnections();
      await new Promise((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      );
    }
  }
  await fs.writeFile(
    path.join(output, "rspack-regression.json"),
    JSON.stringify({ runtime: require("@rspack/core/package.json").version, packets }, null, 2) +
      "\n",
  );
}
