import { defineNitroPlugin } from "nitropack/runtime";
import { devAssetBase } from "#vizejs/nuxt/dev-stylesheet-links-config";
//#region src/dev-html.ts
function sanitizeNuxtDevStylesheetLinks(html, buildAssetsDir = "/_nuxt/") {
  function normalizeUrlPrefix(value) {
    const withLeadingSlash = value.startsWith("/") ? value : `/${value}`;
    return withLeadingSlash.endsWith("/") ? withLeadingSlash : `${withLeadingSlash}/`;
  }
  const normalizedAssetsDir = normalizeUrlPrefix(buildAssetsDir);
  const seenHrefs = /* @__PURE__ */ new Set();
  function decodePathPart(pathPart) {
    try {
      return decodeURIComponent(pathPart);
    } catch {
      return pathPart;
    }
  }
  function hasUnsafePathSegment(pathPart) {
    return pathPart.split(/[\\/]/).some((segment) => segment === "..");
  }
  function isAllowedNuxtDevStylesheetPath(pathPart) {
    return (
      pathPart.startsWith("@fs/") ||
      pathPart.startsWith("@id/") ||
      pathPart.startsWith("assets/") ||
      pathPart.startsWith("virtual:") ||
      /^__[\w.-]+\.css$/i.test(pathPart) ||
      /^[\w.-]+\.css$/i.test(pathPart)
    );
  }
  function shouldKeepHref(href) {
    if (seenHrefs.has(href)) return false;
    seenHrefs.add(href);
    if (!href.startsWith(normalizedAssetsDir)) return true;
    const pathPart = href.slice(normalizedAssetsDir.length).split("?")[0].split("#")[0];
    const decodedPath = decodePathPart(pathPart);
    if (decodedPath.includes("\0") || hasUnsafePathSegment(decodedPath)) return false;
    return isAllowedNuxtDevStylesheetPath(decodedPath);
  }
  return html.replace(
    /<link\b(?=[^>]*\brel=(["'])stylesheet\1)[^>]*\bhref=(["'])(.*?)\2[^>]*>/gi,
    (tag, _relQuote, _hrefQuote, href) => (shouldKeepHref(href) ? tag : ""),
  );
}
//#endregion
//#region src/runtime/server/dev-stylesheet-links.ts
var dev_stylesheet_links_default = defineNitroPlugin((nitroApp) => {
  nitroApp.hooks.hook("render:response", (response) => {
    if (typeof response?.body !== "string" || !response.body.includes("<link")) return;
    response.body = sanitizeNuxtDevStylesheetLinks(response.body, devAssetBase);
  });
});
//#endregion
export { dev_stylesheet_links_default as default };
