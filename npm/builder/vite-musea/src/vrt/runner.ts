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
import { buildVariantUrl, computeSummary } from "./utils.js";

export type { VrtResult, VrtSummary, ExtendedVrtOptions, PixelCompareOptions } from "./types.js";

import type { VrtResult, VrtSummary, ExtendedVrtOptions } from "./types.js";
import { createVrtJobs, normalizeVrtWorkerCount, runJobsWithWorkers } from "./jobs.js";
import { SnapshotIndex } from "./snapshot-index.js";
import { resolveSnapshotIdentity } from "./snapshot-identity.js";
export { normalizeVrtWorkerCount } from "./jobs.js";

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
  private snapshotIndex: SnapshotIndex | undefined;
  private identityOptions: ExtendedVrtOptions;
  private previewUrls: ExtendedVrtOptions["previewUrls"];

  constructor(options: ExtendedVrtOptions = {}) {
    this.identityOptions = {
      ...options,
      projectRoot: path.resolve(options.projectRoot ?? process.cwd()),
      snapshotIdentities: options.snapshotIdentities
        ? { ...options.snapshotIdentities }
        : undefined,
    };
    this.options = {
      previewBasePath: options.previewBasePath ?? "/__musea__",
      snapshotDir: options.snapshotDir ?? ".vize/snapshots",
      threshold: options.threshold ?? 0.1,
      viewports: options.viewports ?? [
        { width: 1280, height: 720, name: "desktop" },
        { width: 375, height: 667, name: "mobile" },
      ],
      workers: normalizeVrtWorkerCount(options.workers),
    };
    this.capture = {
      animations: options.capture?.animations ?? "disabled",
      reducedMotion: options.capture?.reducedMotion ?? "no-preference",
      caret: options.capture?.caret ?? "hide",
      fullPage: options.capture?.fullPage ?? false,
      waitForNetwork: options.capture?.waitForNetwork ?? true,
      waitForPreviewReady: options.capture?.waitForPreviewReady ?? false,
      settleTime: options.capture?.settleTime ?? 100,
      waitSelector: options.capture?.waitSelector ?? ".musea-variant",
      hideElements: options.capture?.hideElements ?? [],
      maskElements: options.capture?.maskElements ?? [],
    };
    this.comparison = options.comparison ?? {};
    this.ci = options.ci ?? {};
    this.previewUrls = options.previewUrls;
  }

  /** Resolve the same actual preview for screenshots and accessibility audits. */
  getPreviewUrl(baseUrl: string, artPath: string, variantName: string): string {
    if (this.previewUrls !== undefined) {
      const variants = Object.hasOwn(this.previewUrls, artPath)
        ? this.previewUrls[artPath]
        : undefined;
      const url =
        variants && Object.hasOwn(variants, variantName) ? variants[variantName] : undefined;
      if (!url) throw new Error(`Hosted preview not found: ${artPath}/${variantName}`);
      return url;
    }
    return buildVariantUrl(baseUrl, artPath, variantName, this.options.previewBasePath);
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

  /** @internal Resolve and reserve a baseline before navigation or PNG writes. */
  async getSnapshotName(
    art: ArtFileInfo,
    variantName: string,
    viewport: ViewportConfig,
  ): Promise<string> {
    const index = await this.getSnapshotIndex();
    const names = await index.plan([{ art, variantName, viewport }]);
    return names.values().next().value!;
  }

  private async getSnapshotIndex(): Promise<SnapshotIndex> {
    this.snapshotIndex ??= await SnapshotIndex.open(this.options.snapshotDir, this.identityOptions);
    return this.snapshotIndex;
  }

  /**
   * Close browser and cleanup.
   */
  async close(): Promise<void> {
    await this.snapshotIndex?.close();
    this.snapshotIndex = undefined;
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

    const retries = this.ci.retries ?? 0;
    const jobs = createVrtJobs(artFiles, this.options.viewports);
    await (await this.getSnapshotIndex()).plan(jobs);

    return runJobsWithWorkers(jobs, this.options.workers, async (job) => {
      let result: VrtResult | null = null;
      let attempts = 0;

      while (attempts <= retries) {
        result = await this.captureAndCompare(job.art, job.variantName, job.viewport, baseUrl);
        if (result.passed || result.isNew || !result.error) {
          break;
        }
        attempts++;
        if (attempts <= retries) {
          console.log(
            `[vrt] Retry ${attempts}/${retries}: ${path.basename(job.art.path)}/${job.variantName}`,
          );
        }
      }

      return result;
    });
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
      reducedMotion: this.capture.reducedMotion,
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
      if (result.error) continue;
      const art = { path: result.artPath } as ArtFileInfo;
      const name = await this.getSnapshotName(art, result.variantName, result.viewport);
      if (path.resolve(result.snapshotPath) !== path.resolve(snapshotDir, name))
        throw new Error("Baseline path does not match its snapshot owner");
      const currentPath =
        result.currentPath ?? path.join(currentDir, path.basename(result.snapshotPath));

      if (await fileExists(currentPath)) {
        await fs.promises.mkdir(path.dirname(result.snapshotPath), { recursive: true });
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
    const candidates = results.filter((result) => !result.passed && !result.error);
    const toApprove = pattern
      ? candidates.filter((result) => {
          const identity = resolveSnapshotIdentity(
            result.artPath,
            this.identityOptions.projectRoot,
            this.identityOptions.snapshotIdentities,
          );
          const relativeName = `${identity.replace(/\.art\.vue$/, "")}/${result.variantName}`;
          const legacyName = `${path.basename(result.artPath, ".art.vue")}/${result.variantName}`;
          return (
            relativeName.includes(pattern) ||
            matchGlob(relativeName, pattern) ||
            legacyName.includes(pattern) ||
            matchGlob(legacyName, pattern)
          );
        })
      : candidates;
    if (pattern) {
      const legacyMatches = results.filter((result) => {
        const name = `${path.basename(result.artPath, ".art.vue")}/${result.variantName}`;
        return name.includes(pattern) || matchGlob(name, pattern);
      });
      if (new Set(legacyMatches.map((result) => result.artPath)).size > 1)
        throw new Error(
          `Ambiguous approval pattern ${pattern}; use a project-relative Art path: ${legacyMatches.map((result) => resolveSnapshotIdentity(result.artPath, this.identityOptions.projectRoot, this.identityOptions.snapshotIdentities)).join(", ")}`,
        );
    }

    return this.updateBaselines(toApprove);
  }

  /**
   * Clean orphaned snapshots (no corresponding art/variant).
   */
  async cleanOrphans(artFiles: ArtFileInfo[]): Promise<number> {
    const index = await this.getSnapshotIndex();
    const names = await index.plan(createVrtJobs(artFiles, this.options.viewports));
    const validNames = new Set(names.values());
    const files = await fs.promises.readdir(this.options.snapshotDir);
    let cleaned = 0;
    for (const file of files) {
      if (file.endsWith(".png") && !validNames.has(file)) {
        await fs.promises.unlink(path.join(this.options.snapshotDir, file));
        cleaned++;
        console.log(`[vrt] Cleaned: ${file}`);
      }
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
