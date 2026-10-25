import assert from "node:assert/strict";
import { createServer } from "node:http";
import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import test from "node:test";
import { PNG } from "pngjs";
import type { ArtFileInfo, DiffStyle } from "../types/index.ts";
import { SessionArtifacts } from "../cli/service/artifacts.ts";
import { generateVrtJsonReport, generateVrtReport } from "./report.ts";
import { MuseaVrtRunner } from "./runner.ts";

const browserTest = { skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1" };
const artifacts = fileURLToPath(
  new URL("../../../../../artifacts/musea-vrt-styles/", import.meta.url),
);

function expectedPng(style: DiffStyle) {
  const png = new PNG({ width: style === "sideBySide" ? 64 : 32, height: 16 });
  if (style === "sideBySide") {
    for (let index = 0; index < png.data.length; index += 4) png.data.set([10, 20, 30, 255], index);
    png.data.set([200, 20, 30, 255], png.data.length - 4);
  } else {
    png.data.set([7, 8, 9, 255], png.data.length - 4);
  }
  return png;
}

void test(
  "selected diff PNG styles survive capture, JSON, hosted artifact URLs, and HTML report consumers",
  browserTest,
  async () => {
    await rm(artifacts, { recursive: true, force: true });
    await mkdir(artifacts, { recursive: true });
    const session = new SessionArtifacts();
    let changed = false;
    let previewImage = "";
    const server = createServer((request, response) => {
      if (request.url?.startsWith("/artifacts/")) {
        if (!session.send(request.url, response)) {
          response.statusCode = 404;
          response.end();
        }
        return;
      }
      response.setHeader("Content-Type", "text/html");
      if (request.url === "/consumer") {
        response.end(`<img id="diff" src="${previewImage}">`);
        return;
      }
      response.end(
        `<style>html,body{margin:0;background:rgb(10,20,30)}.musea-variant{width:32px;height:16px}i{position:absolute;left:31px;top:15px;width:1px;height:1px;background:rgb(${changed ? 200 : 10},20,30)}</style><div class="musea-variant"><i></i></div>`,
      );
    });
    await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
    const address = server.address();
    assert.ok(address && typeof address !== "string");
    const base = `http://127.0.0.1:${address.port}`;
    const observations: unknown[] = [];
    try {
      for (const diffStyle of ["diffOnly", "sideBySide"] as const) {
        changed = false;
        const root = path.join(artifacts, diffStyle);
        const runner = new MuseaVrtRunner({
          projectRoot: root,
          snapshotDir: root,
          threshold: 0,
          capture: { waitForNetwork: false, settleTime: 0 },
          comparison: {
            diffStyle,
            colorThreshold: 0,
            antiAliasing: false,
            diffColor: { r: 7, g: 8, b: 9 },
          },
        });
        try {
          await runner.init();
          const art = { path: path.join(root, "Pixels.art.vue") } as ArtFileInfo;
          const viewport = { width: 32, height: 16, name: diffStyle };
          const capture = () => runner.captureAndCompare(art, "Default", viewport, base);
          const first = await capture();
          assert.deepEqual(first, {
            artPath: art.path,
            variantName: "Default",
            viewport,
            passed: true,
            snapshotPath: path.join(root, `Pixels--Default--${diffStyle}.png`),
            currentPath: path.join(root, "current", `Pixels--Default--${diffStyle}.png`),
            isNew: true,
            diffStyle,
          });
          const repeat = await capture();
          assert.equal(repeat.diffPixels, 0);
          assert.equal(repeat.diffStyle, diffStyle);
          assert.equal(repeat.diffPath, undefined);
          changed = true;
          const diff = await capture();
          assert.deepEqual(diff, {
            artPath: art.path,
            variantName: "Default",
            viewport,
            passed: false,
            snapshotPath: first.snapshotPath,
            currentPath: first.currentPath,
            diffPath: path.join(root, "diff", `Pixels--Default--${diffStyle}.png`),
            diffPixels: 1,
            totalPixels: 512,
            diffPercentage: 0.1953125,
            diffStyle,
          });
          assert.equal(runner.getBrowser()!.contexts().length, 0);
          const bytes = await readFile(diff.diffPath!);
          const png = PNG.sync.read(bytes);
          const expected = expectedPng(diffStyle);
          assert.deepEqual(
            { width: png.width, height: png.height, data: png.data },
            {
              width: expected.width,
              height: expected.height,
              data: expected.data,
            },
          );

          const summary = {
            total: 1,
            passed: 0,
            failed: 1,
            new: 0,
            skipped: 0,
            errors: 0,
            duration: 0,
          };
          const json = JSON.parse(generateVrtJsonReport([diff], summary));
          assert.deepEqual(json.results, [
            {
              art: "Pixels",
              artPath: art.path,
              variantName: "Default",
              snapshotPath: diff.snapshotPath,
              currentPath: diff.currentPath,
              diffPath: diff.diffPath,
              diffStyle,
              diffPixels: 1,
              totalPixels: 512,
              variant: "Default",
              viewport: diffStyle,
              status: "failed",
              diffPercentage: 0.1953125,
            },
          ]);
          const [hosted] = await session.results([diff]);
          assert.equal(hosted.diffStyle, diffStyle);
          assert.deepEqual(Object.keys(hosted.images).sort(), ["current", "diff", "snapshot"]);
          previewImage = hosted.images.diff!;
          const response = await fetch(`${base}${previewImage}`);
          assert.equal(response.status, 200);
          assert.equal(response.headers.get("Content-Type"), "image/png");
          assert.equal(response.headers.get("X-Content-Type-Options"), "nosniff");
          assert.deepEqual(Buffer.from(await response.arrayBuffer()), bytes);

          const page = await runner.getBrowser()!.newPage();
          try {
            await page.goto(`${base}/consumer`);
            await page.locator("#diff").evaluate((image) => (image as HTMLImageElement).decode());
            assert.deepEqual(
              await page.locator("#diff").evaluate((image) => ({
                width: (image as HTMLImageElement).naturalWidth,
                height: (image as HTMLImageElement).naturalHeight,
              })),
              { width: expected.width, height: expected.height },
            );
            const htmlPath = path.join(root, "report.html");
            const label = diffStyle === "sideBySide" ? "Baseline / Current" : "Diff only";
            await writeFile(htmlPath, generateVrtReport([diff], summary));
            await writeFile(path.join(root, "report.json"), generateVrtJsonReport([diff], summary));
            await page.goto(pathToFileURL(htmlPath).href);
            const image = page.getByAltText(label, { exact: true });
            await image.evaluate((element) => (element as HTMLImageElement).decode());
            assert.deepEqual(
              await image.evaluate((element) => ({
                width: (element as HTMLImageElement).naturalWidth,
                height: (element as HTMLImageElement).naturalHeight,
              })),
              { width: expected.width, height: expected.height },
            );
            assert.equal(await image.getAttribute("src"), `file://${diff.diffPath}`);
            observations.push({
              diffStyle,
              first,
              repeat,
              diff,
              hosted,
              contentType: "image/png",
              completePixels: true,
              reportLabel: label,
              geometry: { width: expected.width, height: expected.height },
            });
          } finally {
            await page.close();
          }
        } finally {
          await runner.close();
        }
      }
    } finally {
      await writeFile(
        path.join(artifacts, "observations.json"),
        `${JSON.stringify(observations, null, 2)}\n`,
      );
      await new Promise<void>((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      );
    }
  },
);
