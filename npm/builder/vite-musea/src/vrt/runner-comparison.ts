/**
 * Screenshot capture and pixel comparison logic for VRT.
 *
 * Contains the captureAndCompare flow (browser context, navigation,
 * element hiding/masking, screenshot, baseline check) and the
 * pixel-level compareImages implementation.
 */

import type { BrowserContext, Page } from "playwright";
import type { ArtFileInfo, ViewportConfig } from "../types/index.js";
import fs from "node:fs";
import path from "node:path";
import { PNG } from "pngjs";

import { readPng, writePng, colorDelta, isAntiAliased, fileExists } from "./comparison.js";
import type { VrtResult } from "./types.js";
import type { MuseaVrtRunner } from "./runner.js";
import { buildVariantUrl } from "./utils.js";

/**
 * Capture screenshot and compare with baseline.
 *
 * Standalone function that operates on a MuseaVrtRunner instance.
 */
export async function captureAndCompare(
  runner: MuseaVrtRunner,
  art: ArtFileInfo,
  variantName: string,
  viewport: ViewportConfig,
  baseUrl: string,
): Promise<VrtResult> {
  const browser = runner.getBrowser();
  if (!browser) {
    throw new Error("VRT runner not initialized. Call init() first.");
  }

  const options = runner.getOptions();
  const capture = runner.getCapture();
  const comparison = runner.getComparison();

  const snapshotDir = options.snapshotDir;
  const artBaseName = path.basename(art.path, ".art.vue");
  const viewportName = viewport.name || `${viewport.width}x${viewport.height}`;
  const snapshotName = `${artBaseName}--${variantName}--${viewportName}.png`;
  const snapshotPath = path.join(snapshotDir, snapshotName);
  const currentPath = path.join(snapshotDir, "current", snapshotName);
  const diffPath = path.join(snapshotDir, "diff", snapshotName);

  // Ensure directories exist
  await fs.promises.mkdir(path.dirname(snapshotPath), { recursive: true });
  await fs.promises.mkdir(path.join(snapshotDir, "current"), { recursive: true });
  await fs.promises.mkdir(path.join(snapshotDir, "diff"), { recursive: true });

  let context: BrowserContext | null = null;
  let page: Page | null = null;

  try {
    context = await browser.newContext({
      viewport: {
        width: viewport.width,
        height: viewport.height,
      },
      deviceScaleFactor: viewport.deviceScaleFactor ?? 1,
    });
    page = await context.newPage();

    // Navigate to variant preview URL
    const variantUrl = buildVariantUrl(baseUrl, art.path, variantName);
    const waitUntil = capture.waitForNetwork ? ("networkidle" as const) : ("load" as const);
    await page.goto(variantUrl, { waitUntil });

    // Wait for content to render
    await page.waitForSelector(capture.waitSelector, { timeout: 10000 });

    // Additional wait for animations to settle
    await page.waitForTimeout(capture.settleTime);

    // Hide elements before capture
    if (capture.hideElements.length > 0) {
      for (const selector of capture.hideElements) {
        await page.evaluate((sel) => {
          document.querySelectorAll(sel).forEach((el) => {
            (el as HTMLElement).style.visibility = "hidden";
          });
        }, selector);
      }
    }

    // Mask elements before capture (replace with colored box)
    if (capture.maskElements.length > 0) {
      for (const selector of capture.maskElements) {
        await page.evaluate((sel) => {
          document.querySelectorAll(sel).forEach((el) => {
            const htmlEl = el as HTMLElement;
            htmlEl.style.background = "#ff00ff";
            htmlEl.style.color = "transparent";
            htmlEl.innerHTML = "";
          });
        }, selector);
      }
    }

    // Take screenshot
    await page.screenshot({
      path: currentPath,
      fullPage: capture.fullPage,
    });

    // Check if baseline exists
    const hasBaseline = await fileExists(snapshotPath);

    if (!hasBaseline) {
      // First run - save as baseline
      await fs.promises.copyFile(currentPath, snapshotPath);
      return {
        artPath: art.path,
        variantName,
        viewport,
        passed: true,
        snapshotPath,
        currentPath,
        isNew: true,
      };
    }

    // Compare images using pixel comparison
    const comparisonResult = await compareImages(snapshotPath, currentPath, diffPath, comparison);

    const passed = comparisonResult.diffPercentage <= options.threshold;

    return {
      artPath: art.path,
      variantName,
      viewport,
      passed,
      snapshotPath,
      currentPath,
      diffPath: passed ? undefined : diffPath,
      diffPercentage: comparisonResult.diffPercentage,
      diffPixels: comparisonResult.diffPixels,
      totalPixels: comparisonResult.totalPixels,
    };
  } catch (error) {
    return {
      artPath: art.path,
      variantName,
      viewport,
      passed: false,
      snapshotPath,
      error: error instanceof Error ? error.message : String(error),
    };
  } finally {
    if (page) await page.close();
    if (context) await context.close();
  }
}

