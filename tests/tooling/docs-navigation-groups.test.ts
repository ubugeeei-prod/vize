import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { SCRIPT_BASENAMES } from "../../docs/theme/background.ts";
import { repoRoot } from "./_helpers/tool-commands.ts";

type Sitemap = {
  hiddenPathPatterns: RegExp[];
  navGroups: Array<{ key: string; paths: string[]; pathsByLocale?: Record<string, string[]> }>;
  supportedLocales: Array<{ code: string; name: string }>;
};

const themeDir = path.join(repoRoot, "docs/theme");
const contentDir = path.join(repoRoot, "docs/content");

// The theme scripts hand data to each other through globals; load the sitemap
// the same way the site does.
for (const basename of SCRIPT_BASENAMES) {
  if (basename === "i18n/sitemap") await import(path.join(themeDir, `${basename}.js`));
}

const sitemap = (globalThis as { __vizeDocsSitemap?: Sitemap }).__vizeDocsSitemap!;
const localeCodes = sitemap.supportedLocales.map(({ code }) => code);

/** Authored page routes of one locale, in the form the sitemap lists them. */
function authoredRoutes(locale: string): string[] {
  const localeDir = locale === "en" ? contentDir : path.join(contentDir, locale);
  const skipped = new Set(["generated", "i18n", ...localeCodes]);
  const walk = (directory: string, prefix: string): string[] =>
    fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
      if (entry.isDirectory()) {
        if (!prefix && skipped.has(entry.name)) return [];
        return walk(path.join(directory, entry.name), `${prefix}${entry.name}/`);
      }
      if (!entry.name.endsWith(".md")) return [];
      const route = `/${prefix}${entry.name}`.replace(/\.md$/u, "").replace(/\/index$/u, "");
      return [route || "/"];
    });
  return walk(localeDir, "");
}

void test("every published page has a named sidebar group in every locale", () => {
  // `navigation.js` collects unlisted pages into a trailing "More" section.
  // That section grew to forty unrelated pages once; a new page now has to be
  // filed under a group, or hidden because its parent page links to it.
  for (const locale of localeCodes) {
    const listed = sitemap.navGroups.flatMap(
      (group) => group.pathsByLocale?.[locale] ?? group.paths,
    );
    assert.deepEqual(
      listed.filter((navPath, index) => listed.indexOf(navPath) !== index),
      [],
      `${locale} lists a page in more than one group`,
    );
    assert.deepEqual(
      authoredRoutes(locale)
        .filter(
          (route) =>
            !listed.includes(route) &&
            !sitemap.hiddenPathPatterns.some((pattern) => pattern.test(route)),
        )
        .sort(),
      [],
      `${locale} has pages that would fall into the More section`,
    );
  }
});
