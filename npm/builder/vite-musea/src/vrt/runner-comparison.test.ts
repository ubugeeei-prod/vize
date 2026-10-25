import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { PNG } from "pngjs";

import { readPng, writePng } from "./comparison.ts";
import { compareImages } from "./runner-comparison.ts";

void test("-t 0 detects a 96-level grey change with the default per-pixel threshold", async () => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vrt-grey-"));
  const baselinePath = path.join(workspace, "baseline.png");
  const currentPath = path.join(workspace, "current.png");
  const diffPath = path.join(workspace, "diff.png");
  await writePng(createSolidPng(1, 1, { r: 238, g: 238, b: 238 }), baselinePath);
  await writePng(createSolidPng(1, 1, { r: 142, g: 142, b: 142 }), currentPath);

  const allowedDiffPercentage = 0;
  const result = await compareImages(baselinePath, currentPath, diffPath, {});

  assert.equal(result.diffPixels, 1);
  assert.equal(result.diffPercentage > allowedDiffPercentage, true);
});

void test("colorThreshold controls per-pixel sensitivity independently", async () => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vrt-color-threshold-"));
  const baselinePath = path.join(workspace, "baseline.png");
  const currentPath = path.join(workspace, "current.png");
  await writePng(createSolidPng(1, 1, { r: 238, g: 238, b: 238 }), baselinePath);
  await writePng(createSolidPng(1, 1, { r: 237, g: 237, b: 237 }), currentPath);

  const exact = await compareImages(baselinePath, currentPath, path.join(workspace, "exact.png"), {
    colorThreshold: 0,
  });
  const tolerant = await compareImages(
    baselinePath,
    currentPath,
    path.join(workspace, "tolerant.png"),
    { colorThreshold: 0.1 },
  );

  assert.equal(exact.diffPixels, 1);
  assert.equal(tolerant.diffPixels, 0);
});

void test("anti-aliased pixels still count as visual diffs", async () => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vrt-aa-"));
  const baselinePath = path.join(workspace, "baseline.png");
  const currentPath = path.join(workspace, "current.png");
  const diffPath = path.join(workspace, "diff.png");
  const baseline = createSolidPng(3, 3, { r: 255, g: 255, b: 255 });
  const current = createSolidPng(3, 3, { r: 255, g: 255, b: 255 });

  setPixel(current, 1, 1, { r: 0, g: 0, b: 0 });
  setPixel(current, 1, 0, { r: 0, g: 0, b: 0 });
  await writePng(baseline, baselinePath);
  await writePng(current, currentPath);

  const result = await compareImages(baselinePath, currentPath, diffPath, { antiAliasing: true });

  assert.equal(result.diffPixels, 2);
  assert.equal(result.totalPixels, 9);
  assert.equal(await fileExists(diffPath), true);
});

void test("compareImages creates nested diff directories before writing", async () => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vrt-diff-dir-"));
  const baselinePath = path.join(workspace, "baseline.png");
  const currentPath = path.join(workspace, "current.png");
  const diffPath = path.join(workspace, "diff", "MyChip--Outlined%20%2F%20squared--desktop.png");
  const baseline = createSolidPng(1, 1, { r: 255, g: 255, b: 255 });
  const current = createSolidPng(1, 1, { r: 0, g: 0, b: 0 });

  await writePng(baseline, baselinePath);
  await writePng(current, currentPath);

  const result = await compareImages(baselinePath, currentPath, diffPath, {});

  assert.equal(result.diffPixels, 1);
  assert.equal(await fileExists(diffPath), true);
});

void test("identical decoded RGBA passes without creating a diff for every comparison option", async (t) => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vrt-identical-"));
  t.after(() => fs.promises.rm(workspace, { recursive: true, force: true }));
  const baseline = createSolidPng(3, 1, { r: 16, g: 32, b: 48 });
  for (const [pixel, alpha] of [0, 128, 255].entries()) baseline.data[pixel * 4 + 3] = alpha;
  const current = createSolidPng(3, 1, { r: 16, g: 32, b: 48 });
  baseline.data.copy(current.data);
  // Encoded metadata can differ while the existing decoded-pixel semantics agree.
  baseline.gamma = 0.45455;
  current.gamma = 1;
  const baselinePath = path.join(workspace, "baseline.png");
  const currentPath = path.join(workspace, "current.png");
  await writePng(baseline, baselinePath);
  await writePng(current, currentPath);
  assert.notDeepEqual(
    await fs.promises.readFile(baselinePath),
    await fs.promises.readFile(currentPath),
  );

  for (const alpha of [true, false]) {
    for (const antiAliasing of [true, false]) {
      for (const colorThreshold of [0, 0.1, 1]) {
        const diffPath = path.join(workspace, `${alpha}-${antiAliasing}-${colorThreshold}.png`);
        assert.deepEqual(
          await compareImages(baselinePath, currentPath, diffPath, {
            alpha,
            antiAliasing,
            colorThreshold,
            diffColor: { r: 10, g: 20, b: 30 },
          }),
          { diffPixels: 0, totalPixels: 3, diffPercentage: 0 },
        );
        assert.equal(await fileExists(diffPath), false);
      }
    }
  }
});

