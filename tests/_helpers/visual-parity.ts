import { expect, type Page } from "@playwright/test";
import * as fs from "node:fs";
import * as path from "node:path";
import pixelmatch from "pixelmatch";
import { PNG } from "pngjs";

export interface VisualParityOptions {
  name: string;
  outputDir: string;
  maxDiffPixels?: number;
  maxDiffRatio?: number;
  pixelThreshold?: number;
  fullPage?: boolean;
}

interface PngCompareResult {
  diffPixels: number;
  diffRatio: number;
  height: number;
  totalPixels: number;
  width: number;
}

interface ImageDimensions {
  height: number;
  width: number;
}

export interface VisualStabilityStyleOptions {
  css: string;
  sheetKey: string;
  styleId: string;
}

const DEFAULT_MAX_DIFF_RATIO = 0.002;
const DEFAULT_PIXEL_THRESHOLD = 0.1;
const VISUAL_STABILITY_SHEET_KEY = "__vizeVisualStabilitySheet";
const VISUAL_STABILITY_STYLE_ID = "vize-visual-stability";
export const VISUAL_STABILITY_CSS = `
  *, *::before, *::after {
    animation-delay: 0s !important;
    animation-duration: 0s !important;
    caret-color: transparent !important;
    scroll-behavior: auto !important;
    transition-delay: 0s !important;
    transition-duration: 0s !important;
  }
`;

export async function installVisualStabilityHooks(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const fixedNow = new Date("2026-01-01T00:00:00.000Z").valueOf();
    Object.defineProperty(Date, "now", { value: () => fixedNow });
    Object.defineProperty(Math, "random", { value: () => 0.42 });
    Object.defineProperty(navigator, "userAgent", {
      configurable: true,
      get: () => "Chromatic Playwright",
    });
  });
}

// Runs inside the page: keep it self-contained so Playwright can serialize it,
// and avoid `addStyleTag` so strict app CSPs cannot block the stability CSS.
export function applyVisualStabilityStyles({
  css,
  sheetKey,
  styleId,
}: VisualStabilityStyleOptions): void {
  if ("adoptedStyleSheets" in document && typeof CSSStyleSheet !== "undefined") {
    const existing = Reflect.get(window, sheetKey);
    const sheet = existing instanceof CSSStyleSheet ? existing : new CSSStyleSheet();
    sheet.replaceSync(css);
    if (!document.adoptedStyleSheets.includes(sheet)) {
      document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet];
    }
    Reflect.set(window, sheetKey, sheet);
    return;
  }

  let style = document.getElementById(styleId);
  if (!(style instanceof HTMLStyleElement)) {
    style = document.createElement("style");
    style.id = styleId;
    document.head.append(style);
  }
  style.textContent = css;
}

export async function prepareStableVisualState(page: Page): Promise<void> {
  await page.evaluate(applyVisualStabilityStyles, {
    css: VISUAL_STABILITY_CSS,
    sheetKey: VISUAL_STABILITY_SHEET_KEY,
    styleId: VISUAL_STABILITY_STYLE_ID,
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

  const viewportWidths = [
    referencePage.viewportSize()?.width,
    candidatePage.viewportSize()?.width,
  ].filter((width): width is number => width !== undefined);
  const viewportWidth = viewportWidths.length > 0 ? Math.min(...viewportWidths) : undefined;
  const result = comparePngBuffers(referenceBuffer, candidateBuffer, diffPath, {
    threshold: options.pixelThreshold ?? DEFAULT_PIXEL_THRESHOLD,
    viewportWidth,
  });
  const maxDiffRatio = options.maxDiffRatio ?? DEFAULT_MAX_DIFF_RATIO;
  const message = [
    `${options.name} visual diff ratio ${result.diffRatio}`,
    `diffPixels=${result.diffPixels}/${result.totalPixels}`,
    `size=${result.width}x${result.height}`,
    `maxDiffRatio=${maxDiffRatio}`,
    options.maxDiffPixels == null ? null : `maxDiffPixels=${options.maxDiffPixels}`,
    `artifacts=${outputDir}`,
  ]
    .filter((part): part is string => part != null)
    .join(" ");

  expect(
    visualDiffWithinBudget(result, {
      maxDiffPixels: options.maxDiffPixels,
      maxDiffRatio,
    }),
    message,
  ).toBe(true);
}

export function visualDiffWithinBudget(
  result: Pick<PngCompareResult, "diffPixels" | "diffRatio">,
  options: { maxDiffPixels?: number; maxDiffRatio?: number } = {},
): boolean {
  const maxDiffRatio = options.maxDiffRatio ?? DEFAULT_MAX_DIFF_RATIO;
  return (
    result.diffRatio <= maxDiffRatio ||
    (options.maxDiffPixels != null && result.diffPixels <= options.maxDiffPixels)
  );
}

export function comparePngBuffers(
  referenceBuffer: Buffer,
  candidateBuffer: Buffer,
  diffPath: string,
  options: { threshold?: number; viewportWidth?: number } = {},
): PngCompareResult {
  const reference = PNG.sync.read(referenceBuffer);
  const candidate = PNG.sync.read(candidateBuffer);
  const { height, width } = visualComparisonDimensions(reference, candidate, options.viewportWidth);
  const diff = new PNG({ width, height });
  const referenceFrame = normalizePngFrame(reference, width, height);
  const candidateFrame = normalizePngFrame(candidate, width, height);
  const diffPixels = pixelmatch(
    referenceFrame.data,
    candidateFrame.data,
    diff.data,
    width,
    height,
    {
      includeAA: false,
      threshold: options.threshold ?? DEFAULT_PIXEL_THRESHOLD,
    },
  );

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

function normalizePngFrame(source: PNG, width: number, height: number): PNG {
  const frame = new PNG({ width, height });

  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const targetIdx = pixelIndex(width, x, y);
      if (x >= source.width || y >= source.height) {
        frame.data[targetIdx] = 0;
        frame.data[targetIdx + 1] = 0;
        frame.data[targetIdx + 2] = 0;
        frame.data[targetIdx + 3] = 0;
        continue;
      }

      const sourceIdx = pixelIndex(source.width, x, y);
      frame.data[targetIdx] = source.data[sourceIdx];
      frame.data[targetIdx + 1] = source.data[sourceIdx + 1];
      frame.data[targetIdx + 2] = source.data[sourceIdx + 2];
      frame.data[targetIdx + 3] = source.data[sourceIdx + 3];
    }
  }

  return frame;
}

export function visualComparisonDimensions(
  reference: ImageDimensions,
  candidate: ImageDimensions,
  viewportWidth?: number,
): ImageDimensions {
  return {
    // Full-page screenshots can include horizontal document overflow beyond the
    // shared browser viewport. Cap the common PNG width to the actual viewport
    // while retaining the full vertical extent so missing page content fails.
    height: Math.max(reference.height, candidate.height),
    width: Math.min(reference.width, candidate.width, viewportWidth ?? Number.POSITIVE_INFINITY),
  };
}

function pixelIndex(width: number, x: number, y: number): number {
  return (y * width + x) * 4;
}
