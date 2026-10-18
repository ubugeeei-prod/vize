import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { createServer as createTcpServer } from "node:net";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { chromium, type Locator } from "playwright";
import { PNG } from "pngjs";
import { createServer } from "vite";
import type { MuseaVrtOptions } from "./types/plugin.ts";
import {
  createPackedGalleryHttpObserver,
  drainPackedGalleryHttp,
  tracePackedGalleryHttp,
  type PackedGalleryHttpRecord,
} from "./packed-gallery-http.ts";
import type { VrtResult, VrtSummary } from "./vrt/types.ts";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));
const sha256 = (value: Uint8Array) => createHash("sha256").update(value).digest("hex");

interface CaptureResponse {
  success: boolean;
  summary: VrtSummary;
  results: Array<Omit<VrtResult, "viewport"> & { viewport: string }>;
  artifacts: { snapshotDir: string };
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

void test(
  "built gallery serves assets, previews and VRT at default and configured base paths",
  { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1", timeout: 300000 },
  async (t) => {
    const fixturePath = "tests/tooling/fixtures/musea/packed-gallery-base.json";
    const bytes = await readFile(path.join(repository, fixturePath));
    const fixture = JSON.parse(bytes.toString()) as {
      cases: string[];
      titles: string[];
      viewport: { name: string; width: number; height: number };
    };
    assert.deepEqual(fixture.cases, ["/__musea__", "/gallery/vrt", "/gallery/vrt/"]);
    assert.deepEqual(fixture.titles, ["Left", "Right"]);
    const entry = path.join(repository, "npm/builder/vite-musea/dist/index.mjs");
    const { musea } = (await import(pathToFileURL(entry).href)) as typeof import("./index.js");
    const compilerEntry = path.join(repository, "npm/builder/vite/dist/index.mjs");
    const { default: vize } = (await import(
      pathToFileURL(compilerEntry).href
    )) as typeof import("../../vite/src/index.ts");
    const index = path.join(path.dirname(entry), "gallery/index.html");
    const galleryBytes = await readFile(index);
    assert.match(galleryBytes.toString(), /(?:src|href)="\/__musea__\/assets\//u);
    const output = path.join(repository, "artifacts/musea-native-packed-gallery-base");
    await mkdir(output, { recursive: true });
    await writeFile(
      path.join(output, "source-custody.json"),
      JSON.stringify(
        {
          fixture: { path: fixturePath, sha256: sha256(bytes) },
          entries: await Promise.all(
            [entry, compilerEntry, index].map(async (file) => ({
              path: file,
              sha256: sha256(await readFile(file)),
            })),
          ),
          scope: "Actual built source package entries; not installed public-package acceptance",
        },
        null,
        2,
      ),
    );

    for (const [caseIndex, basePath] of fixture.cases.entries()) {
      await t.test(basePath, async (context) => {
        const releases: Array<() => void | Promise<void>> = [];
        context.after(async () => {
          const errors: unknown[] = [];
          for (const release of releases.reverse()) {
            try {
              await release();
            } catch (error) {
              errors.push(error);
            }
          }
          if (errors.length === 1) throw errors[0];
          if (errors.length > 1) throw new AggregateError(errors, "Packed gallery cleanup failed");
        });
        const root = await mkdtemp(path.join(os.tmpdir(), "musea-packed-base-"));
        releases.push(() => rm(root, { recursive: true, force: true }));
        const caseOutput = path.join(output, String(caseIndex));
        await mkdir(caseOutput, { recursive: true });
        const arts: string[] = [];
        const sourceRecords: unknown[] = [];
        for (const side of ["left", "right"]) {
          const source = `tests/tooling/fixtures/musea/snapshot-collision/${side}/Button.art.vue`;
          const target = path.join(root, "src", side, "Button.art.vue");
          await mkdir(path.dirname(target), { recursive: true });
          await cp(path.join(repository, source), target);
          const raw = await readFile(target);
          sourceRecords.push({ path: source, bytes: raw.toString(), sha256: sha256(raw) });
          arts.push(target);
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
            alias: {
              vue: fileURLToPath(import.meta.resolve("vue/dist/vue.runtime.esm-bundler.js")),
            },
          },
          plugins: [vize(), musea({ include: ["src/**/*.art.vue"], basePath, vrt: config })],
          server: { host: "127.0.0.1", port: await unusedPort(), strictPort: true },
        });
        releases.push(() => server.close());
        await server.listen();
        const address = server.httpServer!.address();
        assert.ok(address && typeof address !== "string");
        assert.equal(address.port, server.config.server.port);
        const origin = `http://127.0.0.1:${address.port}`;
        const normalized = basePath.replace(/\/+$/, "");
        const url = `${origin}${normalized}/`;
        const browser = await chromium.launch();
        releases.push(() => browser.close());
        const page = await browser.newPage();
        page.setDefaultTimeout(15000);
        const errors: string[] = [],
          consoleErrors: string[] = [],
          httpErrors: string[] = [],
          pending: Promise<void>[] = [];
        const responses: PackedGalleryHttpRecord[] = [];
        const observations: unknown[] = [];
        const httpLifecycle: unknown[] = [];
        let httpPhase = "initial";
        const httpTrace = await tracePackedGalleryHttp(page, httpLifecycle, () => httpPhase);
        releases.push(httpTrace.close);
        let previousPreview: Awaited<ReturnType<Locator["elementHandle"]>> | undefined;
        async function recordPreviewOwner(title: string) {
          const frame = await page.locator(".variant-card iframe").elementHandle();
          assert.ok(frame);
          releases.push(() => frame.dispose());
          const state = await frame.evaluate((element) => {
            const frame = element as HTMLIFrameElement;
            return {
              url: frame.src,
              documentURL: frame.contentDocument?.URL,
              connected: frame.isConnected,
              buttons: [...(frame.contentDocument?.querySelectorAll("button") ?? [])].map(
                (button) => button.textContent,
              ),
            };
          });
          const previousConnected = previousPreview
            ? await previousPreview.evaluate((frame) => frame.isConnected)
            : null;
          observations.push({
            phase: "preview-document-owner",
            title,
            previousConnected,
            ...state,
          });
          assert.equal(state.connected, true);
          assert.equal(
            state.url,
            `${origin}${normalized}/preview?art=${encodeURIComponent(arts[fixture.titles.indexOf(title)])}&variant=Default`,
          );
          assert.equal(state.documentURL, state.url);
          assert.deepEqual(state.buttons, [title]);
          if (previousPreview) assert.equal(previousConnected, false);
          previousPreview = frame;
        }
        page.on("pageerror", (error) => errors.push(String(error)));
        page.on("console", (message) => {
          if (message.type() === "error") consoleErrors.push(message.text());
        });
        page.on("requestfailed", (request) => {
          if (request.url().startsWith(origin + "/"))
            httpErrors.push(`${request.failure()?.errorText}: ${request.url()}`);
        });
        page.on(
          "response",
          createPackedGalleryHttpObserver({
            origin,
            caseOutput,
            responses,
            pending,
            httpErrors,
            sha256,
            onBodyCapture: httpTrace.capture,
          }),
        );
        let failure: unknown;
        const evidenceErrors: unknown[] = [];
        try {
          await page.goto(url);
          await page.locator(".art-item").filter({ hasText: "Left" }).waitFor();
          assert.deepEqual(
            await page.locator(".art-item .art-name").allTextContents(),
            fixture.titles,
          );
          const assets = await page
            .locator('script[src], link[rel="stylesheet"][href]')
            .evaluateAll((elements) =>
              elements
                .map((element) => element.getAttribute("src") ?? element.getAttribute("href"))
                .filter((value): value is string => Boolean(value?.startsWith("/"))),
            );
          const galleryAssets = assets.filter((asset) => asset !== "/@vite/client");
          assert.ok(galleryAssets.length >= 2);
          assert.ok(galleryAssets.every((asset) => asset.startsWith(`${normalized}/assets/`)));
          for (const title of fixture.titles) {
            httpPhase = `select-${title}`;
            await drainPackedGalleryHttp(pending);
            await page.locator(".art-item").filter({ hasText: title }).click();
            const preview = page.frameLocator(".variant-card iframe");
            await preview.getByRole("button", { name: title, exact: true }).waitFor();
            assert.equal(await preview.locator("museacomponent").count(), 0);
            await recordPreviewOwner(title);
          }
          httpPhase = "select-Left-again";
          await drainPackedGalleryHttp(pending);
          await page.locator(".art-item").filter({ hasText: "Left" }).click();
          await page
            .frameLocator(".variant-card iframe")
            .getByRole("button", { name: "Left", exact: true })
            .waitFor();
          await recordPreviewOwner("Left");
          await drainPackedGalleryHttp(pending);
          httpPhase = "VRT-tab";
          await page.getByRole("button", { name: "VRT", exact: true }).click();
          const token = await page.evaluate(() => {
            const value = crypto.randomUUID();
            Reflect.set(window, "__packedGalleryDocument", value);
            return value;
          });
          let initialPng: Buffer | undefined;
          for (const phase of ["new", "repeat"] as const) {
            httpPhase = `VRT-${phase}`;
            const [response] = await Promise.all([
              page.waitForResponse(
                (item) =>
                  item.url() === `${origin}${normalized}/api/run-vrt` &&
                  item.request().method() === "POST",
                { timeout: 120000 },
              ),
              page.getByRole("button", { name: "Run VRT", exact: true }).click(),
            ]);
            const raw = await response.body();
            await writeFile(path.join(caseOutput, `${phase}.json`), raw);
            assert.equal(response.status(), 200, raw.toString());
            const packet = JSON.parse(raw.toString()) as CaptureResponse;
            assert.equal(packet.success, true);
            assert.deepEqual(
              Object.fromEntries(
                (["total", "passed", "failed", "new", "skipped", "errors"] as const).map((key) => [
                  key,
                  packet.summary[key],
                ]),
              ),
              {
                total: 1,
                passed: phase === "repeat" ? 1 : 0,
                failed: 0,
                new: phase === "new" ? 1 : 0,
                skipped: 0,
                errors: 0,
              },
            );
            assert.equal(packet.results.length, 1);
            const result = packet.results[0];
            assert.equal(result.artPath, arts[0]);
            assert.equal(result.variantName, "Default");
            assert.equal(result.viewport, fixture.viewport.name);
            assert.equal(result.passed, true);
            assert.equal(Boolean(result.isNew), phase === "new");
            assert.equal(result.error, undefined);
            assert.equal(packet.artifacts.snapshotDir, path.join(root, "reviewed-baselines"));
            const png = await readFile(result.snapshotPath);
            const decoded = PNG.sync.read(png);
            assert.equal(decoded.width, fixture.viewport.width);
            assert.equal(decoded.height, fixture.viewport.height);
            await writeFile(path.join(caseOutput, `${phase}.png`), png);
            if (phase === "new") initialPng = png;
            else {
              assert.equal(result.diffPercentage, 0);
              assert.deepEqual(png, initialPng);
            }
            assert.equal(
              await page.evaluate(() => Reflect.get(window, "__packedGalleryDocument")),
              token,
            );
            observations.push({ phase, packet, pngSHA256: sha256(png) });
          }
          httpPhase = "before-physical-edit";
          const beforeEdit = await readFile(arts[0], "utf8");
          const changed = beforeEdit.replace("#0000ff", "#00ff00");
          assert.notEqual(changed, beforeEdit);
          await drainPackedGalleryHttp(pending);
          const reloaded = page.waitForEvent("load", { timeout: 15000 });
          httpPhase = "physical-edit";
          await writeFile(arts[0], changed);
          await reloaded;
          httpPhase = "physical-edit-loaded";
          await page.waitForLoadState("load");
          assert.equal(
            await page.evaluate(() => Reflect.get(window, "__packedGalleryDocument")),
            undefined,
          );
          httpPhase = "physical-edit-loaded-poll";
          await page.waitForFunction(
            async (url) => {
              const art = await (await fetch(url)).json();
              return art.variants[0].template.includes("#00ff00");
            },
            `${origin}${normalized}/api/arts/${encodeURIComponent(arts[0])}`,
          );
          httpPhase = "physical-edit-select-preview";
          await drainPackedGalleryHttp(pending);
          await page.locator(".art-item").filter({ hasText: "Left" }).click();
          const updated = page.frameLocator(".variant-card iframe").getByRole("button", {
            name: "Left",
            exact: true,
          });
          await updated.waitFor();
          assert.equal(
            await updated.evaluate((element) => getComputedStyle(element).backgroundColor),
            "rgb(0, 255, 0)",
          );
          assert.equal(
            await page.evaluate(() => Reflect.get(window, "__packedGalleryDocument")),
            undefined,
          );
          const scripts = await page
            .locator("script[src]")
            .evaluateAll((elements) => elements.map((element) => element.getAttribute("src")));
          assert.equal(scripts.filter((source) => source === "/@vite/client").length, 1);
          const green = await updated.screenshot();
          await writeFile(path.join(caseOutput, "physical-edit.png"), green);
          const pixels = PNG.sync.read(green);
          let greenPixels = 0;
          for (let index = 0; index < pixels.data.length; index += 4) {
            if (
              pixels.data[index] === 0 &&
              pixels.data[index + 1] === 255 &&
              pixels.data[index + 2] === 0 &&
              pixels.data[index + 3] === 255
            )
              greenPixels++;
          }
          assert.ok(
            greenPixels > 2000,
            "the authored physical edit must reach the real packed preview",
          );
          observations.push({
            phase: "physical-edit-reload",
            source: arts[0],
            before: beforeEdit,
            after: changed,
            beforeSha256: sha256(Buffer.from(beforeEdit)),
            afterSha256: sha256(Buffer.from(changed)),
            url: page.url(),
            scripts,
            greenPixels,
            pngSHA256: sha256(green),
          });
          await drainPackedGalleryHttp(pending);
          assert.deepEqual(errors, []);
          assert.deepEqual(consoleErrors, []);
          assert.deepEqual(httpErrors, []);
          assert.ok(responses.length > 0);
          for (const asset of galleryAssets) {
            const physical = await readFile(
              path.join(path.dirname(index), asset.slice(normalized.length + 1)),
            );
            const response = responses.find((item) => item.url === origin + asset);
            assert.ok(response, asset);
            assert.equal(response.status, 200);
            const expected =
              normalized === "/__musea__"
                ? physical
                : Buffer.from(physical.toString().replaceAll("/__musea__/", `${normalized}/`));
            assert.equal(response.sha256, sha256(expected));
            assert.equal(response.bytes, expected.length);
          }
          await page.screenshot({ path: path.join(caseOutput, "gallery.png"), fullPage: true });
        } catch (error) {
          failure = error;
          throw error;
        } finally {
          const evidence = [
            async () => {
              await drainPackedGalleryHttp(pending);
            },
            () =>
              writeFile(
                path.join(caseOutput, "observations.json"),
                JSON.stringify(
                  {
                    basePath,
                    normalized,
                    sourceRecords,
                    config,
                    observations,
                    errors,
                    consoleErrors,
                    httpErrors,
                    responses,
                    httpLifecycle,
                    failure: failure
                      ? {
                          error:
                            failure instanceof Error ? failure.message : JSON.stringify(failure),
                          stack: failure instanceof Error ? failure.stack : undefined,
                        }
                      : undefined,
                  },
                  null,
                  2,
                ),
              ),
            async () => writeFile(path.join(caseOutput, "gallery.html"), await page.content()),
            () => cp(root, path.join(caseOutput, "project"), { recursive: true }),
          ];
          for (const save of evidence) {
            try {
              await save();
            } catch (error) {
              evidenceErrors.push(error);
            }
          }
        }
        if (evidenceErrors.length)
          throw new AggregateError(evidenceErrors, "Packed gallery evidence failed");
        assert.deepEqual(errors, []);
        assert.deepEqual(consoleErrors, []);
        assert.deepEqual(httpErrors, []);
      });
    }
  },
);
