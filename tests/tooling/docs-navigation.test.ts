import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { SCRIPT_BASENAMES } from "../../docs/theme/background.ts";
import { repoRoot } from "./_helpers/moonbit.ts";

type LocaleStrings = {
  labels: Record<string, string>;
  ui: { groups: Record<string, string> } & Record<string, unknown>;
};

type Sitemap = {
  blogNavigationPaths: string[];
  hiddenPathPatterns: RegExp[];
  navGroups: Array<{ key: string; paths: string[] }>;
  supportedLocales: Array<{ code: string; name: string }>;
};

const themeDir = path.join(repoRoot, "docs/theme");
const contentDir = path.join(repoRoot, "docs/content");

// The docs site ships no module loader: `background.ts` concatenates these
// files into one inline script and they hand data to each other through
// globals. Importing them in the same order is how this file reads them.
for (const basename of SCRIPT_BASENAMES) {
  if (basename.startsWith("i18n/")) {
    await import(path.join(themeDir, `${basename}.js`));
  }
}

const sitemap = (globalThis as { __vizeDocsSitemap?: Sitemap }).__vizeDocsSitemap!;
const locales = (globalThis as { __vizeDocsLocales?: Record<string, LocaleStrings> })
  .__vizeDocsLocales!;
const localeCodes = sitemap.supportedLocales.map(({ code }) => code);

/** The `{ code, name }` pairs `docs/vite.config.ts` builds the site for. */
function configuredLocales(): Array<{ code: string; name: string }> {
  const viteConfig = fs.readFileSync(path.join(repoRoot, "docs/vite.config.ts"), "utf8");
  const block = /locales:\s*\[(?<entries>[^\]]*)\]/u.exec(viteConfig)?.groups?.entries;
  assert.ok(block, "docs/vite.config.ts must declare an i18n locales array");
  return [
    ...block.matchAll(/\{\s*code:\s*"(?<code>[^"]+)"\s*,\s*name:\s*"(?<name>[^"]+)"\s*\}/gu),
  ].map(({ groups }) => ({ code: groups!.code, name: groups!.name }));
}

function hasContentPage(locale: string, navPath: string): boolean {
  const localeDir = locale === "en" ? contentDir : path.join(contentDir, locale);
  if (navPath === "/") {
    return fs.existsSync(path.join(localeDir, "index.md"));
  }
  const relative = navPath.slice(1);
  return (
    fs.existsSync(path.join(localeDir, `${relative}.md`)) ||
    fs.existsSync(path.join(localeDir, relative, "index.md"))
  );
}

void test("theme scripts load the sitemap and every locale before navigation", () => {
  assert.deepEqual(SCRIPT_BASENAMES, [
    "vein",
    "i18n/sitemap",
    "i18n/locales/en",
    "i18n/locales/ja",
    "i18n/locales/zh-CN",
    "i18n/locales/pt-BR",
    "i18n/locales/fr",
    "i18n/navigation",
    "syntax-highlight",
  ]);

  // A locale file that exists but is never concatenated would leave the
  // sidebar untranslated in the browser while every unit test still passed.
  assert.deepEqual(
    fs.readdirSync(path.join(themeDir, "i18n/locales")).sort(),
    localeCodes.map((code) => `${code}.js`).sort(),
  );
  assert.deepEqual(
    SCRIPT_BASENAMES.filter((basename) => basename.startsWith("i18n/locales/")),
    localeCodes.map((code) => `i18n/locales/${code}`),
  );
});

void test("the sitemap ships the locales the docs site is built for", () => {
  assert.deepEqual(sitemap.supportedLocales, [
    { code: "en", name: "English" },
    { code: "ja", name: "日本語" },
    { code: "zh-CN", name: "简体中文" },
    { code: "pt-BR", name: "Português" },
    { code: "fr", name: "Français" },
  ]);
  assert.deepEqual(Object.keys(locales), localeCodes);

  // The whole list, both directions: a locale the theme knows about but the
  // site is not built for is as broken as the reverse.
  assert.deepEqual(configuredLocales(), sitemap.supportedLocales);
});

void test("the sidebar groups the pages the site actually publishes", () => {
  assert.deepEqual(
    sitemap.navGroups.map((group) => group.key),
    ["start", "projectSetup", "staticAnalysis", "rules", "tooling", "architecture", "blog"],
  );
  assert.deepEqual(sitemap.navGroups.at(-1)?.paths, sitemap.blogNavigationPaths);
  assert.deepEqual(sitemap.blogNavigationPaths, ["/blog", "/blog/notes", "/blog/releases"]);

  assert.deepEqual(sitemap.navGroups.find((group) => group.key === "projectSetup")?.paths, [
    "/guide/vite-plugin",
    "/integrations/nuxt",
    "/guide/workflows",
    "/guide/configuration",
    "/guide/jsx",
    "/guide/jsx-babel-compat",
    "/guide/troubleshooting",
    "/guide/unplugin",
  ]);

  const navPaths = sitemap.navGroups.flatMap((group) => group.paths);
  for (const locale of localeCodes) {
    const missing = navPaths.filter((navPath) => !hasContentPage(locale, navPath));
    assert.deepEqual(missing, [], `${locale} is missing pages for sidebar entries`);
  }
});

void test("every locale labels every sidebar entry", () => {
  const hidden = (navPath: string) =>
    sitemap.hiddenPathPatterns.some((pattern) => pattern.test(navPath));
  // English is the fallback for every other locale, so it is the only file
  // carrying the dated blog post titles. Strip those and the five files must
  // agree key for key, in the same order.
  const expectedKeys = Object.keys(locales.en.labels).filter((navPath) => !hidden(navPath));

  const groupKeys = sitemap.navGroups.map((group) => group.key);
  for (const locale of localeCodes) {
    const keys = Object.keys(locales[locale].labels);
    assert.deepEqual(locale === "en" ? keys.filter((k) => !hidden(k)) : keys, expectedKeys);
    // Derived from the sitemap, whose own key list is pinned literally above,
    // so a new group forces a heading in every locale. Order-sensitive: these
    // are hand-maintained files and keeping them aligned costs nothing.
    assert.deepEqual(
      Object.keys(locales[locale].ui.groups),
      groupKeys,
      `${locale} must name every sitemap group`,
    );
  }

  const navPaths = sitemap.navGroups.flatMap((group) => group.paths);
  assert.deepEqual(
    navPaths.filter((navPath) => !(navPath in locales.en.labels)),
    [],
  );
});

void test("the Babel JSX compatibility page is labelled in every locale", () => {
  assert.deepEqual(
    Object.fromEntries(
      localeCodes.map((code) => [code, locales[code].labels["/guide/jsx-babel-compat"]]),
    ),
    {
      en: "Babel JSX Compat",
      ja: "Babel JSX 互換",
      "zh-CN": "Babel JSX 兼容",
      "pt-BR": "Compatibilidade com Babel JSX",
      fr: "Compatibilité Babel JSX",
    },
  );
});

void test("the docs theme suite passes", () => {
  const result = spawnSync(
    process.execPath,
    ["--test", "docs/theme/navigation.test.js", "docs/theme/background.test.ts"],
    { cwd: repoRoot, encoding: "utf8" },
  );

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
});
