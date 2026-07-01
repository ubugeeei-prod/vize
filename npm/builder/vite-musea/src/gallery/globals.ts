import { serializeScriptValue } from "../security.js";
import type { MuseaTokenPreviewConfig } from "../tokens/preview.js";

export interface MuseaGalleryGlobals {
  basePath: string;
  devSessionToken?: string;
  staticPreviews?: Record<string, Record<string, string>>;
  themeConfig?: { default: string; custom?: Record<string, unknown> };
  tokenPreviewConfig?: MuseaTokenPreviewConfig;
}

export function generateGalleryGlobalsScript(globals: MuseaGalleryGlobals): string {
  const parts = [`window.__MUSEA_BASE_PATH__=${serializeScriptValue(globals.basePath)};`];
  if (globals.devSessionToken !== undefined) {
    parts.push(`window.__MUSEA_SESSION_TOKEN__=${serializeScriptValue(globals.devSessionToken)};`);
  }
  if (globals.staticPreviews !== undefined) {
    parts.push("window.__MUSEA_STATIC__=true;");
    parts.push(`window.__MUSEA_STATIC_PREVIEWS__=${serializeScriptValue(globals.staticPreviews)};`);
  }
  if (globals.themeConfig) {
    parts.push(`window.__MUSEA_THEME_CONFIG__=${serializeScriptValue(globals.themeConfig)};`);
  }
  if (globals.tokenPreviewConfig) {
    parts.push(
      `window.__MUSEA_TOKEN_PREVIEWS__=${serializeScriptValue(globals.tokenPreviewConfig)};`,
    );
  }
  return parts.join("");
}

export function generateDevGlobalsScript(
  basePath: string,
  devSessionToken: string,
  themeConfig?: MuseaGalleryGlobals["themeConfig"],
  tokenPreviewConfig?: MuseaTokenPreviewConfig,
): string {
  return generateGalleryGlobalsScript({
    basePath,
    devSessionToken,
    themeConfig,
    tokenPreviewConfig,
  });
}
