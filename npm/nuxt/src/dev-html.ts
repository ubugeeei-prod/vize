export function sanitizeNuxtDevStylesheetLinks(html: string, buildAssetsDir = "/_nuxt/"): string {
  function normalizeUrlPrefix(value: string): string {
    const withLeadingSlash = value.startsWith("/") ? value : `/${value}`;
    return withLeadingSlash.endsWith("/") ? withLeadingSlash : `${withLeadingSlash}/`;
  }

  const normalizedAssetsDir = normalizeUrlPrefix(buildAssetsDir);
  const seenHrefs = new Set<string>();

  function decodePathPart(pathPart: string): string {
    try {
      return decodeURIComponent(pathPart);
    } catch {
      return pathPart;
    }
  }

  function hasUnsafePathSegment(pathPart: string): boolean {
    return pathPart.split(/[\\/]/).some((segment) => segment === "..");
  }

  function isAllowedNuxtDevStylesheetPath(pathPart: string): boolean {
    return (
      pathPart.startsWith("@fs/") ||
      pathPart.startsWith("@id/") ||
      pathPart.startsWith("assets/") ||
      pathPart.startsWith("virtual:") ||
      /^__[\w.-]+\.css$/i.test(pathPart) ||
      /^[\w.-]+\.css$/i.test(pathPart)
    );
  }

  function shouldKeepHref(href: string): boolean {
    if (seenHrefs.has(href)) {
      return false;
    }
    seenHrefs.add(href);

    if (!href.startsWith(normalizedAssetsDir)) {
      return true;
    }

    const pathPart = href.slice(normalizedAssetsDir.length).split("?")[0].split("#")[0];
    const decodedPath = decodePathPart(pathPart);

    if (decodedPath.includes("\0") || hasUnsafePathSegment(decodedPath)) {
      return false;
    }

    return isAllowedNuxtDevStylesheetPath(decodedPath);
  }

  return html.replace(
    /<link\b(?=[^>]*\brel=(["'])stylesheet\1)[^>]*\bhref=(["'])(.*?)\2[^>]*>/gi,
    (tag, _relQuote, _hrefQuote, href) => (shouldKeepHref(href) ? tag : ""),
  );
}