/**
 * Compare two PNG images and generate a diff image.
 * Returns pixel difference statistics.
 */
export async function compareImages(
  baselinePath: string,
  currentPath: string,
  diffPath: string,
  comparison: {
    antiAliasing?: boolean;
    alpha?: boolean;
    diffColor?: { r: number; g: number; b: number };
  },
): Promise<{ diffPixels: number; totalPixels: number; diffPercentage: number }> {
  const baseline = await readPng(baselinePath);
  const current = await readPng(currentPath);

  // Handle size mismatch
  if (baseline.width !== current.width || baseline.height !== current.height) {
    const width = Math.max(baseline.width, current.width);
    const height = Math.max(baseline.height, current.height);
    const diff = new PNG({ width, height });

    // Fill with red to indicate size mismatch
    for (let i = 0; i < diff.data.length; i += 4) {
      diff.data[i] = 255; // R
      diff.data[i + 1] = 0; // G
      diff.data[i + 2] = 0; // B
      diff.data[i + 3] = 255; // A
    }

    await writePng(diff, diffPath);

    return {
      diffPixels: width * height,
      totalPixels: width * height,
      diffPercentage: 100,
    };
  }

  const width = baseline.width;
  const height = baseline.height;
  const totalPixels = width * height;
  const diff = new PNG({ width, height });

  const useAntiAliasing = comparison.antiAliasing ?? true;
  const useAlpha = comparison.alpha ?? true;
  const diffColor = comparison.diffColor ?? { r: 255, g: 0, b: 0 };

  // Pixel comparison
  let diffPixels = 0;
  const threshold = 0.1; // Color difference threshold

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const idx = (y * width + x) * 4;

      const r1 = baseline.data[idx];
      const g1 = baseline.data[idx + 1];
      const b1 = baseline.data[idx + 2];
      const a1 = useAlpha ? baseline.data[idx + 3] : 255;

      const r2 = current.data[idx];
      const g2 = current.data[idx + 1];
      const b2 = current.data[idx + 2];
      const a2 = useAlpha ? current.data[idx + 3] : 255;

      // Calculate color difference using YIQ color space
      const delta = colorDelta(r1, g1, b1, a1, r2, g2, b2, a2);

      if (delta > threshold * 255 * 255) {
        // Anti-aliasing detection: check if pixel is likely AA
        if (useAntiAliasing && isAntiAliased(baseline, current, x, y, width, height)) {
          // Mark as AA (yellow)
          diff.data[idx] = 255;
          diff.data[idx + 1] = 200;
          diff.data[idx + 2] = 0;
          diff.data[idx + 3] = 128;
        } else {
          // Mark as different
          diffPixels++;
          diff.data[idx] = diffColor.r;
          diff.data[idx + 1] = diffColor.g;
          diff.data[idx + 2] = diffColor.b;
          diff.data[idx + 3] = 255;
        }
      } else {
        // Grayscale for matching pixels
        const gray = Math.round((r2 + g2 + b2) / 3);
        diff.data[idx] = gray;
        diff.data[idx + 1] = gray;
        diff.data[idx + 2] = gray;
        diff.data[idx + 3] = 128; // Semi-transparent
      }
    }
  }

  // Only write diff if there are differences
  if (diffPixels > 0) {
    await writePng(diff, diffPath);
  }

  const diffPercentage = (diffPixels / totalPixels) * 100;

  return {
    diffPixels,
    totalPixels,
    diffPercentage,
  };
}
