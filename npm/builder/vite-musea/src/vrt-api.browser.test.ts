import assert from "node:assert/strict";
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { createServer as createTcpServer } from "node:net";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { chromium, type Page } from "playwright";
import { PNG } from "pngjs";
import { createServer } from "vite";
import vize from "../../vite/src/index.ts";
import { musea } from "./plugin/index.ts";
import type { VrtSummary } from "./vrt/types.ts";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));

interface CaptureResponse {
  success: boolean;
  summary: VrtSummary;
  results: Array<{
    artPath: string;
    viewport: string;
    snapshotPath: string;
    currentPath?: string;
    diffPercentage: number;
    passed: boolean;
    isNew: boolean;
    error?: string;
  }>;
  artifacts: { snapshotDir: string; jsonReportPath: string };
}

async function unusedPort(): Promise<number> {
  const listener = createTcpServer();
  await new Promise<void>((resolve, reject) => {
    listener.once("error", reject);
    listener.listen(0, "127.0.0.1", resolve);
  });
  const address = listener.address();
  assert.ok(address && typeof address !== "string");
  await new Promise<void>((resolve, reject) =>
    listener.close((error) => (error ? reject(error) : resolve())),
  );
  return address.port;
}

async function capture(page: Page): Promise<CaptureResponse> {
  const response = page.waitForResponse((item) => item.url().endsWith("/api/run-vrt"));
  await page.getByRole("button", { name: "Run VRT", exact: true }).click();
  const actual = await response;
  assert.equal(actual.status(), 200);
  const data = (await actual.json()) as CaptureResponse;
  assert.equal(data.success, true, JSON.stringify(data));
  await page.getByRole("button", { name: "Run VRT", exact: true }).waitFor();
  return data;
}

void test(
  "gallery VRT captures honor the actual project viewport, snapshot directory and threshold",
  { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1" },
  async () => {
    const root = await mkdtemp(path.join(os.tmpdir(), "musea-native-vrt-api-"));
    const output = path.join(repository, "artifacts/musea-native-vrt-api");
    await mkdir(output, { recursive: true });
    await mkdir(path.join(root, "src"));
    const artPath = path.join(root, "src/Button.art.vue");
    await cp(
      path.join(repository, "tests/tooling/fixtures/musea/snapshot-collision/left/Button.art.vue"),
      artPath,
    );
    const authored = await readFile(artPath, "utf8");
    await writeFile(path.join(root, "index.html"), "<!doctype html><html><body></body></html>");
    const viewport = { name: "authored-compact", width: 320, height: 180 };
    const configuredSnapshotDir = path.join(root, "reviewed-baselines");
    const config = {
      viewports: [viewport],
      snapshotDir: "reviewed-baselines",
      threshold: 100,
      workers: 2,
      capture: { settleTime: 0, waitForNetwork: false },
      comparison: { antiAliasing: false },
    };
    const server = await createServer({
      root,
      configFile: false,
      resolve: {
        alias: { vue: fileURLToPath(import.meta.resolve("vue/dist/vue.runtime.esm-bundler.js")) },
      },
      plugins: [vize(), musea({ include: ["src/**/*.art.vue"], vrt: config })],
      server: { host: "127.0.0.1", port: await unusedPort(), strictPort: true },
    });
    const browser = await chromium.launch();
    const page = await browser.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(String(error)));
    const observations: unknown[] = [];
    try {
      await server.listen();
      const address = server.httpServer!.address();
      assert.ok(address && typeof address !== "string");
      await page.goto(`http://127.0.0.1:${address.port}/__musea__/`);
      await page.locator(".art-item").filter({ hasText: "Left" }).click();
      const preview = page.frameLocator(".variant-card iframe");
      await preview.getByRole("button", { name: "Left", exact: true }).waitFor();
      assert.equal(await preview.locator("museacomponent").count(), 0);
      await page.getByRole("button", { name: "VRT", exact: true }).click();
      const first = await capture(page);
      observations.push({ phase: "configured-initial", data: first });
      await writeFile(path.join(output, "initial.json"), JSON.stringify(first, null, 2));
      assert.equal(first.summary.total, 1);
      assert.equal(first.summary.new, 1);
      assert.equal(first.artifacts.snapshotDir, configuredSnapshotDir);
      assert.equal(first.results[0].viewport, viewport.name);
      assert.equal(first.results[0].artPath, artPath);
      const png = PNG.sync.read(await readFile(first.results[0].snapshotPath));
      assert.deepEqual({ width: png.width, height: png.height }, { width: 320, height: 180 });
      const repeated = await capture(page);
      observations.push({ phase: "configured-repeat", data: repeated });
      assert.equal(repeated.summary.total, 1);
      assert.equal(repeated.summary.passed, 1);
      assert.equal(repeated.results[0].diffPercentage, 0);
      assert.equal(repeated.results[0].snapshotPath, first.results[0].snapshotPath);
      await writeFile(artPath, authored.replace("#0000ff", "#00ff00"));
      await page.waitForFunction(async (artPath) => {
        const response = await fetch(`/__musea__/api/arts/${encodeURIComponent(artPath)}`);
        const art = await response.json();
        return art.variants[0].template.includes("#00ff00");
      }, artPath);
      const changed = await capture(page);
      observations.push({ phase: "configured-threshold", data: changed });
      assert.equal(changed.summary.total, 1);
      assert.equal(changed.summary.passed, 1);
      assert.ok(changed.results[0].diffPercentage > 0, JSON.stringify(changed));
      assert.equal(changed.results[0].passed, true);
      await page.getByRole("checkbox", { name: "Update snapshots", exact: true }).check();
      const updated = await capture(page);
      observations.push({ phase: "configured-update", data: updated });
      assert.equal(updated.summary.total, 1);
      await page.getByRole("checkbox", { name: "Update snapshots", exact: true }).uncheck();
      const clean = await capture(page);
      observations.push({ phase: "configured-updated-repeat", data: clean });
      assert.equal(clean.results[0].diffPercentage, 0);
      assert.equal(clean.summary.passed, 1);
      assert.deepEqual(errors, []);
      await page.screenshot({ path: path.join(output, "gallery.png"), fullPage: true });
    } catch (error) {
      await writeFile(
        path.join(output, "failure.json"),
        JSON.stringify(
          { error: String(error), observations, errors, html: await page.content() },
          null,
          2,
        ),
      );
      throw error;
    } finally {
      await cp(root, path.join(output, "project"), { recursive: true });
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify({ authored, config, observations, errors }, null, 2),
      );
      await browser.close();
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  },
);
