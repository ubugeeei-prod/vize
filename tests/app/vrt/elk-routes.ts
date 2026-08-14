import { ELK_RENDER_ROUTE } from "../dev/elk-route-contract.ts";

export interface ElkVisualRouteConfig {
  maxDiffRatio?: number;
  name: string;
  path: string;
  storage?: Record<string, string>;
  viewport?: { height: number; width: number };
}

export const DEFAULT_VIEWPORT = { width: 1280, height: 720 };
export const MOBILE_VIEWPORT = { width: 390, height: 844 };
export const DEFAULT_MAX_DIFF_RATIO = 0.04;

export const elkVisualRoutes: ElkVisualRouteConfig[] = [
  { name: "settings-shell", path: ELK_RENDER_ROUTE },
  { name: "settings-shell-mobile", path: ELK_RENDER_ROUTE, viewport: MOBILE_VIEWPORT },
  { name: "explore", path: "/explore" },
  { name: "explore-users", path: "/explore/users" },
  { name: "explore-tags", path: "/explore/tags" },
  { name: "explore-links", path: "/explore/links" },
  { name: "public", path: "/public" },
  { name: "public-local", path: "/public/local" },
  { name: "search", path: "/search" },
  { name: "hashtags", path: "/hashtags" },
  { name: "settings-interface", path: "/settings/interface" },
  { name: "settings-language", path: "/settings/language" },
  { name: "settings-preferences", path: "/settings/preferences" },
  { name: "notifications", path: "/notifications" },
  { name: "compose", path: "/compose" },
  { name: "share-target", path: "/share-target?text=hello" },
];
