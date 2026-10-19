import type { Plugin } from "vite";

/** The query `toPluginVisibleVirtualId` gives a main SFC module. */
const MAIN_SFC_QUERY = "?vue&vize";

type ManifestEntry = { src?: string; imports?: string[]; dynamicImports?: string[] };
type BundleAsset = { type: string; source?: string | Uint8Array };

function sourceIdentity(key: string): string | null {
  const query = key.indexOf("?");
  if (query < 0 || key.slice(query) !== MAIN_SFC_QUERY) return null;
  const source = key.slice(0, query);
  return source.endsWith(".vue") ? source : null;
}

/**
 * Key a Vite build manifest by SFC source path, as `@vitejs/plugin-vue` does.
 *
 * Vize's main SFC modules carry `?vue&vize`, so Vite keys their chunks
 * `pages/index.vue?vue&vize`. Servers that look up a route's CSS and preloads
 * by its source file (Void, Vite SSR setups) would miss them.
 */
export function restoreManifestSourceIds(manifest: Record<string, ManifestEntry>): boolean {
  const identities = new Map<string, string>();
  for (const [key, entry] of Object.entries(manifest)) {
    const source = sourceIdentity(key);
    if (!source || entry.src !== key || Object.hasOwn(manifest, source)) continue;
    identities.set(key, source);
  }
  if (identities.size === 0) return false;

  for (const [key, entry] of Object.entries(manifest)) {
    const source = identities.get(key);
    if (source) {
      delete manifest[key];
      manifest[source] = entry;
      entry.src = source;
    }
    for (const field of ["imports", "dynamicImports"] as const) {
      const references = entry[field];
      if (references?.some((reference) => identities.has(reference))) {
        entry[field] = references.map((reference) => identities.get(reference) ?? reference);
      }
    }
  }
  return true;
}

/** Same for `build.ssrManifest`, whose keys are module IDs. */
export function restoreSsrManifestSourceIds(manifest: Record<string, string[]>): boolean {
  let changed = false;
  for (const key of Object.keys(manifest)) {
    const source = sourceIdentity(key);
    if (!source || Object.hasOwn(manifest, source)) continue;
    manifest[source] = manifest[key];
    delete manifest[key];
    changed = true;
  }
  return changed;
}

function manifestFileName(option: unknown, fallback: string): string | null {
  if (!option) return null;
  return typeof option === "string" ? option : fallback;
}

function rewriteAsset(
  bundle: Record<string, BundleAsset>,
  fileName: string | null,
  restore: (manifest: never) => boolean,
): void {
  const asset = fileName ? bundle[fileName] : undefined;
  if (asset?.type !== "asset" || asset.source === undefined) return;
  const text =
    typeof asset.source === "string" ? asset.source : new TextDecoder().decode(asset.source);
  const manifest = JSON.parse(text) as never;
  if (restore(manifest)) asset.source = JSON.stringify(manifest, null, 2);
}

export function createManifestPlugin(): Plugin {
  return {
    name: "vite-plugin-vize:manifest",
    apply: "build",
    generateBundle: {
      order: "post",
      handler(_, bundle) {
        const build = this.environment?.config.build;
        if (!build) return;
        const assets = bundle as unknown as Record<string, BundleAsset>;
        rewriteAsset(
          assets,
          manifestFileName(build.manifest, ".vite/manifest.json"),
          restoreManifestSourceIds,
        );
        rewriteAsset(
          assets,
          manifestFileName(build.ssrManifest, ".vite/ssr-manifest.json"),
          restoreSsrManifestSourceIds,
        );
      },
    },
  };
}
