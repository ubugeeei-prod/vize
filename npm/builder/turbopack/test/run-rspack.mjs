import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { createRequire } from "node:module";
import { createServer } from "node:http";
import { once } from "node:events";
import { eachMapping, TraceMap } from "@jridgewell/trace-mapping";
import { consumer, output, sha256 } from "./prepare-consumer.mjs";

/** Exercise the packed existing adapter after sharing its module/map helpers. */
export async function checkRspackConsumer(browser, measurements) {
  const require = createRequire(path.join(consumer, "package.json"));
  const { rspack } = require("@rspack/core");
  const { VizePlugin } = require("@vizejs/rspack-plugin");
  const root = path.join(consumer, "rspack");
  const filename = "组件 space/RegressionCounter.vue";
  const original = await fs.readFile(path.join(root, filename), "utf8");
  const packets = [];
  for (const mode of ["development", "production"]) {
    const target = path.join(root, mode);
    const start = performance.now();
    const compiler = rspack({
      context: root,
      mode,
      entry: "./entry.js",
      devtool: "source-map",
      output: { path: target, filename: "bundle.js", cssFilename: "styles.css" },
      optimization: { minimize: false },
      // This map regression retains statements in the unminified Vue runtime.
      // Keep a bounded fixture budget and continue rejecting all diagnostics.
      performance: { maxAssetSize: 512 * 1024, maxEntrypointSize: 512 * 1024 },
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
    });
    let stats;
    try {
      stats = await new Promise((resolve, reject) => {
        compiler.run((error, result) => (error ? reject(error) : resolve(result)));
      });
    } finally {
      await new Promise((resolve, reject) =>
        compiler.close((error) => (error ? reject(error) : resolve())),
      );
    }
    const report = stats.toJson({
      all: false,
      errors: true,
      warnings: true,
      timings: true,
      assets: true,
    });
    await fs.writeFile(
      path.join(output, `rspack-${mode}-stats.json`),
      JSON.stringify(report, null, 2) + "\n",
    );
    assert.deepEqual(report.errors, []);
    assert.deepEqual(report.warnings, []);
    measurements.push({ phase: `rspack-${mode}-build`, ms: performance.now() - start });

    const bundle = await fs.readFile(path.join(target, "bundle.js"), "utf8");
    const mapBytes = await fs.readFile(path.join(target, "bundle.js.map"));
    const map = JSON.parse(mapBytes);
    const authored = map.sources.findIndex((source) => decodeURI(source).endsWith(filename));
    assert.notEqual(authored, -1);
    assert.equal(map.sourcesContent[authored], original);
    const prefix = original.slice(0, original.indexOf("const initial"));
    const line = prefix.split("\n").length;
    const matches = [];
    eachMapping(new TraceMap(map), (mapping) => {
      if (decodeURI(mapping.source ?? "").endsWith(filename) && mapping.originalLine === line) {
        matches.push(mapping);
      }
    });
    assert.ok(matches.length, "Rspack must retain the script statement's authored position");
    const generated = bundle.split("\n");
    assert.ok(
      matches.some((mapping) =>
        /^(?:const|let|var) [\w$]+ = 2;/.test(
          generated[mapping.generatedLine - 1].slice(mapping.generatedColumn),
        ),
      ),
      "the composed map points at the surviving statement after Unicode text",
    );
    await fs.mkdir(path.join(output, "maps"), { recursive: true });
    await fs.writeFile(path.join(output, "maps", `rspack-${mode}.map`), mapBytes);

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
      packets.push({ mode, ...packet, mapSha256: sha256(mapBytes) });
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
