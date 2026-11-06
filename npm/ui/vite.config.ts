import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";

import vue from "@vitejs/plugin-vue";
import type { Plugin } from "vite";
import { defineConfig } from "vite-plus";

import { uiStoryVariants } from "./scripts/story-variants.ts";
import { uiFamilyCatalog } from "./src/catalog/family-catalog.ts";

/**
 * Declared browser floor for the packaged stylesheet.
 *
 * Styles are authored in native CSS (nesting, cascade layers, logical
 * properties, native color functions) and down-compiled to this floor by the
 * package build, so `dist/style.css` never depends on the consumer's own CSS
 * toolchain. The floor is the earliest evergreen release line where cascade
 * layers, `:where()`, logical properties, and `oklch()` are all native; only
 * CSS Nesting is newer than the floor and is therefore always flattened.
 * `src/families/foundations/theme/style-pipeline.behavior.md` documents the
 * policy and how consumers override it;
 * `src/families/foundations/theme/style-pipeline.test.ts` pins the observable
 * output.
 */
export const cssBrowserFloor = ["chrome111", "edge111", "firefox113", "safari16.4"];

type CssAssetEntrypoint = {
  readonly fileName: `${string}.css`;
  readonly source: `src/${string}.css`;
};

type LightningCssModule = {
  readonly browserslistToTargets: (queries: readonly string[]) => Record<string, number>;
  readonly transform: (options: {
    readonly filename: string;
    readonly code: Uint8Array;
    readonly minify: boolean;
    readonly targets: Record<string, number>;
  }) => { readonly code: Uint8Array };
};

const cssAssetEntrypoints = Object.freeze([
  { fileName: "theme.css", source: "src/families/foundations/theme/theme.css" },
  {
    fileName: "theme-preset-headless.css",
    source: "src/families/foundations/theme/theme-preset-headless.css",
  },
  {
    fileName: "theme-preset-atelier.css",
    source: "src/families/foundations/theme/theme-preset-atelier.css",
  },
  {
    fileName: "theme-preset-midnight.css",
    source: "src/families/foundations/theme/theme-preset-midnight.css",
  },
  {
    fileName: "theme-preset-paper.css",
    source: "src/families/foundations/theme/theme-preset-paper.css",
  },
  {
    fileName: "theme-preset-play.css",
    source: "src/families/foundations/theme/theme-preset-play.css",
  },
  {
    fileName: "theme-preset-signal.css",
    source: "src/families/foundations/theme/theme-preset-signal.css",
  },
  {
    fileName: "theme-preset-high-contrast.css",
    source: "src/families/foundations/theme/theme-preset-high-contrast.css",
  },
  {
    fileName: "component-alert.css",
    source: "src/families/feedback/alert/alert-visual.css",
  },
  {
    fileName: "component-badge.css",
    source: "src/families/feedback/badge/badge-visual.css",
  },
  {
    fileName: "component-breadcrumb.css",
    source: "src/families/navigation/breadcrumb/breadcrumb-visual.css",
  },
  {
    fileName: "component-button.css",
    source: "src/families/actions/button/button-visual.css",
  },
  {
    fileName: "component-card.css",
    source: "src/families/layout/card/card-visual.css",
  },
  {
    fileName: "component-checkbox.css",
    source: "src/families/selection/checkbox/checkbox-visual.css",
  },
  {
    fileName: "component-dialog.css",
    source: "src/families/overlays/dialog/dialog-visual.css",
  },
  {
    fileName: "component-input.css",
    source: "src/families/form/input/input-visual.css",
  },
  {
    fileName: "component-pagination.css",
    source: "src/families/navigation/pagination/pagination-visual.css",
  },
  {
    fileName: "component-progress-bar.css",
    source: "src/families/feedback/progress-bar/progress-bar.css",
  },
  {
    fileName: "component-scroll-area.css",
    source: "src/families/layout/scroll-area/scroll-area.css",
  },
  {
    fileName: "component-switch.css",
    source: "src/families/selection/switch/switch-visual.css",
  },
  {
    fileName: "component-stepper.css",
    source: "src/families/navigation/stepper/stepper-visual.css",
  },
  {
    fileName: "component-tabs.css",
    source: "src/families/navigation/tabs/tabs-visual.css",
  },
  {
    fileName: "component-textarea.css",
    source: "src/families/form/textarea/textarea-visual.css",
  },
  {
    fileName: "component-tooltip.css",
    source: "src/families/overlays/tooltip/tooltip-visual.css",
  },
  {
    fileName: "motion.css",
    source: "src/families/overlays/motion/motion.css",
  },
] as const satisfies readonly CssAssetEntrypoint[]);
const themeLayerPrelude = "@layer vize.tokens,vize.ui,vize.preset,vize.policy;";
const browserTargetQueries = cssBrowserFloor.map((target) =>
  target.replace(/^([a-z]+)(\d)/, "$1 $2"),
);
const require = createRequire(import.meta.url);
const cssDecoder = new TextDecoder();

function loadLightningCss(): LightningCssModule {
  const cssRequire = createRequire(require.resolve("@tsdown/css/package.json"));
  return cssRequire("lightningcss") as LightningCssModule;
}

function cssAssetEntrypointPlugin(): Plugin {
  return {
    name: "vize-ui-css-assets",
    async generateBundle() {
      const { browserslistToTargets, transform } = loadLightningCss();
      const targets = browserslistToTargets(browserTargetQueries);

      await Promise.all(
        cssAssetEntrypoints.map(async ({ fileName, source }) => {
          const sourceCode = await readFile(new URL(source, import.meta.url));
          const output = transform({
            filename: source,
            code: sourceCode,
            minify: true,
            targets,
          });

          this.emitFile({
            type: "asset",
            fileName,
            source: `${themeLayerPrelude}${cssDecoder.decode(output.code)}`,
          });
        }),
      );
    },
  };
}

export default defineConfig({
  plugins: [uiStoryVariants(), vue()],
  test: {
    environment: "happy-dom",
    include: ["src/**/*.test.ts"],
  },
  lint: {
    ignorePatterns: ["dist/**"],
    options: { typeAware: true },
  },
  fmt: { ignorePatterns: ["dist/**"] },
  pack: {
    entry: {
      index: "src/index.ts",
      // Every catalogued family publishes its own subpath from its catalog entry file.
      ...Object.fromEntries(
        uiFamilyCatalog.map(({ canonicalName, entryFile }) => [canonicalName, entryFile]),
      ),
      "theme-scope": "src/families/foundations/theme/theme-scope.ts",
      resolver: "src/resolver/resolver.ts",
      "qr-code-kanji": "src/families/media/qr-code/qr-code-kanji.ts",
      catalog: "src/catalog/family-catalog.ts",
      media: "src/media/media.ts",
      "media-pdf": "src/media/pdf.ts",
      "media-source": "src/media/media-source.ts",
    },
    format: "esm",
    dts: { vue: true },
    plugins: [vue(), cssAssetEntrypointPlugin()],
    css: {
      inject: true,
      minify: true,
      target: cssBrowserFloor,
    },
    clean: true,
    deps: {
      neverBundle: ["vue"],
    },
  },
});
