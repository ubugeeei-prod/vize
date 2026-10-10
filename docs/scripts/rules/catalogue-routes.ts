export const categoryFiles = [
  "type-and-script",
  "html",
  "accessibility",
  "ssr",
  "petite-vue",
  "vapor",
  "ecosystem",
  "musea-and-css",
] as const;
export type CategoryFile = (typeof categoryFiles)[number];

/** These are the same category membership boundaries as the existing reference. */
export function categoryIncludes(file: CategoryFile, id: string) {
  if (file === "type-and-script") return /^(?:script|type)\//.test(id);
  if (file === "html") return id.startsWith("html/");
  if (file === "accessibility")
    return id.startsWith("a11y/") || id === "vue/use-unique-element-ids";
  if (file === "ssr" || file === "petite-vue") return id.startsWith(`${file}/`);
  if (file === "vapor")
    return (
      id.startsWith("vapor/") ||
      ["script/no-options-api", "script/no-get-current-instance", "script/no-next-tick"].includes(
        id,
      )
    );
  if (file === "ecosystem") return /^(?:ecosystem|nuxt)\//.test(id);
  return /^(?:musea|css)\//.test(id);
}

export const englishCategoryTitles: Record<CategoryFile | "all" | "cross-file", string> = {
  all: "All lint rules",
  "cross-file": "Cross-file rules",
  "type-and-script": "Type and script rules",
  html: "HTML rules",
  accessibility: "Accessibility rules",
  ssr: "SSR rules",
  "petite-vue": "petite-vue rules",
  vapor: "Vapor rules",
  ecosystem: "Ecosystem rules",
  "musea-and-css": "Musea and CSS rules",
};
export const japaneseCategoryTitles: typeof englishCategoryTitles = {
  all: "全 lint ルール",
  "cross-file": "ファイル間ルール",
  "type-and-script": "型と script のルール",
  html: "HTML ルール",
  accessibility: "アクセシビリティ ルール",
  ssr: "SSR ルール",
  "petite-vue": "petite-vue ルール",
  vapor: "Vapor ルール",
  ecosystem: "エコシステム ルール",
  "musea-and-css": "Musea と CSS のルール",
};
