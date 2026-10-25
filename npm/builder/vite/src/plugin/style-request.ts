import path from "node:path";

const STYLE_MARKER = ".__vize_style_";

/** Keep the CSS extension in the path, outside Vue/Nuxt's query metadata. */
export function createStyleVirtualId(id: string): string {
  const start = id.indexOf("?");
  const filename = id.slice(0, start);
  const params = new URLSearchParams(id.slice(start + 1));
  if (params.has("vize-file") && filename.includes(STYLE_MARKER)) {
    // Rolldown presents emitted entry IDs relative to root. Retain the producer's
    // exact absolute source path so Nuxt's inline style entry remains resolvable.
    const source = params.get("vize-file") ?? "";
    return `${source}${filename.slice(filename.lastIndexOf(STYLE_MARKER))}?${id.slice(start + 1).replaceAll(".", "%2E")}`;
  }
  const index = params.get("index") ?? "0";
  const lang = params.get("lang") || "css";
  const suffix = params.has("module") ? `.module.${lang}` : `.${lang}`;
  // Nuxt CSS discovery excludes dots in the query; URI-encode the source path's dots.
  const source = params.has("vize-file")
    ? ""
    : `&${new URLSearchParams({ "vize-file": filename }).toString().replaceAll(".", "%2E")}`;
  return `${filename}${STYLE_MARKER}${index}${suffix}?${(id.slice(start + 1) + source).replaceAll(".", "%2E")}`;
}

/** Give Vite's CSS resolvers a physical origin without changing emitted imports. */
export function resolveStyleVirtualId(id: string, root: string): string {
  const styleId = createStyleVirtualId(id);
  const queryStart = styleId.indexOf("?");
  const params = new URLSearchParams(styleId.slice(queryStart + 1));
  const source = params.get("vize-file");
  if (!params.has("vize-root-relative") || !source?.startsWith("/")) return styleId;

  const marker = styleId.slice(0, queryStart).lastIndexOf(STYLE_MARKER);
  const physicalSource = path.resolve(root, source.slice(1)).split(path.sep).join("/");
  return `${physicalSource}${styleId.slice(marker)}`;
}

/** Recover the authored SFC before native classification and cache lookup. */
export function normalizeStyleVirtualId(id: string): string {
  const withoutPrefix = id.startsWith("\0") ? id.slice(1) : id;
  const start = withoutPrefix.indexOf("?");
  if (start < 0) return id;
  const query = withoutPrefix.slice(start + 1);
  const params = new URLSearchParams(query);
  if (!params.has("vue")) return id;
  const source = params.get("vize-file");
  if (source && withoutPrefix.slice(0, start).includes(STYLE_MARKER)) {
    return `${source}?${query}`;
  }

  // Accept style IDs emitted by older versions of the plugin.
  return withoutPrefix.replace(/\.module\.\w+$/, "").replace(/\.\w+$/, "");
}
