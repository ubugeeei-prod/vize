/// <reference lib="dom" />
/// <reference lib="dom.iterable" />
import { createHash } from "node:crypto";

export type DocumentMetadata = {
  title: string;
  socialTitle: string;
  description: string;
  paragraph: string;
  language: string;
};
export type PageProps = {
  title: string;
  description: string;
  siteName: string;
  category: string;
  locale: Locale;
  localeName: string;
  route: string;
  isHome: boolean;
  assetFingerprint: string;
};
export type PageMetadata = {
  route: string;
  title: string;
  description: string;
  descriptionOrigin: "authored" | "content" | "fallback";
  locale: string;
  type: "article" | "website";
  url: string;
  image: string;
  props: PageProps;
};

export const docsSiteUrl = "https://vizejs.dev";
export const ogWidth = 1200;
export const ogHeight = 630;
const localeSettings = {
  en: { name: "English", og: "en_US", fallback: "High-Performance Vue.js Toolchain in Rust" },
  ja: { name: "日本語", og: "ja_JP", fallback: "Rust の高性能 Vue.js ツールチェーン" },
  "zh-CN": { name: "简体中文", og: "zh_CN", fallback: "Rust 驱动的高性能 Vue.js 工具链" },
  "pt-BR": {
    name: "Português",
    og: "pt_BR",
    fallback: "Ferramentas Vue.js de alto desempenho em Rust",
  },
  fr: { name: "Français", og: "fr_FR", fallback: "Outils Vue.js haute performance en Rust" },
};
type Locale = keyof typeof localeSettings;
function isLocale(value: string | undefined): value is Locale {
  return value !== undefined && Object.hasOwn(localeSettings, value);
}
export function isPageMetadata(value: unknown): value is PageMetadata {
  if (!isRecord(value)) return false;
  const entry = value;
  if (
    !["route", "title", "description", "locale", "url", "image"].every(
      (key) => typeof entry[key] === "string",
    )
  )
    return false;
  if (
    (entry.descriptionOrigin !== "authored" &&
      entry.descriptionOrigin !== "content" &&
      entry.descriptionOrigin !== "fallback") ||
    (entry.type !== "article" && entry.type !== "website")
  )
    return false;
  const props = entry.props;
  if (!isRecord(props)) return false;
  const fields = props;
  return (
    [
      "title",
      "description",
      "siteName",
      "category",
      "localeName",
      "route",
      "assetFingerprint",
    ].every((key) => typeof fields[key] === "string") &&
    typeof fields.isHome === "boolean" &&
    typeof fields.locale === "string" &&
    isLocale(fields.locale)
  );
}
function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
const categories = {
  guide: ["Guide", "ガイド", "指南", "Guia", "Guide"],
  rules: [
    "Rule reference",
    "ルールリファレンス",
    "规则参考",
    "Referência de regras",
    "Référence des règles",
  ],
  components: ["Components", "コンポーネント", "组件", "Componentes", "Composants"],
  architecture: ["Architecture", "アーキテクチャ", "架构", "Arquitetura", "Architecture"],
  integrations: ["Integrations", "統合", "集成", "Integrações", "Intégrations"],
  blog: ["Blog", "ブログ", "博客", "Blog", "Blog"],
  release: ["Release notes", "リリースノート", "发布说明", "Notas de versão", "Notes de version"],
};

/** Runs inside Chromium against the actual generated document. */
export function readDocumentMetadata(html: string): DocumentMetadata {
  const document = new DOMParser().parseFromString(html, "text/html");
  const normalize = (value: string | null | undefined) => value?.replace(/\s+/gu, " ").trim() ?? "";
  const description = document.head
    .querySelector('meta[name="description"]')
    ?.getAttribute("content");
  const paragraphs = [...document.querySelectorAll(".content p")];
  const paragraph = paragraphs.find((element) => {
    const clone = element.cloneNode(true);
    if (!(clone instanceof Element)) throw new Error("Generated paragraph is not an element");
    clone.querySelectorAll("a").forEach((link) => link.remove());
    return /[\p{L}\p{N}]/u.test(clone.textContent ?? "");
  });
  const socialTitle = normalize(
    document.head.querySelector('meta[property="og:title"]')?.getAttribute("content") ||
      document.title,
  );
  return {
    title:
      socialTitle.replace(/ - Vize$/u, "") || normalize(document.querySelector("h1")?.textContent),
    socialTitle,
    description: normalize(description),
    paragraph: normalize(paragraph?.textContent),
    language: document.documentElement.lang,
  };
}

