import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { categoryFiles } from "./rules/catalogue-routes.ts";
import { catalogueSubgroups } from "./rules/catalogue-subgroups.ts";
import { vueLocales } from "./rules/vue-category-labels.ts";

/** Derivative inputs compose existing routes and are not separate site pages. */
const catalogues: readonly (readonly [string, string])[] = [
  ...vueLocales.flatMap((locale) =>
    ["all", "vue", ...categoryFiles, "cross-file"].map((file): [string, string] => [
      `generated/rules/${locale}/${file}.md`,
      `${locale === "en" ? "" : `${locale}/`}rules/${file}.md`,
    ]),
  ),
  ...Object.keys(catalogueSubgroups).map((file): [string, string] => [
    `generated/rules/en/${file}.md`,
    `rules/${file}.md`,
  ]),
];
export const CATALOGUE_SOURCES = catalogues.map(([source]) => source);

/** Keep native Ox Content routing while composing generated catalogue pages. */
export function materializeContent(docsRoot: string) {
  const directory = resolve(docsRoot, ".generated/content");
  rmSync(directory, { recursive: true, force: true });
  mkdirSync(directory, { recursive: true });
  const derivatives = new Set(CATALOGUE_SOURCES.map((path) => resolve(docsRoot, "content", path)));
  cpSync(resolve(docsRoot, "content"), directory, {
    recursive: true,
    filter: (source) => !derivatives.has(source),
  });
  for (const [source, target] of catalogues) {
    const catalogue = readFileSync(resolve(docsRoot, "content", source), "utf8");
    writeFileSync(resolve(directory, target), catalogue);
  }
  return directory;
}
