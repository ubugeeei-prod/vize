/**
 * VRT runner using Playwright for browser automation.
 *
 * Manages browser lifecycle, screenshot capture, and baseline comparison
 * for visual regression testing of Musea art file variants.
 */

import type { Browser, BrowserContext, Page } from "playwright";
import type {
  ArtFileInfo,
  VrtOptions,
  ViewportConfig,
  CaptureConfig,
  ComparisonConfig,
  CiConfig,
} from "../types/index.js";
import fs from "node:fs";
import path from "node:path";

import { fileExists, matchGlob } from "./comparison.js";
import { captureAndCompare } from "./runner-comparison.js";
import { computeSummary } from "./utils.js";

export type { VrtResult, VrtSummary, ExtendedVrtOptions, PixelCompareOptions } from "./types.js";

import type { VrtResult, VrtSummary, ExtendedVrtOptions } from "./types.js";

/**
 * VRT runner using Playwright.
 */
export class MuseaVrtRunner {
  private options: Required<VrtOptions>;
  private capture: Required<CaptureConfig>;
  private comparison: ComparisonConfig;
  private ci: CiConfig;
  private browser: Browser | null = null;
  private startTime: number = 0;

  constructor(options: ExtendedVrtOptions = {}) {
    this.options = {
      snapshotDir: options.snapshotDir ?? ".vize/snapshots",
      threshold: options.threshold ?? 0.1,
      viewports: options.viewports ?? [
        { width: 1280, height: 720, name: "desktop" },
        { width: 375, height: 667, name: "mobile" },
      ],
    };
    this.capture = {
      fullPage: options.capture?.fullPage ?? false,
      waitForNetwork: options.capture?.waitForNetwork ?? true,
      settleTime: options.capture?.settleTime ?? 100,
      waitSelector: options.capture?.waitSelector ?? ".musea-variant",
      hideElements: options.capture?.hideElements ?? [],
      maskElements: options.capture?.maskElements ?? [],
    };
    this.comparison = options.comparison ?? {};
    this.ci = options.ci ?? {};
  }

  // --- Internal accessors used by runner-comparison ---

  /** @internal */
  getBrowser(): Browser | null {
    return this.browser;
  }

  /** @internal */
  getOptions(): Required<VrtOptions> {
    return this.options;
  }

  /** @internal */
  getCapture(): Required<CaptureConfig> {
    return this.capture;
  }

  /** @internal */
  getComparison(): ComparisonConfig {
    return this.comparison;
  }

  /**
   * Initialize Playwright browser.
   */
  async init(): Promise<void> {
    const { chromium } = await import("playwright");
    this.browser = await chromium.launch({ headless: true });
    this.startTime = Date.now();
  }

  /**
   * Close browser and cleanup.
   */
  async close(): Promise<void> {
    if (this.browser) {
      await this.browser.close();
      this.browser = null;
    }
  }

  /**
   * Alias for init() - used by the plugin API.
   */
  async start(): Promise<void> {
    return this.init();
  }

  /**
   * Alias for close() - used by the plugin API.
   */
  async stop(): Promise<void> {
    return this.close();
  }

  /**
   * Run VRT tests for all Art files.
   */
  async runAllTests(artFiles: ArtFileInfo[], baseUrl: string): Promise<VrtResult[]> {
    if (!this.browser) {
      throw new Error("VRT runner not initialized. Call init() first.");
    }

    const results: VrtResult[] = [];
    const retries = this.ci.retries ?? 0;

    for (const art of artFiles) {
      for (const variant of art.variants) {
        if (variant.skipVrt) {
          continue;
        }

        // Determine viewports: use per-variant viewport if defined, else global viewports
        const viewports = variant.args?.viewport
          ? [variant.args.viewport as ViewportConfig]
          : this.options.viewports;

        for (const viewport of viewports) {
          let result: VrtResult | null = null;
          let attempts = 0;

          while (attempts <= retries) {
            result = await this.captureAndCompare(art, variant.name, viewport, baseUrl);
            if (result.passed || result.isNew || !result.error) {
              break;
            }
            attempts++;
            if (attempts <= retries) {
              console.log(
                `[vrt] Retry ${attempts}/${retries}: ${path.basename(art.path)}/${variant.name}`,
              );
            }
          }

          if (result) {
            results.push(result);
          }
        }
      }
    }

    return results;
  }

