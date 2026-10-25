import assert from "node:assert/strict";
import { createServer } from "node:http";
import { mkdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import type { Page } from "playwright";
import { PNG } from "pngjs";
import type { ArtFileInfo } from "../types/index.ts";
import { MuseaVrtRunner } from "./runner.ts";

const browserTest = { skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1" };
const artifacts = fileURLToPath(
  new URL("../../../../../artifacts/musea-vrt-mask/", import.meta.url),
);
const image = `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="80" height="20"><rect width="80" height="20" fill="red"/></svg>')}`;

async function geometry(page: Page) {
  return page.evaluate(() => {
    const masked = document.querySelector("#masked")!;
    const after = document.querySelector("#after")!;
    const image = document.querySelector("#image")!;
    const box = masked.getBoundingClientRect();
    return {
      width: box.width,
      height: box.height,
      children: masked.childElementCount,
      content: masked.textContent,
      afterTop: after.getBoundingClientRect().top,
      imageTop: image.getBoundingClientRect().top,
    };
  });
}

function expectedPixels(scale: number, mask: readonly number[], afterColor: readonly number[]) {
  const png = new PNG({ width: 160 * scale, height: 120 * scale });
  for (let y = 0; y < png.height; y++) {
    for (let x = 0; x < png.width; x++) {
      const color =
        x < 80 * scale && y < 80 * scale
          ? y >= 40 * scale && y < 60 * scale
            ? afterColor
            : mask
          : [255, 255, 255];
      png.data.set([...color, 255], (y * png.width + x) * 4);
    }
  }
  return png;
}

void test(
  "VRT masks preserve DOM geometry, cover full rectangles, and retain unmasked diffs",
  browserTest,
  async () => {
    await rm(artifacts, { recursive: true, force: true });
    await mkdir(artifacts, { recursive: true });
    let content = "alpha";
    let afterColor = "#0000ff";
    const server = createServer((_request, response) => {
      response.setHeader("Content-Type", "text/html");
      response.end(
        `<style>html,body{margin:0;background:white}.musea-variant{width:80px}#masked{background-image:linear-gradient(red,blue)}#dynamic{height:40px;background:#00ff00}#after{height:20px;background:${afterColor}}img{display:block}</style><div class="musea-variant"><div id="masked"><div id="dynamic">${content}</div></div><div id="after"></div><img id="image" width="80" height="20" src="${image}"></div>`,
      );
    });
    await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
    const address = server.address();
    assert.ok(address && typeof address !== "string");
    const base = `http://127.0.0.1:${address.port}`;
    const observations: unknown[] = [];
    try {
      for (const [name, maskColor, color, scale] of [
        ["default", undefined, [255, 0, 255], 1],
        ["custom-retina", "#123456", [18, 52, 86], 2],
      ] as const) {
        content = "alpha";
        afterColor = "#0000ff";
        const root = path.join(artifacts, name);
        const runner = new MuseaVrtRunner({
          projectRoot: root,
          snapshotDir: root,
          threshold: 0,
          capture: {
            maskElements: ["#masked", "#image"],
            maskColor,
            waitForNetwork: false,
            settleTime: 0,
          },
          comparison: { colorThreshold: 0, antiAliasing: false },
        });
        const captured: unknown[] = [];
        observations.push({ name, captured });
        try {
          await runner.init();
          const browser = runner.getBrowser()!;
          const newContext = browser.newContext.bind(browser);
          browser.newContext = async (options) => {
            const context = await newContext(options);
            const newPage = context.newPage.bind(context);
            context.newPage = async () => {
              const page = await newPage();
              const screenshot = page.screenshot.bind(page);
              page.screenshot = async (options) => {
                const before = await geometry(page);
                const png = await screenshot(options);
                const after = await geometry(page);
                captured.push({ before, after });
                assert.deepEqual(before, {
                  width: 80,
                  height: 40,
                  children: 1,
                  content,
                  afterTop: 40,
                  imageTop: 60,
                });
                assert.deepEqual(after, before);
                return png;
              };
              return page;
            };
            return context;
          };
          const art = { path: path.join(root, "Masked.art.vue") } as ArtFileInfo;
          const viewport = { width: 160, height: 120, deviceScaleFactor: scale, name };
          const capture = () => runner.captureAndCompare(art, "Default", viewport, base);
          const first = await capture();
          assert.equal(first.passed, true, first.error);
          assert.equal(first.isNew, true);
          const png = PNG.sync.read(await readFile(first.currentPath!));
          const expected = expectedPixels(scale, color, [0, 0, 255]);
          assert.deepEqual(
            { width: png.width, height: png.height, data: png.data },
            {
              width: expected.width,
              height: expected.height,
              data: expected.data,
            },
          );
          content = "omega";
          const repeat = await capture();
          assert.deepEqual(repeat, {
            artPath: art.path,
            variantName: "Default",
            viewport,
            passed: true,
            snapshotPath: first.snapshotPath,
            currentPath: first.currentPath,
            diffPath: undefined,
            diffPercentage: 0,
            diffPixels: 0,
            totalPixels: expected.width * expected.height,
          });
          afterColor = "#00ff00";
          const diff = await capture();
          assert.deepEqual(diff, {
            artPath: art.path,
            variantName: "Default",
            viewport,
            passed: false,
            snapshotPath: first.snapshotPath,
            currentPath: first.currentPath,
            diffPath: path.join(root, "diff", path.basename(first.snapshotPath)),
            diffPixels: 80 * 20 * scale * scale,
            totalPixels: expected.width * expected.height,
            diffPercentage: 8.333333333333332,
          });
          const changed = PNG.sync.read(await readFile(diff.currentPath!));
          assert.deepEqual(changed.data, expectedPixels(scale, color, [0, 255, 0]).data);
          const expectedDiff = new PNG({ width: expected.width, height: expected.height });
          const maskGray = name === "default" ? 170 : 52;
          for (let y = 0; y < expected.height; y++) {
            for (let x = 0; x < expected.width; x++) {
              const highlighted = x < 80 * scale && y >= 40 * scale && y < 60 * scale;
              const gray = x < 80 * scale && y < 80 * scale ? maskGray : 255;
              expectedDiff.data.set(
                highlighted ? [255, 0, 0, 255] : [gray, gray, gray, 128],
                (y * expected.width + x) * 4,
              );
            }
          }
          const diffPng = PNG.sync.read(await readFile(diff.diffPath!));
          assert.deepEqual(
            { width: diffPng.width, height: diffPng.height, data: diffPng.data },
            {
              width: expected.width,
              height: expected.height,
              data: expectedDiff.data,
            },
          );
          assert.equal(browser.contexts().length, 0);
          observations.push({
            name,
            first,
            repeat,
            diff,
            completePixels: true,
            resourcesClosed: true,
          });
        } finally {
          await runner.close();
        }
      }

      for (const [name, selectors, maskColor, error] of [
        ["empty-selector", [""], "#ff00ff", /non-empty selectors/],
        ["invalid-selector", ["["], "#ff00ff", /selector/i],
        ["invalid-color", ["#masked"], "not-a-color", /Invalid VRT mask color/],
      ] as const) {
        const root = path.join(artifacts, name);
        const runner = new MuseaVrtRunner({
          projectRoot: root,
          snapshotDir: root,
          capture: {
            maskElements: [...selectors],
            maskColor,
            waitForNetwork: false,
            settleTime: 0,
          },
        });
        try {
          await runner.init();
          const artPath = path.join(root, "Invalid.art.vue");
          const viewport = { width: 160, height: 120, name };
          const result = await runner.captureAndCompare(
            { path: artPath } as ArtFileInfo,
            "Default",
            viewport,
            base,
          );
          assert.match(result.error!, error);
          assert.deepEqual(result, {
            artPath,
            variantName: "Default",
            viewport,
            passed: false,
            snapshotPath: path.join(root, `Invalid--Default--${name}.png`),
            error: result.error,
          });
          await assert.rejects(stat(result.snapshotPath), { code: "ENOENT" });
          assert.equal(runner.getBrowser()!.contexts().length, 0);
          observations.push({ name, result, noBaselineCreated: true, resourcesClosed: true });
        } finally {
          await runner.close();
        }
      }

      const unmatched: Buffer[] = [];
      for (const maskElements of [[], ["#does-not-exist"]]) {
        const root = path.join(artifacts, maskElements.length ? "unmatched" : "unmasked");
        const runner = new MuseaVrtRunner({
          projectRoot: root,
          snapshotDir: root,
          capture: { maskElements, waitForNetwork: false, settleTime: 0 },
        });
        try {
          await runner.init();
          const result = await runner.captureAndCompare(
            { path: path.join(root, "Control.art.vue") } as ArtFileInfo,
            "Default",
            { width: 160, height: 120 },
            base,
          );
          assert.equal(result.passed, true, result.error);
          unmatched.push(await readFile(result.currentPath!));
          observations.push({ maskElements, result });
        } finally {
          await runner.close();
        }
      }
      assert.deepEqual(unmatched[0], unmatched[1]);
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
