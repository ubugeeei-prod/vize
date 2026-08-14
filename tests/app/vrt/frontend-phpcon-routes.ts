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
export const NEWS_ROUTE_MAX_DIFF_RATIO = 0.008;
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
  { name: "job-board", path: "/job-board", maxDiffRatio: STRICT_ROUTE_MAX_DIFF_RATIO },
  { name: "english-home", path: "/en", maxDiffRatio: STRICT_ROUTE_MAX_DIFF_RATIO },
  { name: "english-about", path: "/en/about" },
  {
    name: "english-news",
    path: "/en/news/2026-05-06-social-gathering-ticket",
    maxDiffRatio: NEWS_ROUTE_MAX_DIFF_RATIO,
  },
  { name: "english-job-board", path: "/en/job-board", maxDiffRatio: STRICT_ROUTE_MAX_DIFF_RATIO },
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
