import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { assertPngDimensions, docsSiteUrl } from "../theme/open-graph.ts";
import type { PageMetadata } from "../theme/open-graph.ts";

export type RenderedPageMetadata = PageMetadata & { imageSha256: string };
type NativeDocument = { title: string; language: string };
const locales: Record<string, string> = {
  en: "en_US",
  ja: "ja_JP",
  "zh-CN": "zh_CN",
  "pt-BR": "pt_BR",
  fr: "fr_FR",
};
const homeRoutes = new Set(["/", ...Object.keys(locales).map((locale) => `/${locale}/`)]);

/** Bind native page identity and successful provider bytes to their published metadata. */
export function verifyImageIdentity(
  entry: RenderedPageMetadata,
  actual: NativeDocument,
  png: Buffer,
  fingerprint: string,
) {
  assertPngDimensions(png, entry.route);
  assert.equal(
    createHash("sha256").update(png).digest("hex"),
    entry.imageSha256,
    `${entry.route}: actual provider PNG byte custody`,
  );
  const routeLocale = entry.route.split("/")[1];
  const locale = Object.hasOwn(locales, routeLocale) ? routeLocale : "en";
  assert.equal(entry.props.locale, locale, `${entry.route}: route-derived language`);
  assert.equal(entry.locale, locales[locale], `${entry.route}: route-derived Open Graph locale`);
  assert.equal(
    entry.props.isHome,
    homeRoutes.has(entry.route),
    `${entry.route}: homepage layout authority`,
  );
  assert.equal(entry.props.description, entry.description, `${entry.route}: image description`);
  assert.equal(entry.props.siteName, "Vize", `${entry.route}: image site identity`);
  assert(entry.props.category.trim(), `${entry.route}: image category`);
  assert.equal(
    actual.title,
    entry.title,
    `${entry.route}: social title must retain the rendered page title`,
  );
  assert.equal(
    actual.title.replace(/ - Vize$/u, ""),
    entry.props.title,
    `${entry.route}: image must retain the rendered page title`,
  );
  assert.equal(
    entry.url,
    new URL(entry.route, docsSiteUrl).href,
    `${entry.route}: canonical page URL`,
  );
  assert.equal(entry.props.route, entry.route, `${entry.route}: image route identity`);
  assert.equal(
    entry.props.assetFingerprint,
    fingerprint,
    `${entry.route}: actual template identity`,
  );
  assert.equal(actual.language, entry.props.locale, `${entry.route}: native page language`);
}

/** Real unchanged page/PNG inputs must reject deliberately contradictory metadata and a swapped valid PNG. */
export function verifyImageIdentityControls(
  entry: RenderedPageMetadata,
  actual: NativeDocument,
  png: Buffer,
  otherPng: Buffer,
  fingerprint: string,
) {
  const variants: { name: string; entry: RenderedPageMetadata; failure: RegExp }[] = [
    {
      name: "image-description",
      entry: { ...entry, props: { ...entry.props, description: `${entry.description} foreign` } },
      failure: /image description/u,
    },
    {
      name: "home-layout",
      entry: { ...entry, props: { ...entry.props, isHome: !entry.props.isHome } },
      failure: /homepage layout authority/u,
    },
    {
      name: "og-locale",
      entry: { ...entry, locale: entry.locale === "ja_JP" ? "en_US" : "ja_JP" },
      failure: /route-derived Open Graph locale/u,
    },
    {
      name: "image-language",
      entry: {
        ...entry,
        props: { ...entry.props, locale: entry.props.locale === "ja" ? "en" : "ja" },
      },
      failure: /route-derived language/u,
    },
    {
      name: "social-title",
      entry: { ...entry, title: `${entry.title} foreign` },
      failure: /social title must retain/u,
    },
    {
      name: "image-title",
      entry: { ...entry, props: { ...entry.props, title: `${entry.props.title} foreign` } },
      failure: /image must retain/u,
    },
    {
      name: "image-route",
      entry: { ...entry, props: { ...entry.props, route: `${entry.route}foreign/` } },
      failure: /image route identity/u,
    },
  ];
  verifyImageIdentity(entry, actual, png, fingerprint);
  for (const variant of variants)
    assert.throws(
      () => verifyImageIdentity(variant.entry, actual, png, fingerprint),
      variant.failure,
      variant.name,
    );
  assertPngDimensions(otherPng, "valid foreign-page control PNG");
  assert.notEqual(createHash("sha256").update(otherPng).digest("hex"), entry.imageSha256);
  assert.throws(
    () => verifyImageIdentity(entry, actual, otherPng, fingerprint),
    /actual provider PNG byte custody/u,
    "valid distinct PNG permutation",
  );
  return [...variants.map((variant) => variant.name), "valid-png-permutation"];
}
