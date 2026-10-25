import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { freshMetrics } from "./fixtures.mjs";
import { wholeError } from "../hosts/contracts.mjs";
import { json } from "./environment.mjs";

const input = JSON.parse(fs.readFileSync(process.argv[2]));
const requireRuntime = createRequire(path.join(input.runtime.root, "package.json"));
const Vue = await import(pathToFileURL(requireRuntime.resolve("vue")).href);
const renderer = await import(pathToFileURL(requireRuntime.resolve("@vue/server-renderer")).href);
assert.equal(Vue.version, input.runtime.version);
const report = { runtime: input.runtime, observations: [], fatal: null };
for (const kind of ["candidate", "reference"]) {
  const row = { kind, html: null, metrics: null, warnings: [], errors: [], failure: null };
  report.observations.push(row);
  const warn = console.warn,
    error = console.error;
  try {
    console.warn = (...args) => row.warnings.push(args.map(String));
    console.error = (...args) => row.errors.push(args.map(String));
    globalThis.__vizeBabelRuntime = freshMetrics();
    const file = path.join(
      input.directory,
      kind === "candidate" ? "server.mjs" : "reference-server.mjs",
    );
    const module = await import(pathToFileURL(file).href);
    assert(
      module.App && typeof module.App === "object",
      "block component has actual component-object ownership",
    );
    const app = Vue.createSSRApp(module.App);
    app.config.warnHandler = (message) => row.warnings.push([message]);
    app.config.errorHandler = (error) => row.errors.push([wholeError(error)]);
    row.html = await renderer.renderToString(app);
    row.metrics = structuredClone(globalThis.__vizeBabelRuntime);
    assert.deepEqual(row.metrics, {
      setup: { App: 1, Counter: 1 },
      childEvents: 0,
      parentEvents: 0,
      events: [],
    });
    assert.deepEqual(row.warnings, []);
    assert.deepEqual(row.errors, []);
    assert.equal(typeof row.html, "string");
    assert(row.html.includes("雪 🧭:0"));
  } catch (error) {
    row.failure = wholeError(error);
  } finally {
    console.warn = warn;
    console.error = error;
  }
}
json(input.report, report);
if (report.observations.some((row) => row.failure)) process.exitCode = 1;
