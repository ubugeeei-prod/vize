import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { PNG } from "pngjs";

import { writePng } from "./comparison.ts";
import { compareImages } from "./runner-comparison.ts";

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
