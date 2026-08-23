import fs from "node:fs";
import path from "node:path";

/**
 * Resolve a `previewCss` entry.
 *
 * Relative and project-root files stay filesystem paths. Bare specifiers
 * (`normalize.css`, `@fontsource/inter/index.css`) are left for Vite.
 */
export function resolvePreviewCssPath(root: string, cssPath: string): string {
  if (path.isAbsolute(cssPath)) {
    return cssPath;
  }

  const rooted = path.resolve(root, cssPath);
  if (cssPath.startsWith(".") || fs.existsSync(rooted)) {
    return rooted;
  }

  return cssPath;
}
