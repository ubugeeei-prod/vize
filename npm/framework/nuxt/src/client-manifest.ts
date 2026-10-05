/** Restore source identities before Nuxt removes page/global-component prefetches. */
export interface NuxtClientManifestEntry {
  src?: string;
  imports?: string[];
  dynamicImports?: string[];
}

export function restoreNuxtClientManifestSourceIds(
  manifest: Record<string, NuxtClientManifestEntry>,
): void {
  const identities = new Map<string, string>();
  for (const [key, entry] of Object.entries(manifest)) {
    // This exact query is emitted by toPluginVisibleVirtualId for a main SFC.
    // Extra queries, style/raw requests and legacy IDs have different owners.
    const query = key.indexOf("?");
    if (query < 0 || key.slice(query) !== "?vue&vize" || entry.src !== key) continue;
    const source = key.slice(0, query);
    if (!source.endsWith(".vue")) continue;
    if (Object.hasOwn(manifest, source)) {
      throw new Error(`@vizejs/nuxt: conflicting client manifest source ${source}`);
    }
    identities.set(key, source);
  }
  if (identities.size === 0) return;

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
}
