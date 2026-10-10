import type { ArtFileInfo } from "../types/index.js";

export interface HostedGallery {
  arts: ArtFileInfo[];
  previewUrls: Record<string, Record<string, string>>;
}

/** Read a built gallery without requiring its build machine's source files. */
export async function loadHostedGallery(galleryUrl: string): Promise<HostedGallery> {
  const base = new URL(galleryUrl);
  if (!["http:", "https:"].includes(base.protocol) || base.username || base.password)
    throw new Error("--gallery-url must be an HTTP(S) URL without embedded credentials");
  base.pathname = `${base.pathname.replace(/\/+$/, "")}/`;
  base.search = "";
  base.hash = "";
  const manifestUrl = new URL("api/static.json", base);
  const response = await fetch(manifestUrl, {
    redirect: "error",
    signal: AbortSignal.timeout(30000),
  });
  if (!response.ok) throw new Error(`Hosted gallery manifest: HTTP ${response.status}`);
  const payload: unknown = await response.json();
  if (!isRecord(payload) || !Array.isArray(payload.arts) || !isRecord(payload.previews))
    throw new Error("Hosted gallery manifest must contain arts and previews");
  const arts: ArtFileInfo[] = [];
  const previewUrls: HostedGallery["previewUrls"] = Object.create(null);
  for (const item of payload.arts) {
    if (
      !isRecord(item) ||
      typeof item.path !== "string" ||
      !isRecord(item.metadata) ||
      typeof item.metadata.title !== "string" ||
      !Array.isArray(item.variants)
    )
      throw new Error("Invalid hosted art metadata");
    if (Object.hasOwn(previewUrls, item.path))
      throw new Error(`Duplicate hosted art: ${item.path}`);
    const variants = item.variants;
    const urls: Record<string, string> = Object.create(null);
    const sourceUrls = payload.previews[item.path];
    if (!isRecord(sourceUrls)) throw new Error(`Missing hosted previews: ${item.path}`);
    for (const variant of variants) {
      if (
        !isRecord(variant) ||
        typeof variant.name !== "string" ||
        typeof variant.skipVrt !== "boolean"
      )
        throw new Error("Invalid hosted variant metadata");
      if (Object.hasOwn(urls, variant.name))
        throw new Error(`Duplicate hosted variant: ${variant.name}`);
      const value = sourceUrls[variant.name];
      if (typeof value !== "string") throw new Error(`Missing hosted preview: ${variant.name}`);
      const url = new URL(value, base);
      if (
        url.origin !== base.origin ||
        !url.pathname.startsWith(base.pathname) ||
        url.username ||
        url.password
      )
        throw new Error(`Hosted preview is outside its gallery: ${variant.name}`);
      urls[variant.name] = url.href;
    }
    // The static emitter owns the complete ArtFileInfo shape. Above validates
    // the fields used by capture/approve/clean; unrelated source fields are inert.
    arts.push(item as unknown as ArtFileInfo);
    previewUrls[item.path] = urls;
  }
  return { arts, previewUrls };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}
