/**
 * Gallery HTML generation for the Musea component gallery.
 *
 * Contains the inline gallery SPA template (used as a fallback when the
 * pre-built gallery is not available) and the gallery virtual module.
 */

import { generateGalleryStyles } from "./styles.js";
import { generateGalleryBody, generateGalleryScript } from "./template.js";
import { serializeScriptValue } from "../security.js";

/**
 * Generate the inline gallery HTML page.
 */
export function generateGalleryHtml(
  basePath: string,
  devSessionToken: string,
  themeConfig?: { default: string; custom?: Record<string, unknown> },
): string {
  const themeScript = themeConfig
    ? `window.__MUSEA_THEME_CONFIG__=${serializeScriptValue(themeConfig)};`
    : "";
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Musea - Component Gallery</title>
  <script>window.__MUSEA_BASE_PATH__=${serializeScriptValue(basePath)};window.__MUSEA_SESSION_TOKEN__=${serializeScriptValue(devSessionToken)};${themeScript}${"<"}/script>
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
