import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { createRenderJobs } from "../../docs/scripts/navigation-render-concurrency.ts";
import { SCRIPT_BASENAMES } from "../../docs/theme/background.ts";

const root = path.resolve(import.meta.dirname, "../..");
const script = path.join(root, "docs/scripts/verify-navigation-render.ts");

async function verify(dir: string, output: string, workers: string, routes: string[]) {
  return new Promise<{ status: number | null; log: string }>((resolve, reject) => {
    const child = spawn(
      process.execPath,
      [
        script,
        "--dir",
        dir,
        "--output",
        output,
        "--workers",
        workers,
        "--routes",
        routes.join(","),
      ],
      {
        cwd: root,
        stdio: ["ignore", "pipe", "pipe"],
      },
    );
    let log = "";
    child.stdout.on("data", (data) => {
      log += data;
    });
    child.stderr.on("data", (data) => {
      log += data;
    });
    child.once("error", reject);
    child.once("close", (status) => resolve({ status, log }));
  });
}

async function json(file: string) {
  return JSON.parse(await readFile(file, "utf8"));
}

async function prepareLocaleControls(dir: string) {
  const scripts = SCRIPT_BASENAMES.filter((name) => name.startsWith("i18n/"));
  for (const name of [...scripts.map((name) => `${name}.js`), "i18n/locale-selector.css"]) {
    const destination = path.join(dir, "locale-fixture", name);
    await mkdir(path.dirname(destination), { recursive: true });
    await writeFile(destination, await readFile(path.join(root, "docs/theme", name)));
  }
  const html = `<!doctype html><html data-theme="light"><head><meta charset="utf-8"><title>Locale fixture</title>
    <link rel="stylesheet" href="/locale-fixture/i18n/locale-selector.css">
    <style>
    :root { font-size: 16px; --octc-font-mono: monospace; --octc-font-sans: Arial, sans-serif;
      --octc-color-bg: #fff; --octc-color-bg-alt: #eee; --octc-color-text: #222;
      --octc-color-text-muted: #444; --octc-color-border: #aaa; --octc-color-primary: #06c; }
    :root[data-theme="dark"] { --octc-color-bg: #121212; --octc-color-bg-alt: #333;
      --octc-color-text: #fff; --octc-color-text-muted: #aaa; --octc-color-border: #555; }
    body { margin: 0; padding: 16px; color: var(--octc-color-text); background: var(--octc-color-bg); }
    .header { display: flex; width: 100%; box-sizing: border-box; padding: 8px 0; min-height: 80px; }
    .header-actions { display: flex; align-items: center; gap: 8px; margin-left: auto; }
    .content { min-height: 150px; } button { min-height: 44px; }
    [data-mobile-theme] { display: none; }
    @media (max-width: 768px) {
      .header-actions { display: none; } [data-mobile-theme] { display: block; }
    }
    </style></head><body>
    <header class="header"><div class="header-actions">
    <button class="search-button">Search</button><button class="theme-toggle">Theme</button>
    </div></header><main class="content"><h1>Locale fixture</h1>
    <a href="/linked/#anchor">Linked page</a></main><button data-mobile-theme>Theme</button>
    ${scripts.map((name) => `<script src="/locale-fixture/${name}.js"></script>`).join("\n")}
    <script>document.addEventListener("click", (event) => {
      if (event.target.closest(".theme-toggle, [data-mobile-theme]"))
        document.documentElement.dataset.theme =
          document.documentElement.dataset.theme === "dark" ? "light" : "dark";
    });</script></body></html>`;
  for (const locale of ["en", "ja", "zh-CN", "pt-BR", "fr"]) {
    const destination = path.join(dir, locale === "en" ? "" : locale, "getting-started");
    await mkdir(destination, { recursive: true });
    await writeFile(path.join(destination, "index.html"), html);
  }
}

