/**
 * Gallery HTML generation for the Musea component gallery.
 *
 * Contains the inline gallery SPA template (used as a fallback when the
 * pre-built gallery is not available) and the gallery virtual module.
 */

import { generateGalleryStyles } from "./styles.js";
import { generateGalleryBody, generateGalleryScript } from "./template.js";
import { generateGalleryGlobalsScript } from "./globals.js";
import { serializeScriptValue } from "../security.js";
import type { MuseaTokenPreviewConfig } from "../tokens/preview.js";

/**
 * Generate the inline gallery HTML page.
 */
export function generateGalleryHtml(
  basePath: string,
  devSessionToken: string,
  themeConfig?: { default: string; custom?: Record<string, unknown> },
  tokenPreviewConfig?: MuseaTokenPreviewConfig,
): string {
  const globalsScript = generateGalleryGlobalsScript({
    basePath,
    devSessionToken,
    themeConfig,
    tokenPreviewConfig,
  });
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Musea - Component Gallery</title>
  <script>${globalsScript}${"<"}/script>
  <style>${generateGalleryStyles()}
  </style>
</head>
<body>${generateGalleryBody(basePath)}

  <script type="module">${generateGalleryScript(basePath)}
  </script>
</body>
</html>`;
}

/**
 * Generate the virtual gallery module code.
 */
export function generateGalleryModule(basePath: string): string {
  return `
export const basePath = ${serializeScriptValue(basePath)};
export async function loadArts() {
  const res = await fetch(basePath + '/api/arts');
  return res.json();
}
`;
}
