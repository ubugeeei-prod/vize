import type { ArtFileInfo, ViewportConfig } from "./types/index.js";
import { MuseaVrtRunner, normalizeVrtWorkerCount } from "./vrt.js";
import { observePreviewReady, waitForMountedPreview } from "./vrt/preview-ready.js";
import { processMuseaArtFile } from "./plugin/art-processing.js";
import { scanArtFiles } from "./utils.js";

export type MuseaTestTraceEntry = {
  kind: "step" | "action" | "assertion";
  name: string;
  status: "running" | "passed" | "failed";
  duration: number;
  target?: string;
  error?: string;
};

export type MuseaTestResult = {
  artPath: string;
  variantName: string;
  status: "passed" | "failed" | "skipped";
  duration: number;
  trace: MuseaTestTraceEntry[];
  error?: string;
  stack?: string;
};

export type MuseaTestRunnerOptions = {
  workers?: number;
  viewport?: ViewportConfig;
  previewBasePath?: string;
  previewUrls?: Record<string, Record<string, string>>;
  launchOptions?: { args?: string[]; headless?: boolean };
};

export type MuseaTestResultReporter = (result: MuseaTestResult) => void;

const DEFAULT_VIEWPORT: ViewportConfig = { width: 1280, height: 720, name: "test" };

export async function loadMuseaArtFiles(
  options: {
    root?: string;
    include?: string[];
    exclude?: string[];
  } = {},
): Promise<ArtFileInfo[]> {
  const root = options.root ?? process.cwd();
  const paths = await scanArtFiles(
    root,
    options.include ?? ["**/*.art.vue"],
    options.exclude ?? ["node_modules/**", "dist/**"],
  );
  const files = await Promise.all(
    paths.map((filePath) => processMuseaArtFile(filePath, { root, command: "build" })),
  );
  return files.filter((file): file is ArtFileInfo => file !== null);
}

/** Fast interaction-only runner shared by the CLI and Vitest adapter. */
export class MuseaTestRunner {
  private readonly browser: MuseaVrtRunner;
  private readonly workers: number;
  private readonly viewport: ViewportConfig;

  constructor(options: MuseaTestRunnerOptions = {}) {
    this.workers = normalizeVrtWorkerCount(options.workers);
    this.viewport = options.viewport ?? DEFAULT_VIEWPORT;
    this.browser = new MuseaVrtRunner({
      workers: this.workers,
      viewports: [this.viewport],
      previewBasePath: options.previewBasePath,
      previewUrls: options.previewUrls,
      launchOptions: options.launchOptions,
      capture: { waitForNetwork: false, waitForPreviewReady: true, settleTime: 0 },
    });
  }

  init(): Promise<void> {
    return this.browser.init();
  }

  close(): Promise<void> {
    return this.browser.close();
  }

  getWorkerCount(): number {
    return this.workers;
  }

  async runAllTests(
    artFiles: ArtFileInfo[],
    baseUrl: string,
    report?: MuseaTestResultReporter,
  ): Promise<MuseaTestResult[]> {
    const cases = artFiles.flatMap((art) =>
      art.variants
        .filter((variant) => Boolean(variant.play))
        .map((variant) => ({ art, variantName: variant.name })),
    );
    const results = Array.from<MuseaTestResult | undefined>({ length: cases.length });
    let next = 0;

    const worker = async () => {
      while (true) {
        const index = next++;
        const testCase = cases[index];
        if (!testCase) return;
        const result = await this.runTest(testCase.art, testCase.variantName, baseUrl);
        results[index] = result;
        report?.(result);
      }
    };
    await Promise.all(Array.from({ length: Math.min(this.workers, cases.length) }, () => worker()));
    return results.filter((result): result is MuseaTestResult => result !== undefined);
  }

  async runTest(art: ArtFileInfo, variantName: string, baseUrl: string): Promise<MuseaTestResult> {
    const variant = art.variants.find((candidate) => candidate.name === variantName);
    if (!variant?.play) {
      return {
        artPath: art.path,
        variantName,
        status: "skipped",
        duration: 0,
        trace: [],
      };
    }

    const startedAt = performance.now();
    const { page, context } = await this.browser.createPage(this.viewport);
    try {
      await observePreviewReady(page);
      await page.goto(this.browser.getPreviewUrl(baseUrl, art.path, variantName), {
        waitUntil: "load",
      });
      await waitForMountedPreview(page);
      await page.waitForSelector(".musea-variant", { timeout: 10_000 });
      const execution = await page.evaluate(async () => {
        const run = Reflect.get(window, "__museaRunPlay");
        if (typeof run !== "function") throw new Error("Musea play runtime is unavailable");
        try {
          return { ok: true as const, result: await run() };
        } catch (error) {
          return {
            ok: false as const,
            error: error instanceof Error ? error.message : String(error),
            stack: error instanceof Error ? error.stack : undefined,
            report:
              error && typeof error === "object" ? Reflect.get(error, "museaReport") : undefined,
          };
        }
      });
      if (!execution.ok) {
        return {
          artPath: art.path,
          variantName,
          status: "failed",
          duration: Math.round(performance.now() - startedAt),
          trace: execution.report?.trace ?? [],
          error: execution.error,
          stack: execution.stack,
        };
      }
      const result = execution.result;
      const report = result as {
        status?: "passed";
        duration?: number;
        report?: { trace?: MuseaTestTraceEntry[] };
      };
      return {
        artPath: art.path,
        variantName,
        status: "passed",
        duration: report.duration ?? Math.round(performance.now() - startedAt),
        trace: report.report?.trace ?? [],
      };
    } catch (error) {
      return {
        artPath: art.path,
        variantName,
        status: "failed",
        duration: Math.round(performance.now() - startedAt),
        trace: [],
        error: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined,
      };
    } finally {
      await context.close();
    }
  }
}

export function formatMuseaTestFailure(result: MuseaTestResult): string {
  const trace = result.trace
    .map(
      (entry) =>
        `  ${entry.status === "passed" ? "✓" : "✗"} ${entry.name}${entry.target ? ` · ${entry.target}` : ""}`,
    )
    .join("\n");
  return [`${result.artPath} › ${result.variantName}`, result.error, trace || undefined]
    .filter(Boolean)
    .join("\n");
}

type VitestTest = {
  (name: string, test: () => void | Promise<void>): void;
  concurrent?: VitestTest;
};

type VitestApi = {
  describe(name: string, factory: () => void): void;
  test: VitestTest;
  beforeAll(hook: () => void | Promise<void>): void;
  afterAll(hook: () => void | Promise<void>): void;
};

/** Register every authored play function as an ordinary Vitest test case. */
export function registerMuseaTests(
  vitest: VitestApi,
  options: MuseaTestRunnerOptions & { artFiles: ArtFileInfo[]; baseUrl: string; name?: string },
): void {
  const runner = new MuseaTestRunner(options);
  const registerTest = vitest.test.concurrent ?? vitest.test;
  vitest.describe(options.name ?? "Musea", () => {
    vitest.beforeAll(() => runner.init());
    vitest.afterAll(() => runner.close());
    for (const art of options.artFiles) {
      for (const variant of art.variants) {
        if (!variant.play) continue;
        registerTest(`${art.metadata.title} › ${variant.name}`, async () => {
          const result = await runner.runTest(art, variant.name, options.baseUrl);
          if (result.status === "failed") throw new Error(formatMuseaTestFailure(result));
        });
      }
    }
  });
}
