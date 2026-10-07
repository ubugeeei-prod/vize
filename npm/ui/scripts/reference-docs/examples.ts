/** Public-import versions of the examples maintained beside the component source. */
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

import { uiFamilyCatalog } from "../../src/catalog/family-catalog.ts";
import type { UiFamilyCatalogEntry } from "../../src/catalog/family-catalog-types.ts";

export const featuredExamples = [
  "button",
  "input",
  "checkbox",
  "switch",
  "tabs",
  "dialog",
  "alert",
  "card",
] as const;

export function publicPackage(entry: UiFamilyCatalogEntry): string {
  return `@vizejs/ui${entry.packageSubpath === "." ? "" : entry.packageSubpath.slice(1)}`;
}

/** Keep the entire real SFC; replace only catalog-owned module specifiers. */
export function publicExample(
  packageRoot: string,
  entry: UiFamilyCatalogEntry,
): string | undefined {
  const file = path.join(
    packageRoot,
    path.dirname(entry.entryFile),
    "examples",
    `${entry.canonicalName}-basic.vue`,
  );
  if (!existsSync(file)) return undefined;
  const byFile = new Map(
    uiFamilyCatalog.map((family) => [
      path.resolve(packageRoot, family.entryFile),
      publicPackage(family),
    ]),
  );
  return readFileSync(file, "utf8").replace(
    /(from\s+)(["'])(\.[^"']+)\2/g,
    (_match, from: string, quote: string, specifier: string) => {
      const target = byFile.get(path.resolve(path.dirname(file), specifier));
      if (target == null)
        throw new Error(
          `Example ${file} imports a module without a public catalog entry: ${specifier}`,
        );
      return `${from}${quote}${target}${quote}`;
    },
  );
}

export function previewMarkup(family: string, title: string, screenshot = false): string {
  const url = `/component-previews/app/index.html?family=${encodeURIComponent(family)}`;
  return [
    `<iframe src="${url}" title="${title} interactive example" loading="lazy" width="100%" height="440" style="border:1px solid #8885;border-radius:8px"></iframe>`,
    "",
    `[Open the interactive example](${url}) · [Styling these components](../ui-styles.md)`,
    screenshot
      ? `\n![${title} example rendered in a real browser](/component-previews/${family}.png)`
      : "",
  ].join("\n");
}
