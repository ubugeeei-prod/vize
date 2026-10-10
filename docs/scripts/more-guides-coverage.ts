import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";

import "../theme/i18n/sitemap.js";

type Sitemap = {
  supportedLocales: { code: string }[];
  hiddenPathPatterns: RegExp[];
  navGroups: { key: string; paths: string[]; pathsByLocale?: Record<string, string[]> }[];
};
type CoveragePage = {
  route: string;
  locales: string[];
  entry: "revised" | "audited";
  capture?: boolean;
  note: string;
};

const sitemap = (globalThis as { __vizeDocsSitemap?: Sitemap }).__vizeDocsSitemap!;
export const docsRoot = path.resolve(import.meta.dirname, "..");
export const moreGuidesCoverage = JSON.parse(
  readFileSync(path.join(docsRoot, "quality/more-guides-coverage.json"), "utf8"),
) as { version: number; issue: number; source: string; pages: CoveragePage[] };

function authoredPages(directory: string, prefix = ""): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    if (
      !prefix &&
      entry.isDirectory() &&
      ["en", "i18n", "generated", ...sitemap.supportedLocales.map(({ code }) => code)].includes(
        entry.name,
      )
    ) {
      return [];
    }
    const relative = `${prefix}${entry.name}`;
    if (entry.isDirectory()) return authoredPages(path.join(directory, entry.name), `${relative}/`);
    if (!entry.isFile() || !entry.name.endsWith(".md")) return [];
    return [
      "/" +
        relative
          .replace(/\.md$/, "")
          .replace(/(^|\/)index$/, "")
          .replace(/\/$/, ""),
    ];
  });
}

/** Match navigation.js's More guides group and ungrouped More fallback. */
export function moreGuideRoutes(locale: "en" | "ja"): string[] {
  const localeDirectory = path.join(docsRoot, "content", locale === "en" ? "" : locale);
  const used = new Set(
    sitemap.navGroups.flatMap((group) => group.pathsByLocale?.[locale] ?? group.paths),
  );
  const architecture = sitemap.navGroups.find(({ key }) => key === "architecture");
  const guides = architecture?.pathsByLocale?.[locale] ?? architecture?.paths ?? [];
  const fallback = authoredPages(localeDirectory).filter(
    (route) =>
      !used.has(route) && !sitemap.hiddenPathPatterns.some((pattern) => pattern.test(route)),
  );
  return [...new Set([...guides, ...fallback])].sort();
}

export const renderedMoreGuideRoutes = moreGuidesCoverage.pages
  .filter(({ entry, capture }) => entry === "revised" || capture === true)
  .flatMap(({ route, locales }) =>
    locales.map((locale) => (locale === "en" ? route : `/${locale}${route}`)),
  );
