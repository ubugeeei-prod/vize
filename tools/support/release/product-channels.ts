// Release channel registry: every artifact Vize publishes belongs to exactly one product,
// and every product to exactly one release channel and one owner. The stability pages
// render their channel table from this file, package manifests carry the channel in
// `metadata.vize.channel`, and tests/tooling/product-channels.test.ts rejects a
// publishable artifact that is missing here, listed twice, or out of date.
//
// All channels still release under one version number. The channel names the product
// line an artifact ships with; the support tier in its manifest names how mature its
// current API is.

import { readFileSync } from "node:fs";
import path from "node:path";

import { jsrPackages } from "./jsr/packages.ts";

export const RELEASE_CHANNELS = ["v1-alpha", "v0", "early", "v2-preview", "v3-preview"] as const;
export type ReleaseChannel = (typeof RELEASE_CHANNELS)[number];

/** Where an artifact is published. */
export type ArtifactRegistry =
  | "npm"
  | "crates.io"
  | "jsr"
  | "vscode-marketplace"
  | "open-vsx"
  | "github-releases";

export const STABILITY_LOCALES = ["en", "ja", "pt-BR", "fr", "zh-CN"] as const;
export type StabilityLocale = (typeof STABILITY_LOCALES)[number];
type Localized = Readonly<Record<StabilityLocale, string>>;

export interface ProductChannel {
  readonly channel: ReleaseChannel;
  readonly owner: string;
  readonly label: Localized;
  /** npm packages, by package name. JSR mirrors inherit the product of their npm package. */
  readonly npm?: readonly string[];
  readonly crates?: readonly string[];
  /** VS Code extensions, by extension name; `openVsx` also publishes them to Open VSX. */
  readonly vscode?: readonly { readonly name: string; readonly openVsx: boolean }[];
  /** GitHub release assets, by file name; `<target>` and `<platform>` stand for each build. */
  readonly githubReleases?: readonly string[];
}

export const CHANNEL_LABELS: Readonly<Record<ReleaseChannel, Localized>> = {
  "v1-alpha": {
    en: "v1 alpha",
    ja: "v1 アルファ",
    "pt-BR": "v1 alfa",
    fr: "v1 alpha",
    "zh-CN": "v1 alpha",
  },
  v0: { en: "v0", ja: "v0", "pt-BR": "v0", fr: "v0", "zh-CN": "v0" },
  early: {
    en: "Early development",
    ja: "開発初期",
    "pt-BR": "Desenvolvimento inicial",
    fr: "Développement précoce",
    "zh-CN": "早期开发",
  },
  "v2-preview": {
    en: "v2 preview",
    ja: "v2 プレビュー",
    "pt-BR": "Prévia v2",
    fr: "Aperçu v2",
    "zh-CN": "v2 预览",
  },
  "v3-preview": {
    en: "v3 preview",
    ja: "v3 プレビュー",
    "pt-BR": "Prévia v3",
    fr: "Aperçu v3",
    "zh-CN": "v3 预览",
  },
};

export const NATIVE_PLATFORM_PACKAGES = [
  "@vizejs/native-darwin-arm64",
  "@vizejs/native-darwin-x64",
  "@vizejs/native-linux-arm64-gnu",
  "@vizejs/native-linux-arm64-musl",
  "@vizejs/native-linux-x64-gnu",
  "@vizejs/native-linux-x64-musl",
  "@vizejs/native-win32-arm64-msvc",
  "@vizejs/native-win32-x64-msvc",
] as const;

