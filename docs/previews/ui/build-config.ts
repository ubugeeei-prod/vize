/** Build real workspace examples against their catalog-declared public entries. */
import { createRequire } from "node:module";
import { createHash } from "node:crypto";
import path from "node:path";

import { uiFamilyCatalog } from "../../../npm/ui/src/catalog/family-catalog.ts";
import { publicExample, publicPackage } from "../../../npm/ui/scripts/reference-docs/examples.ts";
import { COMPOSABLE_CATALOG } from "../../../npm/compose/core/src/catalog.ts";
import {
  composableExamples,
  composableExampleSource,
} from "../../../npm/ui/scripts/reference-docs/composable-examples.ts";

const docsRoot = path.resolve(import.meta.dirname, "../..");
export const uiRoot = path.resolve(docsRoot, "../npm/ui");
export const uiRequire = createRequire(path.join(uiRoot, "package.json"));
export const previewExamples = uiFamilyCatalog.flatMap((entry) => {
  const source = publicExample(uiRoot, entry);
  return source == null
    ? []
    : [{ entry, source, sourceSha256: createHash("sha256").update(source).digest("hex") }];
});
const composableRoot = path.resolve(docsRoot, "../npm/compose/core");
export const previewComposableExamples = composableExamples.map((example) => {
  const source = composableExampleSource(composableRoot, example.name);
  return {
    ...example,
    source,
    sourceSha256: createHash("sha256").update(source).digest("hex"),
    ssrHtml: "",
  };
});

export async function previewBuildConfig() {
  const { default: vue } = await import(uiRequire.resolve("@vitejs/plugin-vue"));
  const themeRoot = path.join(uiRoot, "src/families/foundations/theme");
  const css = new Map<string, string>([
    ["base.css", path.join(themeRoot, "theme.css")],
    ...["paper", "signal", "atelier"].map((preset): [string, string] => [
      `theme-preset-${preset}.css`,
      path.join(themeRoot, `theme-preset-${preset}.css`),
    ]),
    ...["button", "input", "checkbox", "switch", "tabs", "dialog", "alert", "card"].map(
      (family): [string, string] => {
        const entry = uiFamilyCatalog.find((item) => item.canonicalName === family)!;
        return [
          `component-${family}.css`,
          path.join(uiRoot, path.dirname(entry.entryFile), `${family}-visual.css`),
        ];
      },
    ),
  ]);
  const sources = new Map([
    ...previewExamples.map(({ entry, source }): [string, string] => [
      `/virtual-vize-examples/${entry.canonicalName}.vue`,
      source,
    ]),
    ...previewComposableExamples.map(({ name, source }): [string, string] => [
      `/virtual-vize-composable-examples/${name}.vue`,
      source,
    ]),
  ]);
  return {
    configFile: false as const,
    root: import.meta.dirname,
    base: "/component-previews/app/",
    publicDir: false as const,
    logLevel: "warn" as const,
    plugins: [
      {
        name: "vize-docs-source-examples",
        resolveId(id: string) {
          if (id === "virtual:vize-ui-examples") return "\0vize-ui-examples";
          if (sources.has(id)) return id;
          return undefined;
        },
        load(id: string) {
          if (id === "\0vize-ui-examples") {
            return `export const examples = {${previewExamples.map(({ entry, sourceSha256 }) => `${JSON.stringify(entry.canonicalName)}: {title: ${JSON.stringify(entry.title)}, sourceSha256: ${JSON.stringify(sourceSha256)}, load: () => import(${JSON.stringify(`/virtual-vize-examples/${entry.canonicalName}.vue`)})}`).join(",")}}; export const composables = {${previewComposableExamples.map(({ name, title, sourceSha256, ssrHtml }) => `${JSON.stringify(name)}: {title: ${JSON.stringify(title)}, sourceSha256: ${JSON.stringify(sourceSha256)}, ssrHtml: ${JSON.stringify(ssrHtml)}, load: () => import(${JSON.stringify(`/virtual-vize-composable-examples/${name}.vue`)})}`).join(",")}};`;
          }
          return sources.get(id);
        },
      },
      vue(),
    ],
    resolve: {
      dedupe: ["vue"],
      alias: [
        { find: /^vue$/, replacement: uiRequire.resolve("vue/dist/vue.runtime.esm-bundler.js") },
        ...uiFamilyCatalog.map((entry) => ({
          find: new RegExp(`^${publicPackage(entry).replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}$`),
          replacement: path.join(uiRoot, entry.entryFile),
        })),
        ...[...css].map(([name, source]) => ({ find: `@vizejs/ui/${name}`, replacement: source })),
        ...COMPOSABLE_CATALOG.entries.map((entry) => ({
          find: new RegExp(`^@vizejs/composable/${entry.subpath.slice(2)}$`),
          replacement: path.join(composableRoot, entry.source),
        })),
      ],
    },
    build: {
      outDir: path.join(docsRoot, "public/component-previews/app"),
      emptyOutDir: true,
      target: "esnext",
      chunkSizeWarningLimit: 1000,
    },
  };
}
