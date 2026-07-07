import { joinUrlPath } from "./static-data.js";

const TEXT_GALLERY_ASSET_EXT = /\.(?:css|html|js|map|mjs)$/u;

export function rewriteGalleryBase(source: string, basePath: string): string {
  return source.replaceAll("/__musea__/", `${basePath.replace(/\/?$/, "/")}`);
}

export function rewriteGalleryTextAssetBase(
  source: Uint8Array,
  relativePath: string,
  basePath: string,
): string | Uint8Array {
  if (!TEXT_GALLERY_ASSET_EXT.test(relativePath)) return source;
  return rewriteGalleryBase(Buffer.from(source).toString("utf-8"), basePath);
}

export function publicBasePathFromViteBase(viteBase: string | undefined, basePath: string): string {
  return viteBase && viteBase !== "/" && viteBase !== "./"
    ? joinUrlPath(viteBase, basePath)
    : basePath;
}
