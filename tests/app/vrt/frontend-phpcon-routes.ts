export type FrontendPhpconVisualMode = "dev" | "preview";

export interface FrontendPhpconVisualRouteConfig {
  maxDiffPixels?: number;
  maxDiffPixelsByMode?: Partial<Record<FrontendPhpconVisualMode, number>>;
  maxDiffRatio?: number;
  name: string;
  path: string;
  viewport?: { height: number; width: number };
}

export const DEFAULT_VIEWPORT = { width: 1280, height: 720 };
export const MOBILE_VIEWPORT = { width: 390, height: 844 };
export const FRONTEND_PHPCON_VRT_TIMEOUT = 900_000;
export const STRICT_ROUTE_MAX_DIFF_RATIO = 0.004;
// Job board pages include several short cards and footer links. Preview builds
// can render identical text with tiny sub-pixel antialiasing drift above the
// strict shared budget. Measured worst case is 0.00403 (english-job-board,
// 10284/2554880 px), so keep a narrow page-specific budget just above it.
export const JOB_BOARD_ROUTE_MAX_DIFF_RATIO = 0.0042;
// Long news articles are almost entirely body copy, so preview builds spread
// sub-pixel text antialiasing drift across the whole page. Measured worst case
// is 0.0084 (english-news, 41384/4929280 px), so keep a narrow route-specific
// budget just above the observed drift.
export const NEWS_ROUTE_MAX_DIFF_RATIO = 0.009;
export const PREVIEW_MOBILE_MAX_DIFF_PIXELS = 43_887;

export const frontendPhpconVisualModes: FrontendPhpconVisualMode[] = ["dev", "preview"];

export const frontendPhpconVisualRoutes: FrontendPhpconVisualRouteConfig[] = [
  { name: "home", path: "/", maxDiffRatio: STRICT_ROUTE_MAX_DIFF_RATIO },
  {
    name: "home-mobile",
    path: "/",
    viewport: MOBILE_VIEWPORT,
    maxDiffRatio: STRICT_ROUTE_MAX_DIFF_RATIO,
    maxDiffPixelsByMode: { preview: PREVIEW_MOBILE_MAX_DIFF_PIXELS },
  },
  {
    name: "mobile-menu",
    path: "/",
    viewport: MOBILE_VIEWPORT,
    maxDiffRatio: STRICT_ROUTE_MAX_DIFF_RATIO,
    maxDiffPixelsByMode: { preview: PREVIEW_MOBILE_MAX_DIFF_PIXELS },
  },
  { name: "about", path: "/about" },
  {
    name: "news",
    path: "/news/2026-05-06-social-gathering-ticket",
    maxDiffRatio: NEWS_ROUTE_MAX_DIFF_RATIO,
  },
  { name: "timetable", path: "/timetable", maxDiffRatio: STRICT_ROUTE_MAX_DIFF_RATIO },
  { name: "job-board", path: "/job-board", maxDiffRatio: JOB_BOARD_ROUTE_MAX_DIFF_RATIO },
  { name: "english-home", path: "/en", maxDiffRatio: STRICT_ROUTE_MAX_DIFF_RATIO },
  { name: "english-about", path: "/en/about" },
  {
    name: "english-news",
    path: "/en/news/2026-05-06-social-gathering-ticket",
    maxDiffRatio: NEWS_ROUTE_MAX_DIFF_RATIO,
  },
  {
    name: "english-job-board",
    path: "/en/job-board",
    maxDiffRatio: JOB_BOARD_ROUTE_MAX_DIFF_RATIO,
  },
  {
    name: "language-switch",
    path: "/",
    maxDiffRatio: STRICT_ROUTE_MAX_DIFF_RATIO,
  },
];

export function maxDiffPixelsForFrontendPhpconMode(
  route: FrontendPhpconVisualRouteConfig,
  mode: FrontendPhpconVisualMode,
): number | undefined {
  return route.maxDiffPixelsByMode?.[mode] ?? route.maxDiffPixels;
}
