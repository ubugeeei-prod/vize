import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { ResolvedConfig } from "vite";

const moduleDir = path.dirname(fileURLToPath(import.meta.url));
const galleryImports = [
  "vue",
  "vue-router",
  "@mdi/js",
  "marked",
  "highlight.js/lib/core",
  "highlight.js/lib/languages/xml",
  "highlight.js/lib/languages/json",
  "highlight.js/lib/languages/javascript",
  "highlight.js/lib/languages/typescript",
  "highlight.js/lib/languages/css",
  "highlight.js/lib/languages/bash",
];

export async function registerSourceGalleryDependencies(config: ResolvedConfig): Promise<void> {
  const optimizer = config.optimizeDeps;
  if (
    config.command !== "serve" ||
    optimizer.disabled === true ||
    optimizer.disabled === "dev" ||
    (optimizer.noDiscovery && !optimizer.include?.length) ||
    existsSync(path.join(moduleDir, "gallery/index.html")) ||
    !existsSync(path.join(moduleDir, "../gallery/index.html"))
  ) {
    return;
  }

  // Middleware serves this source graph outside the application's HTML scan.
  // Resolve from its real importer so package-local dependencies and user
  // aliases join the first optimizer generation without replacing app entries.
  const resolve = config.createResolver();
  const importer = path.join(moduleDir, "../gallery/main.ts");
  const sources = await Promise.all(
    galleryImports
      .filter(
        (id) =>
          !optimizer.exclude?.some((excluded) => id === excluded || id.startsWith(excluded + "/")),
      )
      .map((id) => resolve(id, importer)),
  );
  optimizer.include = [
    ...new Set([
      ...(optimizer.include ?? []),
      ...sources.filter(
        (source): source is string => source !== undefined && !optimizer.exclude?.includes(source),
      ),
    ]),
  ];
}