export const PRODUCT_CHANNELS = {
  compiler: {
    channel: "v1-alpha",
    owner: "compiler/vize_atelier",
    label: {
      en: "Compiler (VDOM and Vapor)",
      ja: "コンパイラ（VDOM と Vapor）",
      "pt-BR": "Compilador (VDOM e Vapor)",
      fr: "Compilateur (VDOM et Vapor)",
      "zh-CN": "编译器（VDOM 与 Vapor）",
    },
    npm: ["vize", "@vizejs/native", ...NATIVE_PLATFORM_PACKAGES, "@vizejs/wasm", "@vizejs/compile"],
    crates: [
      "vize_carton",
      "vize_relief",
      "vize_armature",
      "vize_croquis",
      "vize_atelier_core",
      "vize_atelier_dom",
      "vize_atelier_vapor",
      "vize_atelier_ssr",
      "vize_atelier_sfc",
    ],
    githubReleases: ["vize-<target>.tar.gz", "vize-<target>.zip"],
  },
  vite: {
    channel: "v1-alpha",
    owner: "integrations/@vizejs/vite-plugin",
    label: {
      en: "Vite integration",
      ja: "Vite 統合",
      "pt-BR": "Integração com Vite",
      fr: "Intégration Vite",
      "zh-CN": "Vite 集成",
    },
    npm: ["@vizejs/vite-plugin", "@vizejs/unplugin"],
  },
  nuxt: {
    channel: "v1-alpha",
    owner: "integrations/@vizejs/nuxt",
    label: {
      en: "Nuxt integration",
      ja: "Nuxt 統合",
      "pt-BR": "Integração com Nuxt",
      fr: "Intégration Nuxt",
      "zh-CN": "Nuxt 集成",
    },
    npm: ["@vizejs/nuxt", "@vizejs/nuxt-lint-config"],
  },
  linter: {
    channel: "v1-alpha",
    owner: "linter/vize_patina",
    label: {
      en: "Native linter",
      ja: "ネイティブリンター",
      "pt-BR": "Linter nativo",
      fr: "Linter natif",
      "zh-CN": "原生 Linter",
    },
    npm: ["oxlint-plugin-vize", "@vizejs/lint"],
    crates: ["vize_patina"],
  },
  typechecker: {
    channel: "v1-alpha",
    owner: "typechecker/vize_canon",
    label: {
      en: "Typechecker and Content Mapper",
      ja: "型チェッカーと Content Mapper",
      "pt-BR": "Verificador de tipos e Content Mapper",
      fr: "Vérificateur de types et Content Mapper",
      "zh-CN": "类型检查器与 Content Mapper",
    },
    npm: ["@vizejs/typecheck", "@vizejs/content-mapper"],
    crates: ["vize_canon"],
  },
  "lsp-editors": {
    channel: "v1-alpha",
    owner: "lsp/vize_maestro",
    label: {
      en: "LSP and editor extensions",
      ja: "LSP とエディタ拡張",
      "pt-BR": "LSP e extensões de editor",
      fr: "LSP et extensions d’éditeur",
      "zh-CN": "LSP 与编辑器扩展",
    },
    npm: ["@vizejs/lsp", "@vizejs/typescript-vue-plugin"],
    vscode: [{ name: "vize", openVsx: true }],
    githubReleases: [
      "vize.vsix",
      "vize-<platform>.vsix",
      "zed-vize-extension.tar.gz",
      "nvim-vize-extension.tar.gz",
      "vim-vize-extension.tar.gz",
      "helix-vize-extension.tar.gz",
      "emacs-vize-extension.tar.gz",
    ],
  },
  "plugin-sdk": {
    channel: "v1-alpha",
    owner: "plugins/@vizejs/plugin-sdk",
    label: {
      en: "Plugin SDK",
      ja: "プラグイン SDK",
      "pt-BR": "SDK de plugins",
      fr: "SDK de plugins",
      "zh-CN": "插件 SDK",
    },
    npm: ["@vizejs/plugin-sdk"],
  },
  formatter: {
    channel: "v0",
    owner: "formatter/vize_glyph",
    label: {
      en: "Glyph formatter",
      ja: "Glyph フォーマッタ",
      "pt-BR": "Formatador Glyph",
      fr: "Formateur Glyph",
      "zh-CN": "Glyph 格式化器",
    },
    npm: ["@vizejs/fmt"],
  },
  musea: {
    channel: "v0",
    owner: "musea/vize_musea",
    label: { en: "Musea", ja: "Musea", "pt-BR": "Musea", fr: "Musea", "zh-CN": "Musea" },
    npm: ["@vizejs/vite-plugin-musea", "@vizejs/musea-mcp-server", "@vizejs/musea-nuxt"],
    crates: ["vize_musea"],
    vscode: [{ name: "vize-art", openVsx: false }],
  },
  ui: {
    channel: "early",
    owner: "ui/@vizejs/ui",
    label: { en: "UI", ja: "UI", "pt-BR": "UI", fr: "UI", "zh-CN": "UI" },
    npm: ["@vizejs/ui"],
  },
  fresco: {
    channel: "early",
    owner: "fresco/vize_fresco",
    label: { en: "Fresco", ja: "Fresco", "pt-BR": "Fresco", fr: "Fresco", "zh-CN": "Fresco" },
    npm: ["@vizejs/fresco", "@vizejs/fresco-native", "@vizejs/create-fresco", "@vizejs/tui"],
    crates: ["vize_fresco"],
  },
  "app-foundations": {
    channel: "early",
    owner: "framework/vize_marquette",
    label: {
      en: "Application foundations",
      ja: "アプリケーション基盤",
      "pt-BR": "Fundamentos de aplicação",
      fr: "Fondations d’application",
      "zh-CN": "应用基础",
    },
    npm: [
      "@vizejs/marquette",
      "@vizejs/composable",
      "@vizejs/data",
      "@vizejs/router",
      "@vizejs/state",
      "@vizejs/devtools",
    ],
    crates: ["vize_marquette"],
  },
  doctor: {
    channel: "early",
    owner: "doctor/vize_doctor",
    label: {
      en: "Doctor and project analysis",
      ja: "Doctor とプロジェクト解析",
      "pt-BR": "Doctor e análise de projeto",
      fr: "Doctor et analyse de projet",
      "zh-CN": "Doctor 与项目分析",
    },
    npm: ["@vizejs/doctor", "@vizejs/curator"],
    crates: ["vize_doctor", "vize_croquis_cf"],
  },
  dialects: {
    channel: "v2-preview",
    owner: "compiler/vize_atelier_jsx",
    label: {
      en: "JSX and TSRX dialects",
      ja: "JSX と TSRX の方言",
      "pt-BR": "Dialetos JSX e TSRX",
      fr: "Dialectes JSX et TSRX",
      "zh-CN": "JSX 与 TSRX 方言",
    },
    npm: ["@vizejs/compile-jsx", "@vizejs/compile-tsrx"],
    crates: ["vize_atelier_jsx"],
  },
  adapters: {
    channel: "v2-preview",
    owner: "integrations/adapters",
    label: {
      en: "Builder, ESLint, Prettier and Volar adapters",
      ja: "ビルダー・ESLint・Prettier・Volar アダプタ",
      "pt-BR": "Adaptadores de builders, ESLint, Prettier e Volar",
      fr: "Adaptateurs de builders, ESLint, Prettier et Volar",
      "zh-CN": "构建工具、ESLint、Prettier 与 Volar 适配器",
    },
    npm: [
      "@vizejs/babel-plugin",
      "@vizejs/rspack-plugin",
      "@vizejs/turbopack-plugin",
      "@vizejs/parcel-transformer",
      "@vizejs/parcel-transformer-native",
      "eslint-plugin-vize",
      "prettier-plugin-vize",
      "@vizejs/volar-plugin",
    ],
  },
  davinci: {
    channel: "v3-preview",
    owner: "davinci/levels",
    label: {
      en: "Davinci levels",
      ja: "Davinci レベル",
      "pt-BR": "Níveis Davinci",
      fr: "Niveaux Davinci",
      "zh-CN": "Davinci 层级",
    },
    crates: [
      "vize_l0",
      "vize_l0_derive",
      "vize_l1",
      "vize_l1_to_l2",
      "vize_l2",
      "vize_l2_to_l3",
      "vize_l3",
      "vize_l4",
    ],
  },
} as const satisfies Readonly<Record<string, ProductChannel>>;

