import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";

import vue from "@vitejs/plugin-vue";
import type { Plugin } from "vite";
import { defineConfig } from "vite-plus";

/**
 * Declared browser floor for the packaged stylesheet.
 *
 * Styles are authored in native CSS (nesting, cascade layers, logical
 * properties, native color functions) and down-compiled to this floor by the
 * package build, so `dist/style.css` never depends on the consumer's own CSS
 * toolchain. The floor is the earliest evergreen release line where cascade
 * layers, `:where()`, logical properties, and `oklch()` are all native; only
 * CSS Nesting is newer than the floor and is therefore always flattened.
 * `src/style-pipeline.behavior.md` documents the policy and how consumers
 * override it; `src/style-pipeline.test.ts` pins the observable output.
 */
export const cssBrowserFloor = ["chrome111", "edge111", "firefox113", "safari16.4"];

type ThemeStyleEntrypoint = {
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

const themeStyleEntrypoints = Object.freeze([
  { fileName: "theme.css", source: "src/theme.css" },
  { fileName: "theme-preset-headless.css", source: "src/theme-preset-headless.css" },
  { fileName: "theme-preset-atelier.css", source: "src/theme-preset-atelier.css" },
  { fileName: "theme-preset-midnight.css", source: "src/theme-preset-midnight.css" },
  { fileName: "theme-preset-paper.css", source: "src/theme-preset-paper.css" },
  { fileName: "theme-preset-play.css", source: "src/theme-preset-play.css" },
  { fileName: "theme-preset-signal.css", source: "src/theme-preset-signal.css" },
  { fileName: "theme-preset-high-contrast.css", source: "src/theme-preset-high-contrast.css" },
] as const satisfies readonly ThemeStyleEntrypoint[]);
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

function themeCssEntrypointPlugin(): Plugin {
  return {
    name: "vize-ui-theme-css-entrypoints",
    async generateBundle() {
      const { browserslistToTargets, transform } = loadLightningCss();
      const targets = browserslistToTargets(browserTargetQueries);

      await Promise.all(
        themeStyleEntrypoints.map(async ({ fileName, source }) => {
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
  plugins: [vue()],
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
      theme: "src/theme.ts",
      "theme-scope": "src/theme-scope.ts",
      alert: "src/families/feedback/alert/alert.ts",
      announcer: "src/announcer.ts",
      "aspect-ratio": "src/families/layout/aspect-ratio/aspect-ratio.ts",
      avatar: "src/families/layout/avatar/avatar.ts",
      badge: "src/families/feedback/badge/badge.ts",
      banner: "src/families/feedback/banner/banner.ts",
      "block-ui": "src/families/feedback/block-ui/block-ui.ts",
      callout: "src/families/feedback/callout/callout.ts",
      blockquote: "src/families/typography/blockquote/blockquote.ts",
      breadcrumb: "src/breadcrumb.ts",
      tabs: "src/tabs.ts",
      stepper: "src/stepper.ts",
      card: "src/families/layout/card/card.ts",
      code: "src/families/typography/code/code.ts",
      cluster: "src/families/layout/cluster/cluster.ts",
      container: "src/families/layout/container/container.ts",
      grid: "src/families/layout/grid/grid.ts",
      "empty-state": "src/families/feedback/empty-state/empty-state.ts",
      collapsible: "src/collapsible.ts",
      button: "src/button.ts",
      "button-group": "src/families/actions/button-group/button-group.ts",
      "copy-button": "src/families/actions/copy-button/copy-button.ts",
      "fullscreen-button": "src/families/actions/fullscreen-button/fullscreen-button.ts",
      "print-button": "src/families/actions/print-button/print-button.ts",
      "share-button": "src/families/actions/share-button/share-button.ts",
      toolbar: "src/families/actions/toolbar/toolbar.ts",
      table: "src/families/data/table/table.ts",
      link: "src/link.ts",
      "skip-link": "src/families/navigation/skip-link/skip-link.ts",
      toggle: "src/toggle.ts",
      "toggle-group": "src/toggle-group.ts",
      popover: "src/families/overlays/popover/popover.ts",
      tooltip: "src/families/overlays/tooltip/tooltip.ts",
      input: "src/input.ts",
      "radio-group": "src/radio-group.ts",
      rating: "src/families/form/rating/rating.ts",
      "search-field": "src/search-field.ts",
      slider: "src/families/form/slider/slider.ts",
      separator: "src/families/layout/separator/separator.ts",
      spacer: "src/families/layout/spacer/spacer.ts",
      stack: "src/families/layout/stack/stack.ts",
      "scroll-area": "src/families/layout/scroll-area/scroll-area.ts",
      surface: "src/families/layout/surface/surface.ts",
      skeleton: "src/families/feedback/skeleton/skeleton.ts",
      meter: "src/families/feedback/meter/meter.ts",
      "native-select": "src/families/selection/native-select/native-select.ts",
      heading: "src/families/typography/heading/heading.ts",
      kbd: "src/families/typography/kbd/kbd.ts",
      list: "src/families/layout/list/list.ts",
      listbox: "src/listbox.ts",
      pagination: "src/pagination.ts",
      text: "src/families/typography/text/text.ts",
      textarea: "src/textarea.ts",
      switch: "src/switch.ts",
      checkbox: "src/checkbox.ts",
      collection: "src/collection.ts",
      "composite-navigation": "src/composite-navigation.ts",
      context: "src/context.ts",
      "controllable-state": "src/controllable-state.ts",
      dialog: "src/families/overlays/dialog/dialog.ts",
      "alert-dialog": "src/alert-dialog.ts",
      "dismissable-layer": "src/dismissable-layer.ts",
      "drag-and-drop": "src/drag-and-drop.ts",
      "error-summary": "src/error-summary.ts",
      icon: "src/families/layout/icon/icon.ts",
      "icon-button": "src/families/layout/icon/icon-button.ts",
      field: "src/field.ts",
      "field-wiring": "src/field-wiring.ts",
      form: "src/form.ts",
      catalog: "src/family-catalog.ts",
      command: "src/command.ts",
      history: "src/history.ts",
      id: "src/id.ts",
      "inert-outside": "src/inert-outside.ts",
      "interaction-modality": "src/interaction-modality.ts",
      focus: "src/focus.ts",
      "focus-scope": "src/focus-scope.ts",
      "focus-guards": "src/focus-guards.ts",
      hover: "src/hover.ts",
      "live-region": "src/live-region.ts",
      locale: "src/families/i18n/locale/locale.ts",
      "long-press": "src/long-press.ts",
      measure: "src/measure.ts",
      motion: "src/motion.ts",
      move: "src/move.ts",
      "pointer-grace": "src/pointer-grace.ts",
      portal: "src/portal.ts",
      positioner: "src/positioner.ts",
      presence: "src/presence.ts",
      progress: "src/families/feedback/progress/progress.ts",
      "progress-bar": "src/families/feedback/progress-bar/progress-bar.ts",
      spinner: "src/families/feedback/spinner/spinner.ts",
      "status-light": "src/families/feedback/status-light/status-light.ts",
      press: "src/press.ts",
      "scroll-lock": "src/scroll-lock.ts",
      shortcut: "src/shortcut.ts",
      sortable: "src/sortable.ts",
      "spatial-navigation": "src/spatial-navigation.ts",
      transition: "src/transition.ts",
      typeahead: "src/typeahead.ts",
      virtualizer: "src/virtualizer.ts",
      primitive: "src/primitive.ts",
      "visually-hidden": "src/visually-hidden.ts",
      media: "src/media.ts",
      "media-pdf": "src/pdf.ts",
      "media-source": "src/media-source.ts",
    },
    format: "esm",
    dts: { vue: true },
    plugins: [vue(), themeCssEntrypointPlugin()],
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
