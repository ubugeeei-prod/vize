const STYLE_MARKER = ".__vize_style_";

/** Keep the CSS extension in the path, outside Vue/Nuxt's query metadata. */
export function createStyleVirtualId(id: string): string {
  const start = id.indexOf("?");
  const filename = id.slice(0, start);
  const params = new URLSearchParams(id.slice(start + 1));
  if (params.has("vize-file") && filename.includes(STYLE_MARKER)) return id;
  const index = params.get("index") ?? "0";
  const lang = params.get("lang") || "css";
  const suffix = params.has("module") ? `.module.${lang}` : `.${lang}`;
  const source = params.has("vize-file")
    ? ""
    : `&${new URLSearchParams({ "vize-file": filename }).toString()}`;
  return `${filename}${STYLE_MARKER}${index}${suffix}?${id.slice(start + 1)}${source}`;
}

/** Recover the authored SFC before native classification and cache lookup. */
export function normalizeStyleVirtualId(id: string): string {
  const withoutPrefix = id.startsWith("\0") ? id.slice(1) : id;
  if (!withoutPrefix.includes("?vue")) return id;

  const start = withoutPrefix.indexOf("?");
  const query = withoutPrefix.slice(start + 1);
  const source = new URLSearchParams(query).get("vize-file");
  if (source && withoutPrefix.slice(0, start).includes(STYLE_MARKER)) {
    return `${source}?${query}`;
  }

  // Accept style IDs emitted by older versions of the plugin.
  return withoutPrefix.replace(/\.module\.\w+$/, "").replace(/\.\w+$/, "");
}