export function pageRoute(relativeFile: string) {
  const route = `/${relativeFile
    .replaceAll("\\", "/")
    .replace(/\.(?:md|markdown|mdx)$/u, "")
    .replace(/(?:^|\/)index$/u, "")}`;
  return `${route.replace(/\/+$/u, "")}/`;
}

export function pageMetadata(
  route: string,
  document: DocumentMetadata,
  assetFingerprint: string,
): PageMetadata {
  const segments = route.split("/").filter(Boolean);
  const firstSegment = segments[0];
  const locale = isLocale(firstSegment) ? firstSegment : "en";
  if (isLocale(firstSegment)) segments.shift();
  const settings = localeSettings[locale];
  if (document.language !== locale)
    throw new Error(`${route}: HTML language ${document.language} differs from route ${locale}`);
  if (!document.title || !document.socialTitle)
    throw new Error(`${route}: generated page has no title`);
  const isHome = segments.length === 0;
  const categoryKey =
    segments[0] === "rules"
      ? "rules"
      : segments[0] === "architecture"
        ? "architecture"
        : segments[0] === "integrations"
          ? "integrations"
          : segments[0] === "blog"
            ? segments[1] === "releases"
              ? "release"
              : "blog"
            : segments.some((segment) => segment === "ui" || segment === "composables")
              ? "components"
              : "guide";
  const localeIndex = Object.keys(localeSettings).indexOf(locale);
  const description =
    document.description || document.paragraph || `${document.title} — ${settings.fallback}`;
  const props = {
    title: document.title,
    description,
    siteName: "Vize",
    category: categories[categoryKey][localeIndex],
    locale,
    localeName: settings.name,
    route,
    isHome,
    assetFingerprint,
  };
  const identity = createHash("sha256").update(JSON.stringify(props)).digest("hex");
  return {
    route,
    title: document.socialTitle,
    description,
    descriptionOrigin: document.description
      ? "authored"
      : document.paragraph
        ? "content"
        : "fallback",
    locale: settings.og,
    type: segments[0] === "blog" && segments.length > 2 ? "article" : "website",
    url: new URL(route, docsSiteUrl).href,
    image: new URL(`/_og/${locale}-${identity}.png`, docsSiteUrl).href,
    props,
  };
}

export function escapeAttribute(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

export function applyOpenGraphMetadata(html: string, page: PageMetadata) {
  const properties = {
    "og:title": page.title,
    "og:description": page.description,
    "og:type": page.type,
    "og:site_name": "Vize",
    "og:url": page.url,
    "og:locale": page.locale,
    "og:image": page.image,
    "og:image:type": "image/png",
    "og:image:width": String(ogWidth),
    "og:image:height": String(ogHeight),
    "og:image:alt": `${page.props.title} · ${page.props.category} · Vize`,
  };
  const names = {
    description: page.description,
    "twitter:card": "summary_large_image",
    "twitter:title": page.title,
    "twitter:description": page.description,
    "twitter:image": page.image,
    "twitter:image:alt": properties["og:image:alt"],
  };
  const tags = [
    ...Object.entries(properties).map(
      ([key, value]) => `<meta property="${key}" content="${escapeAttribute(value)}">`,
    ),
    ...Object.entries(names).map(
      ([key, value]) => `<meta name="${key}" content="${escapeAttribute(value)}">`,
    ),
  ];
  if (!/<head\b[^>]*>[\s\S]*?<\/head>/iu.test(html))
    throw new Error(`${page.route}: generated HTML has no head`);
  return html.replace(
    /(<head\b[^>]*>)\s*([\s\S]*?)(<\/head>)/iu,
    (_: string, head: string, content: string, end: string) => {
      const retained = content.replace(/<meta\b[^>]*>/giu, (tag: string) => {
        const key = /\b(property|name)\s*=\s*["']([^"']+)["']/iu.exec(tag);
        return key &&
          Object.hasOwn(key[1].toLowerCase() === "property" ? properties : names, key[2])
          ? ""
          : tag;
      });
      // Keep the authored charset and other existing head nodes first. A long
      // localized social description must not push charset beyond its first KB.
      return `${head}${retained.trim()}\n${tags.join("\n")}\n${end}`;
    },
  );
}

export function assertPngDimensions(bytes: Buffer, label: string) {
  if (
    bytes.length < 24 ||
    bytes.subarray(0, 8).toString("hex") !== "89504e470d0a1a0a" ||
    bytes.subarray(12, 16).toString() !== "IHDR"
  ) {
    throw new Error(`${label}: invalid PNG`);
  }
  if (bytes.readUInt32BE(16) !== ogWidth || bytes.readUInt32BE(20) !== ogHeight) {
    throw new Error(`${label}: OG image must be ${ogWidth}×${ogHeight}`);
  }
}