for (const changedPixel of [0, 2]) {
  void test(`a change in pixel ${changedPixel} preserves every byte of the complete diff`, async (t) => {
    const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vrt-one-pixel-"));
    t.after(() => fs.promises.rm(workspace, { recursive: true, force: true }));
    const baseline = createSolidPng(3, 1, { r: 10, g: 20, b: 30 });
    const current = createSolidPng(3, 1, { r: 10, g: 20, b: 30 });
    current.data[changedPixel * 4] = 200;
    const baselinePath = path.join(workspace, "baseline.png");
    const currentPath = path.join(workspace, "current.png");
    const diffPath = path.join(workspace, "diff.png");
    await writePng(baseline, baselinePath);
    await writePng(current, currentPath);

    assert.deepEqual(
      await compareImages(baselinePath, currentPath, diffPath, {
        antiAliasing: false,
        colorThreshold: 0,
        diffColor: { r: 7, g: 8, b: 9 },
      }),
      { diffPixels: 1, totalPixels: 3, diffPercentage: 33.33333333333333 },
    );
    const diff = await readPng(diffPath);
    const expected = Buffer.from([20, 20, 20, 128, 20, 20, 20, 128, 20, 20, 20, 128]);
    expected.set([7, 8, 9, 255], changedPixel * 4);
    assert.deepEqual(
      { width: diff.width, height: diff.height, data: diff.data },
      {
        width: 3,
        height: 1,
        data: expected,
      },
    );
  });
}

void test("matching RGBA bytes with different dimensions still produce a complete size-mismatch diff", async (t) => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vrt-shape-"));
  t.after(() => fs.promises.rm(workspace, { recursive: true, force: true }));
  const baselinePath = path.join(workspace, "baseline.png");
  const currentPath = path.join(workspace, "current.png");
  const diffPath = path.join(workspace, "diff.png");
  await writePng(createSolidPng(1, 2, { r: 10, g: 20, b: 30 }), baselinePath);
  await writePng(createSolidPng(2, 1, { r: 10, g: 20, b: 30 }), currentPath);
  assert.deepEqual(await compareImages(baselinePath, currentPath, diffPath, {}), {
    diffPixels: 4,
    totalPixels: 4,
    diffPercentage: 100,
  });
  const diff = await readPng(diffPath);
  assert.deepEqual(
    { width: diff.width, height: diff.height, data: diff.data },
    {
      width: 2,
      height: 2,
      data: Buffer.from([255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255]),
    },
  );
});

void test("nonidentical alpha bytes retain configured compositing and threshold behavior", async (t) => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vrt-alpha-"));
  t.after(() => fs.promises.rm(workspace, { recursive: true, force: true }));
  const baselinePath = path.join(workspace, "baseline.png");
  const currentPath = path.join(workspace, "current.png");
  const baseline = createSolidPng(1, 1, { r: 0, g: 0, b: 0 });
  const current = createSolidPng(1, 1, { r: 0, g: 0, b: 0 });
  current.data[3] = 0;
  await writePng(baseline, baselinePath);
  await writePng(current, currentPath);
  for (const [name, options, expectedPixels] of [
    ["alpha", { alpha: true, colorThreshold: 0 }, 1],
    ["ignore-alpha", { alpha: false, colorThreshold: 0 }, 0],
    ["tolerant", { alpha: true, colorThreshold: 1 }, 0],
  ] as const) {
    const diffPath = path.join(workspace, `${name}.png`);
    assert.deepEqual(await compareImages(baselinePath, currentPath, diffPath, options), {
      diffPixels: expectedPixels,
      totalPixels: 1,
      diffPercentage: expectedPixels * 100,
    });
    assert.equal(await fileExists(diffPath), expectedPixels > 0);
  }
});

void test("identical corrupt encoded PNGs still reject instead of taking an equality shortcut", async (t) => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vrt-corrupt-equal-"));
  t.after(() => fs.promises.rm(workspace, { recursive: true, force: true }));
  const baselinePath = path.join(workspace, "baseline.png");
  const currentPath = path.join(workspace, "current.png");
  const diffPath = path.join(workspace, "diff.png");
  await fs.promises.writeFile(baselinePath, "not a PNG");
  await fs.promises.copyFile(baselinePath, currentPath);
  await assert.rejects(compareImages(baselinePath, currentPath, diffPath, {}));
  assert.equal(await fileExists(diffPath), false);
});

function createSolidPng(
  width: number,
  height: number,
  color: { r: number; g: number; b: number },
): PNG {
  const png = new PNG({ width, height });
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      setPixel(png, x, y, color);
    }
  }
  return png;
}

function setPixel(png: PNG, x: number, y: number, color: { r: number; g: number; b: number }) {
  const idx = (y * png.width + x) * 4;
  png.data[idx] = color.r;
  png.data[idx + 1] = color.g;
  png.data[idx + 2] = color.b;
  png.data[idx + 3] = 255;
}

async function fileExists(filePath: string): Promise<boolean> {
  try {
    await fs.promises.access(filePath);
    return true;
  } catch {
    return false;
  }
}
