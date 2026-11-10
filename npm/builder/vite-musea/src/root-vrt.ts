import type { Browser, BrowserContext, Page } from "playwright";
import type {
  ArtFileInfo,
  CaptureConfig,
  ComparisonConfig,
  ViewportConfig,
  VrtOptions,
} from "./types/index.js";
import type { MuseaVrtRunner as RuntimeMuseaVrtRunner } from "./vrt/runner.js";
import type { ExtendedVrtOptions, VrtResult, VrtSummary } from "./vrt/types.js";
import { resolveDiffStyle } from "./vrt/diff-style-option.js";
import { buildVariantUrl, computeSummary } from "./vrt/utils.js";
import { normalizeVrtWorkerCount } from "./vrt/jobs.js";

type ResolvedCaptureConfig = Required<Omit<CaptureConfig, "beforeCapture">> &
  Pick<CaptureConfig, "beforeCapture">;

/**
 * Backward-compatible root export whose implementation is loaded on first VRT
 * operation. New code can continue to import the dedicated `./vrt` subpath.
 */
export class MuseaVrtRunner {
  private readonly sourceOptions: ExtendedVrtOptions;
  private readonly options: Required<VrtOptions>;
  private readonly capture: ResolvedCaptureConfig;
  private readonly comparison: ComparisonConfig;
  private runtime: RuntimeMuseaVrtRunner | undefined;
  private loading: Promise<RuntimeMuseaVrtRunner> | undefined;

  constructor(options: ExtendedVrtOptions = {}) {
    resolveDiffStyle(options.comparison?.diffStyle);
    if (
      options.capture?.beforeCapture !== undefined &&
      typeof options.capture.beforeCapture !== "function"
    ) {
      throw new TypeError("Musea capture.beforeCapture must be a function.");
    }
    this.sourceOptions = options;
    this.options = {
      previewBasePath: options.previewBasePath ?? "/__musea__",
      snapshotDir: options.snapshotDir ?? ".vize/snapshots",
      threshold: options.threshold ?? 0.1,
      viewports: options.viewports ?? [
        { width: 1280, height: 720, name: "desktop" },
        { width: 375, height: 667, name: "mobile" },
      ],
      workers: normalizeVrtWorkerCount(options.workers),
      launchOptions: options.launchOptions ?? {},
    };
    this.capture = {
      beforeCapture: options.capture?.beforeCapture,
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
      maskColor: options.capture?.maskColor ?? "#ff00ff",
    };
    this.comparison = options.comparison ?? {};
  }

  private async loadRuntime(): Promise<RuntimeMuseaVrtRunner> {
    if (this.runtime) return this.runtime;
    const pending = (this.loading ??= import("./vrt.js").then(
      ({ MuseaVrtRunner: Runtime }) => new Runtime(this.sourceOptions),
    ));
    try {
      return (this.runtime = await pending);
    } finally {
      if (!this.runtime) this.loading = undefined;
    }
  }

  getPreviewUrl(baseUrl: string, artPath: string, variantName: string): string {
    if (this.sourceOptions.previewUrls !== undefined) {
      const variants = Object.hasOwn(this.sourceOptions.previewUrls, artPath)
        ? this.sourceOptions.previewUrls[artPath]
        : undefined;
      const url =
        variants && Object.hasOwn(variants, variantName) ? variants[variantName] : undefined;
      if (!url) throw new Error(`Hosted preview not found: ${artPath}/${variantName}`);
      return url;
    }
    return buildVariantUrl(baseUrl, artPath, variantName, this.options.previewBasePath);
  }

  getBrowser(): Browser | null {
    return this.runtime?.getBrowser() ?? null;
  }

  getOptions(): Required<VrtOptions> {
    return this.options;
  }

  getCapture(): ResolvedCaptureConfig {
    return this.capture;
  }

  getComparison(): ComparisonConfig {
    return this.comparison;
  }

  getHostedNavigation(): string | undefined {
    return this.sourceOptions.hostedNavigation;
  }

  async init(trust?: { hostedCertificateSpki?: string }): Promise<void> {
    return (await this.loadRuntime()).init(trust);
  }

  async getSnapshotName(
    art: ArtFileInfo,
    variantName: string,
    viewport: ViewportConfig,
  ): Promise<string> {
    return (await this.loadRuntime()).getSnapshotName(art, variantName, viewport);
  }

  async close(): Promise<void> {
    return this.runtime?.close();
  }

  async start(): Promise<void> {
    return this.init();
  }

  async stop(): Promise<void> {
    return this.close();
  }

  async runAllTests(artFiles: ArtFileInfo[], baseUrl: string): Promise<VrtResult[]> {
    return (await this.loadRuntime()).runAllTests(artFiles, baseUrl);
  }

  async runTests(
    artFiles: ArtFileInfo[],
    baseUrl: string,
    options?: { updateSnapshots?: boolean },
  ): Promise<VrtResult[]> {
    return (await this.loadRuntime()).runTests(artFiles, baseUrl, options);
  }

  async captureAndCompare(
    art: ArtFileInfo,
    variantName: string,
    viewport: ViewportConfig,
    baseUrl: string,
  ): Promise<VrtResult> {
    return (await this.loadRuntime()).captureAndCompare(art, variantName, viewport, baseUrl);
  }

  async createPage(viewport: ViewportConfig): Promise<{ page: Page; context: BrowserContext }> {
    return (await this.loadRuntime()).createPage(viewport);
  }

  async updateBaselines(results: VrtResult[]): Promise<number> {
    return (await this.loadRuntime()).updateBaselines(results);
  }

  async approveResults(results: VrtResult[], pattern?: string): Promise<number> {
    return (await this.loadRuntime()).approveResults(results, pattern);
  }

  async cleanOrphans(artFiles: ArtFileInfo[]): Promise<number> {
    return (await this.loadRuntime()).cleanOrphans(artFiles);
  }

  getSummary(results: VrtResult[]): VrtSummary {
    return this.runtime?.getSummary(results) ?? computeSummary(results, 0);
  }
}
