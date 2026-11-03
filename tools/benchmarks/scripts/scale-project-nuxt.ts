/**
 * The Nuxt 4 shape of a generated scale project: everything under `app/`,
 * components nested so Nuxt auto-imports them with path-prefixed names
 * (`components/kit/Table.vue` is `<KitTable>`, `components/feature/f0001/Layer00.vue`
 * is `<FeatureF0001Layer00>`), composables auto-imported, one route per
 * feature with `definePageMeta` layouts. Global components need no
 * hand-written augmentation: Nuxt generates `.nuxt/components.d.ts`.
 */

import { type Source, featureDir } from "./scale-project-sfc.ts";

/** Physical path of a logical corpus path in the Nuxt layout. */
export function nuxtPlace(logical: string): string {
  const kit = /^kit\/Kit(\w+)\.vue$/u.exec(logical);
  if (kit) return `app/components/kit/${kit[1]}.vue`;
  const global = /^global\/G(\w+)\.vue$/u.exec(logical);
  if (global) return `app/components/g/${global[1]}.vue`;
  const feature = /^features\/(f\d+)\/(\w+)\.(vue|ts)$/u.exec(logical);
  if (feature) {
    const [, dir, name, extension] = feature;
    if (extension === "ts") return `app/utils/${dir}.ts`;
    return `app/components/feature/${dir}/${name === "Page" ? "Panel" : name}.vue`;
  }
  if (logical === "shared/types.ts") return "app/types/shared.ts";
  if (logical === "App.vue") return "app/app.vue";
  return `app/${logical}`;
}

const pascal = (segment: string) => segment.charAt(0).toUpperCase() + segment.slice(1);

/** The name Nuxt registers a component under: its path segments below `components/`, PascalCased. */
export function autoImportName(physical: string): string {
  const match = /^app\/components\/(.+)\.vue$/u.exec(physical);
  if (!match) throw new Error(`${physical} is not under app/components`);
  return (match[1] as string).split("/").map(pascal).join("");
}

export const NUXT_LAYOUTS = ["default", "wide"] as const;
const layoutOf = (k: number) => NUXT_LAYOUTS[k % NUXT_LAYOUTS.length] as string;

export function composable(k: number, rows: number, model: string): Source {
  return {
    lines: [
      'import { ref } from "vue";',
      `import { makeRows${k}, type Row${k} } from "${model}";`,
      "",
      `export function useFeature${k}() {`,
      `  const rows = ref<Row${k}[]>(makeRows${k}(${rows}));`,
      '  const query = ref("");',
      "  return { rows, query };",
      "}",
    ],
    plants: [],
  };
}

export function routePage(k: number): Source {
  return {
    lines: [
      '<script setup lang="ts">',
      `definePageMeta({ layout: "${layoutOf(k)}" });`,
      "</script>",
      "",
      "<template>",
      `  <${autoImportName(nuxtPlace(`${featureDir(k)}/Page.vue`))} />`,
      "</template>",
    ],
    plants: [],
  };
}

export function indexPage(): Source {
  return routePage(0);
}

export function layout(name: string): Source {
  return {
    lines: [
      '<script setup lang="ts">',
      `const title = useState("layout-title", () => "${name}");`,
      "</script>",
      "",
      "<template>",
      `  <div class="layout-${name}">`,
      '    <header><GIcon name="star" /> {{ title }}</header>',
      "    <slot />",
      "  </div>",
      "</template>",
    ],
    plants: [],
  };
}

export function nuxtApp(): Source {
  return {
    lines: ["<template>", "  <NuxtLayout>", "    <NuxtPage />", "  </NuxtLayout>", "</template>"],
    plants: [],
  };
}

/** A client-only build with no type check in `nuxt build`, strict templates for checkers. */
export const NUXT_CONFIG = `import { defineNuxtConfig } from "nuxt/config";

export default defineNuxtConfig({
  devtools: { enabled: false },
  telemetry: false,
  ssr: false,
  typescript: {
    typeCheck: false,
    tsConfig: { vueCompilerOptions: { strictTemplates: true } },
  },
});
`;

/** Nuxt 4's solution tsconfig; `nuxt prepare` writes the referenced configs. */
export const NUXT_TSCONFIG = {
  files: [],
  references: [
    { path: "./.nuxt/tsconfig.app.json" },
    { path: "./.nuxt/tsconfig.server.json" },
    { path: "./.nuxt/tsconfig.shared.json" },
    { path: "./.nuxt/tsconfig.node.json" },
  ],
};
/** The program the app's SFCs belong to, for checkers that take one project. */
export const NUXT_APP_TSCONFIG = ".nuxt/tsconfig.app.json";
