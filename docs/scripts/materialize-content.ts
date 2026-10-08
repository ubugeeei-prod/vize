import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

/** Derivative inputs compose existing routes and are not separate site pages. */
export const CATALOGUE_SOURCES = ["generated/rules/en/all.md", "generated/rules/ja/all.md"];

/** Keep native Ox Content routing while composing generated catalogue pages. */
export function materializeContent(docsRoot: string) {
  const directory = resolve(docsRoot, ".generated/content");
  rmSync(directory, { recursive: true, force: true });
  mkdirSync(directory, { recursive: true });
  const catalogues = new Set(CATALOGUE_SOURCES.map((path) => resolve(docsRoot, "content", path)));
  cpSync(resolve(docsRoot, "content"), directory, {
    recursive: true,
    filter: (source) => !catalogues.has(source),
  });
  for (const locale of ["en", "ja"]) {
    const catalogue = readFileSync(
      resolve(docsRoot, `content/generated/rules/${locale}/all.md`),
      "utf8",
    );
    writeFileSync(resolve(directory, `${locale === "en" ? "" : "ja/"}rules/all.md`), catalogue);
  }
  return directory;
}
