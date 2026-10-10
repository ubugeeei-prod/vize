import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { createServer as createTcpServer } from "node:net";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { chromium, type Page } from "playwright";
import { createServer } from "vite";
import vize from "../../vite/src/index.ts";
import { musea } from "./plugin/index.ts";
import type { MuseaVrtOptions } from "./types/plugin.ts";
import type { VrtSummary } from "./vrt/types.ts";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));
interface CaptureResponse {
  success: boolean;
  summary: VrtSummary;
  results: Array<{ artPath: string; snapshotPath: string; diffPercentage: number }>;
  artifacts: { htmlReportPath: string; jsonReportPath: string };
}
interface RetainedCapture {
  data: CaptureResponse;
  json: Buffer;
  html: Buffer;
  png: Buffer;
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

async function capture(page: Page, output: string, phase: string): Promise<RetainedCapture> {
  const response = page.waitForResponse((item) => item.url().endsWith("/api/run-vrt"));
  await page.getByRole("button", { name: "Run VRT", exact: true }).click();
  const actual = await response;
  const raw = await actual.text();
  const destination = path.join(output, phase);
  await mkdir(destination, { recursive: true });
  await writeFile(path.join(destination, "response.json"), raw);
  assert.equal(actual.status(), 200, raw);
  const data = JSON.parse(raw) as CaptureResponse;
  assert.equal(data.success, true, raw);
  const json = await readFile(data.artifacts.jsonReportPath);
  const html = await readFile(data.artifacts.htmlReportPath);
  const png = await readFile(data.results[0].snapshotPath);
  await writeFile(path.join(destination, "report.json"), json);
  await writeFile(path.join(destination, "report.html"), html);
  await writeFile(path.join(destination, "snapshot.png"), png);
  await page.getByRole("button", { name: "Run VRT", exact: true }).waitFor();
  return { data, json, html, png };
}

void test(
  "real gallery VRT retains both same-basename Art reports and separate native PNG owners",
  { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1" },
  async () => {
    const root = await mkdtemp(path.join(os.tmpdir(), "musea-native-vrt-reports-"));
    const output = path.join(repository, "artifacts/musea-native-vrt-reports");
    await mkdir(output, { recursive: true });
    const fixture = JSON.parse(
      await readFile(
        path.join(repository, "tests/_fixtures/differential/musea/vrt-report-ownership.json"),
        "utf8",
      ),
    ) as { sources: Array<{ path: string; sha256: string }> };
    for (const source of fixture.sources) {
      assert.equal(
        createHash("sha256")
          .update(await readFile(path.join(repository, source.path)))
          .digest("hex"),
        source.sha256,
      );
    }
    const paths: Record<string, string> = {};
    const authored: Record<string, string> = {};
    for (const side of ["left", "right"]) {
      const artPath = path.join(root, "src", side, "Button.art.vue");
      await mkdir(path.dirname(artPath), { recursive: true });
      await cp(
        path.join(
          repository,
          "tests/tooling/fixtures/musea/snapshot-collision",
          side,
          "Button.art.vue",
        ),
        artPath,
      );
      paths[side] = artPath;
      authored[side] = await readFile(artPath, "utf8");
    }
    await writeFile(path.join(root, "index.html"), "<!doctype html><html><body></body></html>");
    const config = JSON.parse(
      await readFile(
        path.join(repository, "tests/_fixtures/differential/musea/gallery-vrt-options.json"),
        "utf8",
      ),
    ) as MuseaVrtOptions;
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
      const retained: RetainedCapture[] = [];
      for (const [side, title] of [
        ["left", "Left"],
        ["right", "Right"],
      ]) {
        await page.locator(".art-item").filter({ hasText: title }).click();
        const preview = page.frameLocator(".variant-card iframe");
        await preview.getByRole("button", { name: title, exact: true }).waitFor();
        assert.equal(await preview.locator("museacomponent").count(), 0);
        await page.getByRole("button", { name: "VRT", exact: true }).click();
        const actual = await capture(page, output, `${side}-initial`);
        observations.push({ phase: `${side}-initial`, data: actual.data });
        assert.equal(actual.data.summary.total, 1);
        assert.equal(actual.data.summary.new, 1);
        assert.equal(actual.data.results[0].artPath, paths[side]);
        const physical = JSON.parse(actual.json.toString()) as {
          results: Array<{ artPath: string }>;
        };
        assert.deepEqual(
          physical.results.map((item) => item.artPath),
          [paths[side]],
        );
        retained.push(actual);
      }
      const [left, right] = retained;
      assert.notEqual(left.data.results[0].snapshotPath, right.data.results[0].snapshotPath);
      assert.notDeepEqual(left.png, right.png);
      // The old API has distinct PNG owners but overwrites both complete report files here.
      assert.notEqual(left.data.artifacts.jsonReportPath, right.data.artifacts.jsonReportPath);
      assert.notEqual(left.data.artifacts.htmlReportPath, right.data.artifacts.htmlReportPath);
      assert.deepEqual(await readFile(left.data.artifacts.jsonReportPath), left.json);
      assert.deepEqual(await readFile(left.data.artifacts.htmlReportPath), left.html);
      for (const [side, title, previous, untouched] of [
        ["left", "Left", left, right],
        ["right", "Right", right, left],
      ] as const) {
        await page.locator(".art-item").filter({ hasText: title }).click();
        await page.getByRole("button", { name: "VRT", exact: true }).click();
        const actual = await capture(page, output, `${side}-repeat`);
        observations.push({ phase: `${side}-repeat`, data: actual.data });
        assert.equal(actual.data.summary.passed, 1);
        assert.equal(actual.data.results[0].diffPercentage, 0);
        assert.equal(actual.data.results[0].snapshotPath, previous.data.results[0].snapshotPath);
        assert.deepEqual(actual.data.artifacts, previous.data.artifacts);
        assert.deepEqual(await readFile(untouched.data.artifacts.jsonReportPath), untouched.json);
        assert.deepEqual(await readFile(untouched.data.artifacts.htmlReportPath), untouched.html);
        previous.json = actual.json;
        previous.html = actual.html;
      }
      assert.deepEqual(errors, []);
      await page.screenshot({ path: path.join(output, "gallery.png"), fullPage: true });
    } catch (error) {
      await page.screenshot({ path: path.join(output, "failure.png"), fullPage: true });
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
      await browser.close();
      await server.close();
      await cp(root, path.join(output, "project"), { recursive: true });
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify(
          {
            authored,
            config,
            observations,
            errors,
            hashes: Object.fromEntries(
              Object.entries(authored).map(([side, bytes]) => [
                side,
                createHash("sha256").update(bytes).digest("hex"),
              ]),
            ),
          },
          null,
          2,
        ),
      );
      await rm(root, { recursive: true, force: true });
    }
  },
);
