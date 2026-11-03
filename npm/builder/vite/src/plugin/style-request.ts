import fs from "node:fs";
import path from "node:path";

const STYLE_MARKER = ".__vize_style_";

function normalizeInlineStyleModifiers(id: string): string {
  const query = id.indexOf("?");
  const appended = id.lastIndexOf("?inline&used");
  if (
    query < 0 ||
    appended <= query ||
    !id.endsWith("?inline&used") ||
    !id.slice(0, query).includes(STYLE_MARKER)
  ) {
    return id;
  }
  const params = new URLSearchParams(id.slice(query + 1, appended));
  let lastKey: string | undefined;
  for (const key of params.keys()) lastKey = key;
  if (
    !params.has("vue") ||
    params.get("type") !== "style" ||
    params.getAll("type").length !== 1 ||
    lastKey !== "vize-file" ||
    params.getAll("vize-file").length !== 1 ||
    !params.get("vize-file")?.endsWith(".vue")
  )
    return id;
  // Nuxt appends this CSS transport suffix even when the ID already has a
  // query. Keep it out of the preceding vize-file value before reconstruction.
  return `${id.slice(0, appended)}&inline&used`;
}

/**
 * A `<style src>` block's module: the src file itself, with a style query
 * whose `vize-file` names the owning SFC (see createStyleImportUrl).
 */
export function isStyleSrcModuleId(id: string): boolean {
  const start = id.indexOf("?");
  if (start < 0) return false;
  const file = id.slice(0, start);
  if (file.includes(STYLE_MARKER) || file.endsWith(".vue")) return false;
  const params = new URLSearchParams(id.slice(start + 1));
  return params.has("vue") && params.get("type") === "style" && params.has("vize-file");
}

/**
 * The src file's own path for a `<style src>` module requested root-relative
 * (a production import), relative to the root (Nuxt's inline style entries)
 * or through `/@fs`.
 */
export function resolveStyleSrcModuleId(id: string, root: string): string {
  const start = id.indexOf("?");
  let file = id.slice(0, start);
  if (file.startsWith("/@fs/")) file = file.slice("/@fs".length);
  const physical =
    path.isAbsolute(file) && fs.existsSync(file)
      ? file
      : path.resolve(root, file.replace(/^\//, ""));
  return `${physical.split(path.sep).join("/")}${id.slice(start)}`;
}

/** Keep the CSS extension in the path, outside Vue/Nuxt's query metadata. */
export function createStyleVirtualId(id: string): string {
  if (isStyleSrcModuleId(id)) return id;
  id = normalizeInlineStyleModifiers(id);
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
  // A `<style src>` module resolves to its src file.
  if (isStyleSrcModuleId(id)) return resolveStyleSrcModuleId(id, root);
  const styleId = createStyleVirtualId(id);
  const queryStart = styleId.indexOf("?");
  const marker = styleId.slice(0, queryStart).lastIndexOf(STYLE_MARKER);
  if (marker < 0) return id;
  const params = new URLSearchParams(styleId.slice(queryStart + 1));
  const source = params.get("vize-file");
  if (source?.includes("?")) return id;
  if (!params.has("vize-root-relative") || !source?.startsWith("/")) return styleId;

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
  // `vize-file` names the owning SFC, for a generated style path and for a
  // `<style src>` module, whose path is the src file.
  const source = params.get("vize-file");
  if (source) {
    return `${source}?${query}`;
  }

  // Accept style IDs emitted by older versions of the plugin.
  return withoutPrefix.replace(/\.module\.\w+$/, "").replace(/\.\w+$/, "");
}