  /**
   * Run VRT tests - alias used by the plugin API that accepts options.
   */
  async runTests(
    artFiles: ArtFileInfo[],
    baseUrl: string,
    _options?: { updateSnapshots?: boolean },
  ): Promise<VrtResult[]> {
    const results = await this.runAllTests(artFiles, baseUrl);
    if (_options?.updateSnapshots) {
      await this.updateBaselines(results);
    }
    return results;
  }

  /**
   * Capture screenshot and compare with baseline.
   */
  async captureAndCompare(
    art: ArtFileInfo,
    variantName: string,
    viewport: ViewportConfig,
    baseUrl: string,
  ): Promise<VrtResult> {
    return captureAndCompare(this, art, variantName, viewport, baseUrl);
  }

  /**
   * Get the Playwright Page for external use (e.g., a11y auditing).
   */
  async createPage(viewport: ViewportConfig): Promise<{ page: Page; context: BrowserContext }> {
    if (!this.browser) {
      throw new Error("VRT runner not initialized. Call init() first.");
    }
    const context = await this.browser.newContext({
      viewport: { width: viewport.width, height: viewport.height },
      deviceScaleFactor: viewport.deviceScaleFactor ?? 1,
    });
    const page = await context.newPage();
    return { page, context };
  }

  /**
   * Update baseline snapshots with current screenshots.
   */
  async updateBaselines(results: VrtResult[]): Promise<number> {
    let updated = 0;
    const snapshotDir = this.options.snapshotDir;
    const currentDir = path.join(snapshotDir, "current");

    for (const result of results) {
      const currentPath = path.join(currentDir, path.basename(result.snapshotPath));

      if (await fileExists(currentPath)) {
        await fs.promises.copyFile(currentPath, result.snapshotPath);
        updated++;
        console.log(`[vrt] Updated: ${path.basename(result.snapshotPath)}`);
      }
    }

    return updated;
  }

  /**
   * Approve specific failed results (update their baselines).
   */
  async approveResults(results: VrtResult[], pattern?: string): Promise<number> {
    const toApprove = pattern
      ? results.filter((r) => {
          const name = `${path.basename(r.artPath, ".art.vue")}/${r.variantName}`;
          return name.includes(pattern) || matchGlob(name, pattern);
        })
      : results.filter((r) => !r.passed && !r.error);

    return this.updateBaselines(toApprove);
  }

  /**
   * Clean orphaned snapshots (no corresponding art/variant).
   */
  async cleanOrphans(artFiles: ArtFileInfo[]): Promise<number> {
    const snapshotDir = this.options.snapshotDir;
    let cleaned = 0;

    try {
      const files = await fs.promises.readdir(snapshotDir);
      const validNames = new Set<string>();

      for (const art of artFiles) {
        const artBaseName = path.basename(art.path, ".art.vue");
        for (const variant of art.variants) {
          if (variant.skipVrt) continue;
          for (const viewport of this.options.viewports) {
            const viewportName = viewport.name || `${viewport.width}x${viewport.height}`;
            validNames.add(`${artBaseName}--${variant.name}--${viewportName}.png`);
          }
        }
      }

      for (const file of files) {
        if (file.endsWith(".png") && !validNames.has(file)) {
          await fs.promises.unlink(path.join(snapshotDir, file));
          cleaned++;
          console.log(`[vrt] Cleaned: ${file}`);
        }
      }
    } catch {
      // Directory may not exist yet
    }

    return cleaned;
  }

  /**
   * Get VRT summary statistics.
   */
  getSummary(results: VrtResult[]): VrtSummary {
    return computeSummary(results, this.startTime);
  }
}