export type ProductId = keyof typeof PRODUCT_CHANNELS;

export interface ChannelArtifact {
  readonly registry: ArtifactRegistry;
  readonly name: string;
  readonly product: ProductId;
  readonly channel: ReleaseChannel;
}

/** Every registered artifact except JSR mirrors, which follow their npm package. */
export function channelArtifacts(): ChannelArtifact[] {
  const artifacts: ChannelArtifact[] = [];
  for (const [product, entry] of Object.entries(PRODUCT_CHANNELS) as [
    ProductId,
    ProductChannel,
  ][]) {
    const add = (registry: ArtifactRegistry, name: string) =>
      artifacts.push({ registry, name, product, channel: entry.channel });
    for (const name of entry.npm ?? []) add("npm", name);
    for (const name of entry.crates ?? []) add("crates.io", name);
    for (const { name, openVsx } of entry.vscode ?? []) {
      add("vscode-marketplace", name);
      if (openVsx) add("open-vsx", name);
    }
    for (const name of entry.githubReleases ?? []) add("github-releases", name);
  }
  return artifacts;
}

/** The product that owns one npm package, or undefined when it is not registered. */
export function npmProduct(name: string): ProductId | undefined {
  return channelArtifacts().find(
    (artifact) => artifact.registry === "npm" && artifact.name === name,
  )?.product;
}