test(
  "real browser keeps serial/parallel receipts equivalent and persists failed-page evidence",
  { timeout: 120_000 },
  async () => {
    const temporary = await mkdtemp(path.join(os.tmpdir(), "vize-docs-render-workers-"));
    try {
      const dir = path.join(temporary, "dist");
      const routes = ["/a/b", "/a-b"];
      // Deliberately small pages exercise the runner and linked-page assertions.
      // The real capture control separately exercises long-page layout fallback.
      for (const route of [...routes, "/broken", "/linked"]) {
        await mkdir(path.join(dir, route), { recursive: true });
        const href = route === "/broken" ? "/missing/" : "/linked/#anchor";
        await writeFile(
          path.join(dir, route, "index.html"),
          `<!doctype html><html><head><title>Render runner fixture</title></head><body>
        <h1>Render runner fixture</h1><a class="feature-card" href="${href}">Linked page</a>
        <span id="anchor">Anchor</span></body></html>`,
        );
      }
      await prepareLocaleControls(dir);
      const receipts = [];
      const durations = [];
      for (const workers of ["1", "2"]) {
        const output = path.join(temporary, `workers-${workers}`);
        const result = await verify(dir, output, workers, routes);
        assert.equal(result.status, 0, result.log);
        const receipt = await json(path.join(output, "receipt.json"));
        const timings = await json(path.join(output, "timings.json"));
        const locale = await json(path.join(output, "locale-controls/receipt.json"));
        assert.equal(locale.completed, true);
        assert.equal(locale.cases.length, 8);
        assert.equal(locale.failure, null);
        assert.equal(receipt.status, "passed");
        assert.equal(timings.schemaVersion, 1);
        assert.equal(timings.workers, Number(workers));
        assert.equal(timings.status, "passed");
        assert.equal(timings.passedCount, 4);
        assert.equal(timings.failedCount, 0);
        assert.equal(timings.incompleteCount, 0);
        assert.deepEqual(
          receipt.reports.map(({ route, device }: { route: string; device: string }) => [
            route,
            device,
          ]),
          createRenderJobs(routes).map(({ route, device }) => [route, device]),
        );
        assert(
          timings.jobs.every(
            (job: { status: string; phases: { phase: string; status: string }[] }) =>
              job.status === "passed" &&
              job.phases.at(-1)?.phase === "context-close" &&
              job.phases.every(({ status }) => status === "passed"),
          ),
        );
        for (const report of receipt.reports)
          for (const tile of report.capture.tiles)
            assert((await readFile(path.join(output, tile.file))).length > 0);
        const control = await json(path.join(output, "capture-controls/receipt.json"));
        assert.equal(control.capture.layout.identical, false);
        assert.equal(control.capture.layout.captureViewportHeight, 844);
        assert.equal(control.capture.height, 10_000);
        assert.equal(control.capture.tiles[0].top, 0);
        const last = control.capture.tiles.at(-1);
        assert.equal(last.top + last.height, 10_000);
        if (workers === "2") {
          const [first, second] = timings.jobs;
          assert(
            Date.parse(second.startedAt) < Date.parse(first.startedAt) + first.durationMs,
            "real second context starts before the first route completes",
          );
        }
        // Each invocation has its own ephemeral server port; compare the same targets.
        receipts.push(
          receipt.reports.map((report: { links: string[] }) => ({
            ...report,
            links: report.links.map((href) => {
              const target = new URL(href);
              return target.pathname + target.hash;
            }),
          })),
        );
        durations.push({ workers: Number(workers), durationMs: timings.durationMs });
      }
      assert.deepEqual(
        receipts[0],
        receipts[1],
        "parallel execution retains identical complete evidence",
      );
      console.log(`Docs render fixture measurements: ${JSON.stringify(durations)}`);

      const output = path.join(temporary, "failure");
      const result = await verify(dir, output, "2", [routes[0], "/broken"]);
      assert.notEqual(result.status, 0);
      const receipt = await json(path.join(output, "receipt.json"));
      const timings = await json(path.join(output, "timings.json"));
      assert.equal(receipt.status, "failed");
      assert.equal(receipt.reports.length, 2);
      assert(receipt.reports.every(({ route }: { route: string }) => route === routes[0]));
      assert.equal(timings.status, "failed");
      assert.equal(timings.failedCount, 2);
      assert.equal(timings.incompleteCount, 0);
      for (const job of timings.jobs.filter(
        ({ route }: { route: string }) => route === "/broken",
      )) {
        assert.equal(job.status, "failed");
        assert.match(job.error, /broken link/);
        assert.equal(
          job.phases.find(({ phase }: { phase: string }) => phase === "links").status,
          "failed",
        );
        assert.equal(job.phases.at(-1).phase, "context-close");
        assert.equal(job.phases.at(-1).status, "passed");
      }
      assert(
        timings.phases
          .filter(({ phase }: { phase: string }) =>
            ["browser-close", "server-close"].includes(phase),
          )
          .every(({ status }: { status: string }) => status === "passed"),
      );
    } finally {
      await rm(temporary, { recursive: true, force: true });
    }
  },
);
