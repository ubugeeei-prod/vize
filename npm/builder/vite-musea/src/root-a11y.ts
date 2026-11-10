import type { LaunchOptions, Page } from "playwright";
import type { A11yOptions, A11yResult, ArtFileInfo } from "./types/index.js";
import type { A11ySummary, MuseaA11yRunner as RuntimeMuseaA11yRunner } from "./a11y/index.js";
import type { MuseaVrtRunner as RuntimeMuseaVrtRunner } from "./vrt/runner.js";
import type { MuseaVrtRunner } from "./root-vrt.js";
import {
  computeA11ySummary,
  generateA11yHtmlReport,
  generateA11yJsonReport,
} from "./a11y/report.js";

/** Backward-compatible root export that loads browser audit code on demand. */
export class MuseaA11yRunner {
  private runtime: RuntimeMuseaA11yRunner | undefined;
  private loading: Promise<RuntimeMuseaA11yRunner> | undefined;

  constructor(
    private readonly options: A11yOptions = {},
    private readonly launchOptions: LaunchOptions = {},
  ) {}

  private async loadRuntime(): Promise<RuntimeMuseaA11yRunner> {
    if (this.runtime) return this.runtime;
    const pending = (this.loading ??= import("./a11y/index.js").then(
      ({ MuseaA11yRunner: Runtime }) => new Runtime(this.options, this.launchOptions),
    ));
    try {
      return (this.runtime = await pending);
    } finally {
      if (!this.runtime) this.loading = undefined;
    }
  }

  async runAudits(
    artFiles: ArtFileInfo[],
    baseUrl: string,
    vrtRunner?: MuseaVrtRunner,
  ): Promise<A11yResult[]> {
    return (await this.loadRuntime()).runAudits(
      artFiles,
      baseUrl,
      vrtRunner as unknown as RuntimeMuseaVrtRunner,
    );
  }

  async auditPage(page: Page, artPath: string, variantName: string): Promise<A11yResult> {
    return (await this.loadRuntime()).auditPage(page, artPath, variantName);
  }

  getSummary(results: A11yResult[]): A11ySummary {
    return computeA11ySummary(results);
  }

  generateHtmlReport(results: A11yResult[]): string {
    return generateA11yHtmlReport(results, this.getSummary(results));
  }

  generateJsonReport(results: A11yResult[]): string {
    return generateA11yJsonReport(results);
  }
}
