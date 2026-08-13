import { createHash } from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";

type SourceContract = {
  anchors: readonly string[];
};

export type NpmxHeadFixtureContent = {
  aboutDescription: string;
  aboutTitle: string;
  accessibilityDescription: string;
  accessibilityTitle: string;
  docsOgTitle: string;
  docsTitle: string;
};

export const NPMX_HEAD_SOURCE_CONTRACTS = {
  "app/app.vue": {
    anchors: ["useHead({", "titleTemplate:", "name: 'color-scheme'"],
  },
  "app/pages/about.vue": {
    anchors: ["useSeoMeta({", "$t('about.meta_description')", "twitterDescription:"],
  },
  "app/pages/accessibility.vue": {
    anchors: ["definePageMeta({", "name: 'accessibility'", "$t('a11y.welcome'"],
  },
  "app/pages/package-docs/[...path].vue": {
    anchors: [
      "definePageMeta({",
      "name: 'docs'",
      "alias: ['/package/docs/:path+', '/docs/:path+']",
      "scrollMargin: 180",
      "useSeoMeta({",
      "ogTitle: () => t('package.docs.og_title'",
    ],
  },
  "app/pages/package/[[org]]/[name].vue": {
    anchors: ["useHead({", "rel: 'canonical'", "useSeoMeta({"],
  },
  "i18n/locales/en.json": {
    anchors: ['"about"', '"meta_description"', '"a11y"', '"welcome"'],
  },
} as const satisfies Record<string, SourceContract>;

export type NpmxHeadSourceEvidence = Record<string, string>;

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}

export function assertNpmxHeadMacroAnchors(sources: Record<string, string>): void {
  for (const [relativePath, contract] of Object.entries(NPMX_HEAD_SOURCE_CONTRACTS)) {
    const source = sources[relativePath];
    if (source === undefined) {
      throw new Error(`missing pinned npmx head source: ${relativePath}`);
    }
    for (const anchor of contract.anchors) {
      if (!source.includes(anchor)) {
        throw new Error(`missing npmx head macro anchor in ${relativePath}: ${anchor}`);
      }
    }
  }
}

function readJsonObject(filePath: string): Record<string, unknown> {
  const parsed = JSON.parse(fs.readFileSync(filePath, "utf8")) as unknown;
  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error(`expected object JSON fixture: ${filePath}`);
  }
  return parsed as Record<string, unknown>;
}

function readStringPath(
  root: Record<string, unknown>,
  keys: readonly string[],
  filePath: string,
): string {
  let cursor: unknown = root;
  for (const key of keys) {
    if (cursor === null || typeof cursor !== "object" || Array.isArray(cursor)) {
      throw new Error(`missing string fixture key ${keys.join(".")} in ${filePath}`);
    }
    cursor = (cursor as Record<string, unknown>)[key];
  }
  if (typeof cursor !== "string") {
    throw new Error(`missing string fixture key ${keys.join(".")} in ${filePath}`);
  }
  return cursor;
}

function formatFixtureMessage(message: string, replacements: Record<string, string>): string {
  return message.replaceAll(/\{([^{}]+)\}/g, (placeholder, key: string) => {
    const replacement = replacements[key];
    return replacement ?? placeholder;
  });
}

export function readNpmxHeadFixtureContent(fixtureRoot: string): NpmxHeadFixtureContent {
  const localePath = path.join(fixtureRoot, "i18n/locales/en.json");
  const locale = readJsonObject(localePath);
  const docsPackageName = "nuxt";
  const docsPackageVersion = "4.0.0";
  const docsVersionName = `${docsPackageName}@${docsPackageVersion}`;
  return {
    aboutDescription: readStringPath(locale, ["about", "meta_description"], localePath),
    aboutTitle: readStringPath(locale, ["about", "title"], localePath),
    accessibilityDescription: formatFixtureMessage(
      readStringPath(locale, ["a11y", "welcome"], localePath),
      { app: "npmx" },
    ),
    accessibilityTitle: readStringPath(locale, ["a11y", "title"], localePath),
    docsOgTitle: formatFixtureMessage(
      readStringPath(locale, ["package", "docs", "og_title"], localePath),
      { name: docsPackageName },
    ),
    docsTitle: formatFixtureMessage(
      readStringPath(locale, ["package", "docs", "page_title_version"], localePath),
      { name: docsVersionName },
    ),
  };
}

export function readNpmxHeadSourceEvidence(fixtureRoot: string): NpmxHeadSourceEvidence {
  const sources: Record<string, string> = {};
  const evidence: NpmxHeadSourceEvidence = {};

  for (const relativePath of Object.keys(NPMX_HEAD_SOURCE_CONTRACTS)) {
    const source = fs.readFileSync(path.join(fixtureRoot, relativePath), "utf8");
    sources[relativePath] = source;
    evidence[relativePath] = sha256(source);
  }

  assertNpmxHeadMacroAnchors(sources);
  return evidence;
}
