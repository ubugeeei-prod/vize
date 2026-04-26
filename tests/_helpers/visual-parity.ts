import { expect, type Page } from "@playwright/test";
import * as fs from "node:fs";
import * as path from "node:path";
import { PNG } from "pngjs";

export interface VisualParityOptions {
  name: string;
  outputDir: string;
  maxDiffRatio?: number;
  channelThreshold?: number;
  fullPage?: boolean;
}

interface PngCompareResult {
  diffPixels: number;
  diffRatio: number;
  height: number;
  totalPixels: number;
  width: number;
}

const DEFAULT_CHANNEL_THRESHOLD = 16;
const DEFAULT_MAX_DIFF_RATIO = 0.002;

export async function installVisualStabilityHooks(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const fixedNow = new Date("2026-01-01T00:00:00.000Z").valueOf();
    Object.defineProperty(Date, "now", { value: () => fixedNow });
    Object.defineProperty(Math, "random", { value: () => 0.42 });
  });
}

export async function prepareStableVisualState(page: Page): Promise<void> {
  await page.addStyleTag({
    content: `
      *, *::before, *::after {
        animation-delay: 0s !important;
        animation-duration: 0s !important;
        caret-color: transparent !important;
        scroll-behavior: auto !important;
        transition-delay: 0s !important;
        transition-duration: 0s !important;
      }
    `,
  });

  await page.evaluate(async () => {
    window.scrollTo(0, 0);
    await document.fonts?.ready;
  });
}

export async function expectVisualParity(
  referencePage: Page,
  candidatePage: Page,
  options: VisualParityOptions,
): Promise<void> {
  const name = options.name.replace(/[^a-z0-9._-]+/gi, "-").replace(/^-|-$/g, "");
  const outputDir = options.outputDir;
  fs.mkdirSync(outputDir, { recursive: true });

  const referenceBuffer = await referencePage.screenshot({
    animations: "disabled",
    fullPage: options.fullPage ?? true,
    scale: "css",
  });
  const candidateBuffer = await candidatePage.screenshot({
    animations: "disabled",
    fullPage: options.fullPage ?? true,
    scale: "css",
  });

  const referencePath = path.join(outputDir, `${name}-reference.png`);
  const candidatePath = path.join(outputDir, `${name}-candidate.png`);
  const diffPath = path.join(outputDir, `${name}-diff.png`);

  fs.writeFileSync(referencePath, referenceBuffer);
  fs.writeFileSync(candidatePath, candidateBuffer);

  const result = comparePngBuffers(
    referenceBuffer,
    candidateBuffer,
    options.channelThreshold ?? DEFAULT_CHANNEL_THRESHOLD,
    diffPath,
  );
  const maxDiffRatio = options.maxDiffRatio ?? DEFAULT_MAX_DIFF_RATIO;
  const message = [
    `${options.name} visual diff ratio ${result.diffRatio}`,
    `diffPixels=${result.diffPixels}/${result.totalPixels}`,
    `size=${result.width}x${result.height}`,
    `artifacts=${outputDir}`,
  ].join(" ");

  expect(result.diffRatio, message).toBeLessThanOrEqual(maxDiffRatio);
}

function comparePngBuffers(
  referenceBuffer: Buffer,
  candidateBuffer: Buffer,
  channelThreshold: number,
  diffPath: string,
): PngCompareResult {
  const reference = PNG.sync.read(referenceBuffer);
  const candidate = PNG.sync.read(candidateBuffer);
  const width = Math.max(reference.width, candidate.width);
  const height = Math.max(reference.height, candidate.height);
  const diff = new PNG({ width, height });
  let diffPixels = 0;

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const diffIdx = pixelIndex(width, x, y);
      const hasReference = x < reference.width && y < reference.height;
      const hasCandidate = x < candidate.width && y < candidate.height;
      const refIdx = hasReference ? pixelIndex(reference.width, x, y) : -1;
      const candidateIdx = hasCandidate ? pixelIndex(candidate.width, x, y) : -1;
      const differs =
        !hasReference ||
        !hasCandidate ||
        channelDiff(reference, refIdx, candidate, candidateIdx) > channelThreshold;

      if (differs) {
        diffPixels++;
        diff.data[diffIdx] = 255;
        diff.data[diffIdx + 1] = 0;
        diff.data[diffIdx + 2] = 0;
        diff.data[diffIdx + 3] = 255;
        continue;
      }

      diff.data[diffIdx] = candidate.data[candidateIdx];
      diff.data[diffIdx + 1] = candidate.data[candidateIdx + 1];
      diff.data[diffIdx + 2] = candidate.data[candidateIdx + 2];
      diff.data[diffIdx + 3] = 160;
    }
  }

  fs.writeFileSync(diffPath, PNG.sync.write(diff));

  const totalPixels = width * height;
  return {
    diffPixels,
    diffRatio: totalPixels === 0 ? 0 : diffPixels / totalPixels,
    height,
    totalPixels,
    width,
  };
}

function channelDiff(reference: PNG, refIdx: number, candidate: PNG, candidateIdx: number): number {
  let max = 0;
  for (let i = 0; i < 4; i++) {
    const delta = Math.abs(reference.data[refIdx + i] - candidate.data[candidateIdx + i]);
    if (delta > max) {
      max = delta;
    }
  }
  return max;
}

function pixelIndex(width: number, x: number, y: number): number {
  return (y * width + x) * 4;
}
