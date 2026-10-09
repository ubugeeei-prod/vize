import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

/** Derivative inputs compose existing routes and are not separate site pages. */
const catalogues: readonly (readonly [string, string])[] = [
  ["generated/rules/en/all.md", "rules/all.md"],
  ["generated/rules/ja/all.md", "ja/rules/all.md"],
  ...["en", "ja", "zh-CN", "pt-BR", "fr"].map((locale): [string, string] => [
    `generated/rules/${locale}/vue.md`,
    `${locale === "en" ? "" : `${locale}/`}rules/vue.md`,
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