export function productChannel(product: ProductId): ReleaseChannel {
  return PRODUCT_CHANNELS[product].channel;
}

const TABLE_HEADERS: Readonly<Record<StabilityLocale, readonly [string, string, string, string]>> =
  {
    en: ["Channel", "Product", "Owner", "Artifacts"],
    ja: ["チャンネル", "プロダクト", "オーナー", "成果物"],
    "pt-BR": ["Canal", "Produto", "Responsável", "Artefatos"],
    fr: ["Canal", "Produit", "Responsable", "Artefacts"],
    "zh-CN": ["渠道", "产品", "负责人", "产物"],
  };

const REGISTRY_LABELS: Readonly<Record<ArtifactRegistry, string>> = {
  npm: "npm",
  "crates.io": "crates.io",
  jsr: "JSR",
  "vscode-marketplace": "VS Code Marketplace",
  "open-vsx": "Open VSX",
  "github-releases": "GitHub Releases",
};

export const CHANNEL_TABLE_START = "<!-- product-channels:start -->";
export const CHANNEL_TABLE_END = "<!-- product-channels:end -->";

function cell(value: string): string {
  return value.replaceAll("|", "\\|");
}

/** The channel table of one locale's stability page, in channel order. */
export function renderChannelTable(
  locale: StabilityLocale,
  jsrMirrors: ReadonlyMap<string, string>,
): string {
  const lines = [
    `| ${TABLE_HEADERS[locale].join(" | ")} |`,
    `| ${TABLE_HEADERS[locale].map(() => "---").join(" | ")} |`,
  ];
  const artifacts = channelArtifacts();
  for (const channel of RELEASE_CHANNELS)
    for (const [product, entry] of Object.entries(PRODUCT_CHANNELS) as [
      ProductId,
      ProductChannel,
    ][]) {
      if (entry.channel !== channel) continue;
      const owned = artifacts.filter((artifact) => artifact.product === product);
      const jsr = owned.flatMap(({ registry, name }) =>
        registry === "npm" && jsrMirrors.has(name) ? [jsrMirrors.get(name)!] : [],
      );
      const groups = (Object.keys(REGISTRY_LABELS) as ArtifactRegistry[]).flatMap((registry) => {
        const names =
          registry === "jsr"
            ? jsr
            : owned.filter((artifact) => artifact.registry === registry).map(({ name }) => name);
        return names.length === 0
          ? []
          : [`${REGISTRY_LABELS[registry]}: ${names.map((name) => `\`${name}\``).join(", ")}`];
      });
      lines.push(
        `| ${[
          CHANNEL_LABELS[channel][locale],
          entry.label[locale],
          `\`${entry.owner}\``,
          groups.join("; "),
        ]
          .map(cell)
          .join(" | ")} |`,
      );
    }
  return `${lines.join("\n")}\n`;
}

/** Replaces the marked channel table of a stability page. */
export function withChannelTable(page: string, table: string): string {
  const start = page.indexOf(CHANNEL_TABLE_START);
  const end = page.indexOf(CHANNEL_TABLE_END);
  if (start < 0 || end < start) throw new Error("stability page has no channel table markers");
  return `${page.slice(0, start + CHANNEL_TABLE_START.length)}\n\n${table}\n${page.slice(end)}`;
}

export function stabilityPagePath(root: string, locale: StabilityLocale): string {
  return path.join(root, "docs/content", locale === "en" ? "" : locale, "stability.md");
}

/** JSR mirror names by the npm package they re-export. */
export function jsrMirrors(): Map<string, string> {
  return new Map(jsrPackages.map(({ npmName, jsrName }) => [npmName, jsrName]));
}

/** One locale's stability page with its channel table regenerated from this registry. */
export function renderStabilityPage(root: string, locale: StabilityLocale): string {
  const page = readFileSync(stabilityPagePath(root, locale), "utf8");
  return withChannelTable(page, renderChannelTable(locale, jsrMirrors()));
}
