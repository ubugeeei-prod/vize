import type { VizeOptions } from "@vizejs/vite-plugin";
import { createHash } from "node:crypto";

function normalizeUrlPrefix(value: string): string {
  const withLeadingSlash = value.startsWith("/") ? value : `/${value}`;
  return withLeadingSlash.endsWith("/") ? withLeadingSlash : `${withLeadingSlash}/`;
}

export function buildNuxtDevAssetBase(baseURL = "/", buildAssetsDir = "/_nuxt/"): string {
  const normalizedBase = normalizeUrlPrefix(baseURL);
  const normalizedAssetsDir = normalizeUrlPrefix(buildAssetsDir);
  return normalizedBase === "/"
    ? normalizedAssetsDir
    : normalizeUrlPrefix(`${normalizedBase}${normalizedAssetsDir.replace(/^\//, "")}`);
}

export function buildNuxtCompilerOptions(
  rootDir: string,
  baseURL = "/",
  buildAssetsDir = "/_nuxt/",
): Pick<VizeOptions, "devUrlBase" | "root"> {
  return {
    devUrlBase: buildNuxtDevAssetBase(baseURL, buildAssetsDir),
    root: rootDir,
  };
}

export function isVizeVirtualVueModuleId(id: string): boolean {
  return id.startsWith("\0") && /\.vue\.ts(?:\?|$)/.test(id);
}

export function normalizeVizeVirtualVueModuleId(id: string): string {
  const withoutPrefix = id.startsWith("\0vize-ssr:") ? id.slice("\0vize-ssr:".length) : id.slice(1);
  return withoutPrefix.replace(/\.ts(?=\?|$)/, "");
}

const NUXT_INJECTED_MARKER = "/* nuxt-injected */";
const NUXT_INJECTED_KEY_RE = /'\$[^']+'\s+\/\* nuxt-injected \*\//g;

function buildStableNuxtKey(id: string, index: number): string {
  return createHash("sha256")
    .update(id)
    .update(":")
    .update(String(index))
    .digest("base64url")
    .slice(0, 10);
}

export function normalizeNuxtInjectedKeysForVizeVirtualModule(code: string, id: string): string {
  const normalizedId = normalizeVizeVirtualVueModuleId(id);
  let index = 0;
  return code.replace(NUXT_INJECTED_KEY_RE, () => {
    index += 1;
    return `'$${buildStableNuxtKey(normalizedId, index)}' ${NUXT_INJECTED_MARKER}`;
  });
}
