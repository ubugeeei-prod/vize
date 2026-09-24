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
      "input-mask": "src/families/form/input-mask/input-mask.ts",
      "action-sheet": "src/families/overlays/action-sheet/action-sheet.ts",
      "angle-picker": "src/families/form/angle-picker/angle-picker.ts",
      "bottom-navigation": "src/families/navigation/bottom-navigation/bottom-navigation.ts",
      fieldset: "src/families/form/fieldset/fieldset.ts",
      "form-wizard": "src/families/form/form-wizard/form-wizard.ts",
      knob: "src/families/form/knob/knob.ts",
      "number-field": "src/families/form/number-field/number-field.ts",
      pager: "src/families/navigation/pager/pager.ts",
      "phone-field": "src/families/form/phone-field/phone-field.ts",
      "pull-to-refresh": "src/families/interaction/pull-to-refresh/pull-to-refresh.ts",
      "range-slider": "src/families/form/range-slider/range-slider.ts",
      "safe-area": "src/families/layout/safe-area/safe-area.ts",
      "swipe-actions": "src/families/interaction/swipe-actions/swipe-actions.ts",
      theme: "src/families/foundations/theme/theme.ts",
      "theme-scope": "src/families/foundations/theme/theme-scope.ts",
      resolver: "src/resolver/resolver.ts",
      alert: "src/families/feedback/alert/alert.ts",
      announcer: "src/families/accessibility/announcer/announcer.ts",
      "a11y-audit": "src/families/accessibility/a11y-audit/a11y-audit.ts",
      landmark: "src/families/accessibility/landmark/landmark.ts",
      "aspect-ratio": "src/families/layout/aspect-ratio/aspect-ratio.ts",
      avatar: "src/families/layout/avatar/avatar.ts",
      badge: "src/families/feedback/badge/badge.ts",
      banner: "src/families/feedback/banner/banner.ts",
      "block-ui": "src/families/feedback/block-ui/block-ui.ts",
      callout: "src/families/feedback/callout/callout.ts",
      scheduler: "src/families/date-time/scheduler/scheduler.ts",
      "datetime-field": "src/families/date-time/datetime-field/datetime-field.ts",
      "duration-field": "src/families/date-time/duration-field/duration-field.ts",
      "month-picker": "src/families/date-time/month-picker/month-picker.ts",
      "time-picker": "src/families/date-time/time-picker/time-picker.ts",
      "week-picker": "src/families/date-time/week-picker/week-picker.ts",
      "year-picker": "src/families/date-time/year-picker/year-picker.ts",
      calendar: "src/families/date-time/calendar/calendar.ts",
      "date-field": "src/families/date-time/date-field/date-field.ts",
      "date-picker": "src/families/date-time/date-picker/date-picker.ts",
      "date-range-picker": "src/families/date-time/date-range-picker/date-range-picker.ts",
      "range-calendar": "src/families/date-time/range-calendar/range-calendar.ts",
      "time-field": "src/families/date-time/time-field/time-field.ts",
      blockquote: "src/families/typography/blockquote/blockquote.ts",
      breadcrumb: "src/families/navigation/breadcrumb/breadcrumb.ts",
      "checkbox-group": "src/families/selection/checkbox-group/checkbox-group.ts",
      editable: "src/families/form/editable/editable.ts",
      "password-field": "src/families/form/password-field/password-field.ts",
      "pin-input": "src/families/form/pin-input/pin-input.ts",
      tabs: "src/families/navigation/tabs/tabs.ts",
      tree: "src/families/data/tree/tree.ts",
      stepper: "src/families/navigation/stepper/stepper.ts",
      card: "src/families/layout/card/card.ts",
      code: "src/families/typography/code/code.ts",
      cluster: "src/families/layout/cluster/cluster.ts",
      container: "src/families/layout/container/container.ts",
      grid: "src/families/layout/grid/grid.ts",
      "empty-state": "src/families/feedback/empty-state/empty-state.ts",
      collapsible: "src/families/disclosure/collapsible/collapsible.ts",
      accordion: "src/families/disclosure/accordion/accordion.ts",
      button: "src/families/actions/button/button.ts",
      "floating-action-button":
        "src/families/actions/floating-action-button/floating-action-button.ts",
      "back-to-top": "src/families/actions/back-to-top/back-to-top.ts",
      "button-group": "src/families/actions/button-group/button-group.ts",
      "copy-button": "src/families/actions/copy-button/copy-button.ts",
      "fullscreen-button": "src/families/actions/fullscreen-button/fullscreen-button.ts",
      "print-button": "src/families/actions/print-button/print-button.ts",
      "share-button": "src/families/actions/share-button/share-button.ts",
      toolbar: "src/families/actions/toolbar/toolbar.ts",
      "data-grid": "src/families/data/data-grid/data-grid.ts",
      "grid-list": "src/families/data/grid-list/grid-list.ts",
      kanban: "src/families/data/kanban/kanban.ts",
      "rich-text": "src/families/editor/rich-text/rich-text.ts",
      table: "src/families/data/table/table.ts",
      link: "src/families/navigation/link/link.ts",
      "skip-link": "src/families/navigation/skip-link/skip-link.ts",
      toggle: "src/families/selection/toggle/toggle.ts",
      "toggle-group": "src/families/selection/toggle-group/toggle-group.ts",
      "context-menu": "src/families/menus/context-menu/context-menu.ts",
      "dropdown-menu": "src/families/menus/dropdown-menu/dropdown-menu.ts",
      menu: "src/families/menus/menu/menu.ts",
      menubar: "src/families/menus/menubar/menubar.ts",
      popover: "src/families/overlays/popover/popover.ts",
      popconfirm: "src/families/overlays/popconfirm/popconfirm.ts",
      tooltip: "src/families/overlays/tooltip/tooltip.ts",
      "hover-card": "src/families/overlays/hover-card/hover-card.ts",
      toast: "src/families/feedback/toast/toast.ts",
      "notification-center": "src/families/feedback/notification-center/notification-center.ts",
      input: "src/families/form/input/input.ts",
      "radio-group": "src/families/selection/radio-group/radio-group.ts",
      rating: "src/families/form/rating/rating.ts",
      "audio-player": "src/families/media/audio-player/audio-player.ts",
      "audio-visualizer": "src/families/media/audio-visualizer/audio-visualizer.ts",
      "avatar-group": "src/families/layout/avatar-group/avatar-group.ts",
      carousel: "src/families/media/carousel/carousel.ts",
      "color-picker": "src/families/form/color-picker/color-picker.ts",
      "file-upload": "src/families/form/file-upload/file-upload.ts",
      hotspot: "src/families/media/hotspot/hotspot.ts",
      "image-compare": "src/families/media/image-compare/image-compare.ts",
      "image-cropper": "src/families/media/image-cropper/image-cropper.ts",
      image: "src/families/media/image/image.ts",
      "infinite-scroll": "src/families/data/infinite-scroll/infinite-scroll.ts",
      lightbox: "src/families/media/lightbox/lightbox.ts",
      marquee: "src/families/media/marquee/marquee.ts",
      "media-player": "src/families/media/media-player/media-player.ts",
      "pan-zoom": "src/families/media/pan-zoom/pan-zoom.ts",
      "qr-code": "src/families/media/qr-code/qr-code.ts",
      "scrubber-preview": "src/families/media/scrubber-preview/scrubber-preview.ts",
      "webcam-capture": "src/families/media/webcam-capture/webcam-capture.ts",
      "qr-code-kanji": "src/families/media/qr-code/qr-code-kanji.ts",
      "signature-pad": "src/families/media/signature-pad/signature-pad.ts",
      tour: "src/families/overlays/tour/tour.ts",
      "video-player": "src/families/media/video-player/video-player.ts",
      "search-field": "src/families/form/search-field/search-field.ts",
      slider: "src/families/form/slider/slider.ts",
      separator: "src/families/layout/separator/separator.ts",
      "dashboard-grid": "src/families/layout/dashboard-grid/dashboard-grid.ts",
      masonry: "src/families/layout/masonry/masonry.ts",
      "master-detail": "src/families/layout/master-detail/master-detail.ts",
      responsive: "src/families/layout/responsive/responsive.ts",
      "sticky-stack": "src/families/layout/sticky-stack/sticky-stack.ts",
      splitter: "src/families/layout/splitter/splitter.ts",
      "window-manager": "src/families/layout/window-manager/window-manager.ts",
      spacer: "src/families/layout/spacer/spacer.ts",
      stack: "src/families/layout/stack/stack.ts",
      sidebar: "src/families/layout/sidebar/sidebar.ts",
      resizable: "src/families/layout/resizable/resizable.ts",
      sticky: "src/families/layout/sticky/sticky.ts",
      "scroll-area": "src/families/layout/scroll-area/scroll-area.ts",
      surface: "src/families/layout/surface/surface.ts",
      skeleton: "src/families/feedback/skeleton/skeleton.ts",
      meter: "src/families/feedback/meter/meter.ts",
      "native-select": "src/families/selection/native-select/native-select.ts",
      heading: "src/families/typography/heading/heading.ts",
      kbd: "src/families/typography/kbd/kbd.ts",
      list: "src/families/layout/list/list.ts",
      listbox: "src/families/selection/listbox/listbox.ts",
      cascader: "src/families/selection/cascader/cascader.ts",
      "emoji-picker": "src/families/selection/emoji-picker/emoji-picker.ts",
      "transfer-list": "src/families/selection/transfer-list/transfer-list.ts",
      mention: "src/families/form/mention/mention.ts",
      "listbox-grid": "src/families/selection/listbox-grid/listbox-grid.ts",
      autocomplete: "src/families/selection/autocomplete/autocomplete.ts",
      combobox: "src/families/selection/combobox/combobox.ts",
      select: "src/families/selection/select/select.ts",
      pagination: "src/families/navigation/pagination/pagination.ts",
      text: "src/families/typography/text/text.ts",
      "tags-input": "src/families/form/tags-input/tags-input.ts",
      textarea: "src/families/form/textarea/textarea.ts",
      switch: "src/families/selection/switch/switch.ts",
      checkbox: "src/families/selection/checkbox/checkbox.ts",
      collection: "src/families/foundations/collection/collection.ts",
      "composite-navigation":
        "src/families/foundations/composite-navigation/composite-navigation.ts",
      context: "src/families/foundations/context/context.ts",
      forwarding: "src/families/foundations/forwarding/forwarding.ts",
      polymorphic: "src/families/foundations/polymorphic/polymorphic.ts",
      "slot-utils": "src/families/foundations/slot-utils/slot-utils.ts",
      variants: "src/families/foundations/variants/variants.ts",
      "controllable-state": "src/families/foundations/controllable-state/controllable-state.ts",
      dialog: "src/families/overlays/dialog/dialog.ts",
      "alert-dialog": "src/families/overlays/alert-dialog/alert-dialog.ts",
      confirm: "src/families/overlays/confirm/confirm.ts",
      drawer: "src/families/overlays/drawer/drawer.ts",
      "command-palette": "src/families/overlays/command-palette/command-palette.ts",
      "dismissable-layer": "src/families/overlays/dismissable-layer/dismissable-layer.ts",
      "drag-and-drop": "src/families/interaction/drag-and-drop/drag-and-drop.ts",
      "error-summary": "src/families/form/error-summary/error-summary.ts",
      icon: "src/families/layout/icon/icon.ts",
      "icon-button": "src/families/layout/icon/icon-button.ts",
      field: "src/families/form/field/field.ts",
      "field-wiring": "src/families/form/field-wiring/field-wiring.ts",
      form: "src/families/form/form/form.ts",
      catalog: "src/catalog/family-catalog.ts",
      command: "src/families/foundations/command/command.ts",
      history: "src/families/interaction/history/history.ts",
      id: "src/families/foundations/id/id.ts",
      "inert-outside": "src/families/accessibility/inert-outside/inert-outside.ts",
      "focus-scope": "src/families/accessibility/focus-scope/focus-scope.ts",
      "focus-guards": "src/families/accessibility/focus-guards/focus-guards.ts",
      hover: "src/families/interaction/hover/hover.ts",
      "live-region": "src/families/accessibility/live-region/live-region.ts",
      locale: "src/families/i18n/locale/locale.ts",
      direction: "src/families/i18n/direction/direction.ts",
      "long-press": "src/families/interaction/long-press/long-press.ts",
      measure: "src/families/interaction/measure/measure.ts",
      motion: "src/families/overlays/motion/motion.ts",
      move: "src/families/interaction/move/move.ts",
      "pointer-grace": "src/families/interaction/pointer-grace/pointer-grace.ts",
      portal: "src/families/overlays/portal/portal.ts",
      positioner: "src/families/overlays/positioner/positioner.ts",
      presence: "src/families/overlays/presence/presence.ts",
      progress: "src/families/feedback/progress/progress.ts",
      "progress-bar": "src/families/feedback/progress-bar/progress-bar.ts",
      spinner: "src/families/feedback/spinner/spinner.ts",
      "status-light": "src/families/feedback/status-light/status-light.ts",
      press: "src/families/interaction/press/press.ts",
      "interaction-modality":
        "src/families/accessibility/interaction-modality/interaction-modality.ts",
      "media-preferences": "src/families/accessibility/media-preferences/media-preferences.ts",
      focus: "src/families/accessibility/focus/focus.ts",
      "focus-visible": "src/families/accessibility/focus-visible/focus-visible.ts",
      "interaction-hooks": "src/families/interaction/interaction-hooks/interaction-hooks.ts",
      "scroll-lock": "src/families/accessibility/scroll-lock/scroll-lock.ts",
      shortcut: "src/families/interaction/shortcut/shortcut.ts",
      sortable: "src/families/interaction/sortable/sortable.ts",
      "spatial-navigation": "src/families/interaction/spatial-navigation/spatial-navigation.ts",
      transition: "src/families/overlays/transition/transition.ts",
      typeahead: "src/families/interaction/typeahead/typeahead.ts",
      virtualizer: "src/families/interaction/virtualizer/virtualizer.ts",
      chart: "src/families/charts/chart/chart.ts",
      "chart-shape": "src/families/charts/chart-shape/chart-shape.ts",
      "chart-scale": "src/families/charts/chart-scale/chart-scale.ts",
      "navigation-menu": "src/families/navigation/navigation-menu/navigation-menu.ts",
      "scroll-spy": "src/families/interaction/scroll-spy/scroll-spy.ts",
      toc: "src/families/navigation/toc/toc.ts",
      timeline: "src/families/data/timeline/timeline.ts",
      primitive: "src/families/foundations/primitive/primitive.ts",
      "visually-hidden": "src/families/accessibility/visually-hidden/visually-hidden.ts",
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
