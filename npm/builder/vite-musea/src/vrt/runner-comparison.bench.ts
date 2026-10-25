/**
 * Paired end-to-end PNG comparison benchmark, including decoding and diff I/O.
 * Run from the package with `node --import tsx src/vrt/runner-comparison.bench.ts
 * /absolute/baseline/src/vrt/runner-comparison.ts ./src/vrt/runner-comparison.ts`.
 * Add Node's --cpu-prof flag to inspect source-level work in both modules.
 */
import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { pathToFileURL } from "node:url";
import { PNG } from "pngjs";
import { writePng } from "./comparison.ts";
import type { compareImages } from "./runner-comparison.ts";

const modules = process.argv.slice(2);
assert.equal(modules.length, 2, "provide baseline and candidate comparison module paths");
const implementations = await Promise.all(
  modules.map(async (module) => {
    const imported = await import(pathToFileURL(path.resolve(module)).href);
    return imported.compareImages as typeof compareImages;
  }),
);
const root = await mkdtemp(path.join(os.tmpdir(), "musea-vrt-bench-"));
const iterations = 12;
const rows = [];
try {
  for (const [name, width, height, changed] of [
    ["identical-mobile", 375, 667, false],
    ["identical-hd", 1920, 1080, false],
    ["last-pixel-hd", 1920, 1080, true],
  ] as const) {
    const baseline = new PNG({ width, height });
    for (let index = 0; index < baseline.data.length; index += 4) {
      const pixel = index / 4;
      baseline.data[index] = pixel % 251;
      baseline.data[index + 1] = (pixel >>> 5) % 251;
      baseline.data[index + 2] = (pixel >>> 10) % 251;
      baseline.data[index + 3] = pixel % 3 === 0 ? 128 : 255;
    }
    const current = new PNG({ width, height });
    baseline.data.copy(current.data);
    if (changed) current.data[current.data.length - 4] ^= 255;
    const baselinePath = path.join(root, `${name}-baseline.png`);
    const currentPath = path.join(root, `${name}-current.png`);
    await writePng(baseline, baselinePath);
    await writePng(current, currentPath);
    const times: number[][] = [[], []];
    const results: Awaited<ReturnType<typeof compareImages>>[] = [];
    for (let iteration = -3; iteration < iterations; iteration++) {
      // Alternate order for every pair to reduce warm-cache/order bias.
      for (const implementation of iteration % 2 === 0 ? [0, 1] : [1, 0]) {
        const start = performance.now();
        results[implementation] = await implementations[implementation](
          baselinePath,
          currentPath,
          path.join(root, `${name}-diff-${implementation}.png`),
          { colorThreshold: 0, antiAliasing: false },
        );
        if (iteration >= 0) times[implementation].push(performance.now() - start);
      }
      assert.deepEqual(results[0], results[1]);
    }
    assert.equal(results[0].diffPixels, changed ? 1 : 0);
    if (changed) {
      assert.deepEqual(
        await readFile(path.join(root, `${name}-diff-0.png`)),
        await readFile(path.join(root, `${name}-diff-1.png`)),
      );
    }
    const medians = times.map((samples) => {
      const sorted = [...samples].sort((a, b) => a - b);
      return (sorted[sorted.length / 2 - 1] + sorted[sorted.length / 2]) / 2;
    });
    rows.push({
      name,
      width,
      height,
      baselineMs: medians[0],
      candidateMs: medians[1],
      speedup: medians[0] / medians[1],
      samplesMs: times,
      result: results[0],
    });
  }
  process.stdout.write(
    `${JSON.stringify({ node: process.version, platform: process.platform, arch: process.arch, iterations, rows }, null, 2)}\n`,
  );
} finally {
  await rm(root, { recursive: true, force: true });
}
