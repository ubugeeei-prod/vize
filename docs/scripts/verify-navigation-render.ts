import assert from "node:assert/strict";
import { mkdir, readFile, stat, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import path from "node:path";
import { parseArgs } from "node:util";
import { chromium } from "playwright";
import type { Browser } from "playwright";
import { resolvePuppeteerExecutablePath } from "../browser-path.js";
import { navigationRenderRoutes } from "./navigation-render-routes.ts";
import { verifyCaptureRenderControls } from "./capture-render-controls.ts";
import { verifyLocaleRenderControls } from "./locale-control-render.ts";
import { verifyNavigationRenderJob } from "./navigation-render-page.ts";
import type { RouteReceipt } from "./navigation-render-page.ts";
import {
  collectRenderReceipts,
  createRenderJobs,
  measureRenderPhase,
  parseRenderWorkers,
  renderError,
  runRenderJobs,
  withRenderContext,
} from "./navigation-render-concurrency.ts";
import type { RenderJobResult, RenderPhaseTiming } from "./navigation-render-concurrency.ts";

const { values } = parseArgs({
  options: {
    dir: { type: "string", default: "docs/dist" },
    output: { type: "string", default: "docs-render-evidence" },
    routes: { type: "string" },
    workers: { type: "string" },
  },
});
const dist = path.resolve(values.dir);
const output = path.resolve(values.output);
const routes = values.routes?.split(",") ?? [
  ...navigationRenderRoutes,
  ...["zh-CN", "pt-BR", "fr"].map((locale) => `/${locale}/rules/vue`),
];
const workers = parseRenderWorkers(values.workers);
const jobs = createRenderJobs(routes);
const startedAt = new Date().toISOString();
const start = performance.now();
const phases: RenderPhaseTiming[] = [];
const types: Record<string, string> = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".svg": "image/svg+xml",
  ".png": "image/png",
};
const server = createServer(async (request, response) => {
  try {
    assert(request.url !== undefined, "Static request has no URL");
    const pathname = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
    let file = path.resolve(dist, `.${pathname}`);
    if (!file.startsWith(`${dist}${path.sep}`) && file !== dist) throw new Error("Outside site");
    if ((await stat(file)).isDirectory()) file = path.join(file, "index.html");
    response.writeHead(200, {
      "content-type": types[path.extname(file)] ?? "application/octet-stream",
    });
    response.end(await readFile(file));
  } catch {
    response.writeHead(404);
    response.end("Not found");
  }
});
await mkdir(output, { recursive: true });
let browser: Browser | undefined;
let results: RenderJobResult<RouteReceipt>[] = [];
let reports: RouteReceipt[] = [];
let failure: unknown;
let failed = false;
const linkedPages = new Map<string, Promise<string>>();
try {
  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  assert(address && typeof address !== "string", "Static server has no TCP address");
  const origin = `http://127.0.0.1:${address.port}`;
  const activeBrowser = await measureRenderPhase(phases, "browser-launch", () =>
    chromium.launch({
      executablePath: resolvePuppeteerExecutablePath(),
      headless: true,
    }),
  );
  browser = activeBrowser;
  await measureRenderPhase(phases, "capture-controls", () =>
    verifyCaptureRenderControls(activeBrowser, origin, output),
  );
  await verifyLocaleRenderControls(activeBrowser, origin, output);
  results = await runRenderJobs(jobs, workers, async (job, phase) =>
    withRenderContext(
      () =>
        phase("context", () =>
          activeBrowser.newContext({ viewport: job.viewport, reducedMotion: "reduce" }),
        ),
      (context) => verifyNavigationRenderJob(context, job, phase, { origin, output, linkedPages }),
      (close) => phase("context-close", close),
    ),
  );
  reports = collectRenderReceipts(jobs, results);
  const failures = results.filter(({ timing }) => timing.status === "failed");
  assert.equal(
    failures.length,
    0,
    failures.map(({ timing }) => `${timing.device} ${timing.route}: ${timing.error}`).join("\n"),
  );
  assert.equal(reports.length, jobs.length, "All requested route/device receipts must pass");
} catch (error) {
  failed = true;
  failure = error;
} finally {
  // Release browser/server even if assertion, capture or evidence writing fails.
  for (const [phase, close] of [
    [
      "browser-close",
      async () => {
        await browser?.close();
      },
    ],
    [
      "server-close",
      async () => {
        if (server.listening)
          await new Promise<void>((resolve, reject) =>
            server.close((error) => (error ? reject(error) : resolve())),
          );
      },
    ],
  ] as const) {
    try {
      await measureRenderPhase(phases, phase, close);
    } catch (error) {
      failure = failed
        ? new AggregateError([failure, error], "Render verification cleanup failed")
        : error;
      failed = true;
    }
  }
  const status = failed ? "failed" : "passed";
  const requestedJobs = jobs.map(({ index, route, device }) => ({ index, route, device }));
  await writeFile(
    path.join(output, "receipt.json"),
    JSON.stringify(
      { dist, status, workers, expectedCount: jobs.length, requestedJobs, reports },
      null,
      2,
    ) + "\n",
  );
  await writeFile(
    path.join(output, "timings.json"),
    JSON.stringify(
      {
        schemaVersion: 1,
        dist,
        workers,
        routes,
        requestedJobs,
        startedAt,
        durationMs: performance.now() - start,
        status,
        expectedCount: jobs.length,
        passedCount: reports.length,
        failedCount: results.filter(({ timing }) => timing.status === "failed").length,
        incompleteCount: jobs.length - results.length,
        phases,
        jobs: jobs.map(
          (job) =>
            results.find((result) => result.job.index === job.index)?.timing ?? {
              index: job.index,
              route: job.route,
              device: job.device,
              status: "pending",
              phases: [],
            },
        ),
        ...(failed ? { error: renderError(failure) } : {}),
      },
      null,
      2,
    ) + "\n",
  );
}
if (failed) throw failure;
console.log(
  `Verified ${reports.length} real rendered pages with ${workers} workers; desktop/mobile screenshots and timings in ${output}`,
);
