/**
 * Compiled-module caches carrying a reverse index from a `src`-imported
 * dependency to the SFCs that pulled it in.
 *
 * Every hot update has to answer "which SFCs own this changed file?" before it
 * can do anything else, and that answer used to come from a full scan of both
 * caches with a `path.resolve` per dependency per cached file. HMR latency then
 * grew with the number of components in the project, on the interactive path,
 * for every save — including saves of ordinary `.vue` files, because the scan
 * runs before the `.vue` fast path.
 *
 * The index is maintained by overriding `set`/`delete`/`clear` rather than by a
 * second structure updated at each call site. `state.cache` is handed to
 * `compileFile` and `compileBatch`, and is also mutated directly from
 * `precompile-run.ts`, `hmr.ts` and `state.ts`; a structure kept in sync by hand
 * across those would drift the first time a new writer appeared. Subclassing
 * puts the bookkeeping where the mutation is.
 *
 * Each key keeps both its resolved logical path and physical real path. This
 * preserves constant-time lookup when a workspace link is reported from the
 * opposite side by Vite or its watcher.
 */

import type { CompiledModule } from "../types.ts";
import { normalizeCachePath } from "./cache-path.ts";
import { filesystemPathIdentities } from "./path-identity.ts";

export class CompiledModuleCache extends Map<string, CompiledModule> {
  readonly #ownersByDependency = new Map<string, Set<string>>();
  readonly #dependenciesByOwner = new Map<string, Set<string>>();
  readonly #ownersByFileIdentity = new Map<string, Set<string>>();
  readonly #fileIdentitiesByOwner = new Map<string, Set<string>>();

  /**
   * Deliberately takes no entries: `Map`'s constructor would call the
   * overridden `set` from inside `super()`, before `#ownersByDependency` exists.
   */
  constructor() {
    super();
  }

  override get(file: string): CompiledModule | undefined {
    return super.get(normalizeCachePath(file));
  }

  override has(file: string): boolean {
    return super.has(normalizeCachePath(file));
  }

  override set(file: string, compiled: CompiledModule): this {
    file = normalizeCachePath(file);
    this.#unindex(file);
    super.set(file, compiled);
    const dependencies = new Set<string>();
    for (const dependency of compiled.dependencies ?? []) {
      for (const key of filesystemPathIdentities(dependency)) {
        dependencies.add(key);
        const owners = this.#ownersByDependency.get(key);
        if (owners) {
          owners.add(file);
        } else {
          this.#ownersByDependency.set(key, new Set([file]));
        }
      }
    }
    this.#dependenciesByOwner.set(file, dependencies);
    const fileIdentities = new Set(filesystemPathIdentities(file));
    for (const identity of fileIdentities) {
      const owners = this.#ownersByFileIdentity.get(identity);
      if (owners) owners.add(file);
      else this.#ownersByFileIdentity.set(identity, new Set([file]));
    }
    this.#fileIdentitiesByOwner.set(file, fileIdentities);
    return this;
  }

  override delete(file: string): boolean {
    file = normalizeCachePath(file);
    this.#unindex(file);
    return super.delete(file);
  }

  override clear(): void {
    this.#ownersByDependency.clear();
    this.#dependenciesByOwner.clear();
    this.#ownersByFileIdentity.clear();
    this.#fileIdentitiesByOwner.clear();
    super.clear();
  }

  /**
   * Evict a compiled value while retaining the dependency ownership that is
   * required to route another hot update before Vite reloads the owner module.
   * The next `set` reconciles that retained snapshot with the new compilation.
   */
  evict(file: string): boolean {
    return super.delete(normalizeCachePath(file));
  }

  /** The cached SFCs that `src`-import `resolvedDependency`. */
  ownersOf(resolvedDependency: string): readonly string[] {
    const owners = new Set<string>();
    for (const identity of filesystemPathIdentities(resolvedDependency)) {
      for (const owner of this.#ownersByDependency.get(identity) ?? []) owners.add(owner);
    }
    return [...owners];
  }

  /** Cached logical owner paths that name the same physical SFC as `file`. */
  ownersOfFile(file: string): readonly string[] {
    const owners = new Set<string>();
    for (const identity of filesystemPathIdentities(file)) {
      for (const owner of this.#ownersByFileIdentity.get(identity) ?? []) owners.add(owner);
    }
    return [...owners];
  }

  #unindex(file: string): void {
    file = normalizeCachePath(file);
    for (const dependency of this.#dependenciesByOwner.get(file) ?? []) {
      const owners = this.#ownersByDependency.get(dependency);
      if (!owners) continue;
      owners.delete(file);
      if (owners.size === 0) this.#ownersByDependency.delete(dependency);
    }
    this.#dependenciesByOwner.delete(file);
    for (const identity of this.#fileIdentitiesByOwner.get(file) ?? []) {
      const owners = this.#ownersByFileIdentity.get(identity);
      if (!owners) continue;
      owners.delete(file);
      if (owners.size === 0) this.#ownersByFileIdentity.delete(identity);
    }
    this.#fileIdentitiesByOwner.delete(file);
  }
}

/**
 * Invalidate a compiled module without dropping its HMR dependency routing.
 * Hand-built plain Maps retain their historical delete behaviour.
 */
export function evictCompiledModule(cache: Map<string, CompiledModule>, file: string): boolean {
  return cache instanceof CompiledModuleCache ? cache.evict(file) : cache.delete(file);
}

/**
 * Owners of `resolvedDependency` in one cache.
 *
 * Plain `Map`s — the hand-built states in unit tests — keep the original linear
 * scan, so an unindexed cache answers exactly what the indexed one does.
 */
export function ownersOfDependency(
  cache: Map<string, CompiledModule>,
  resolvedDependency: string,
): readonly string[] {
  if (cache instanceof CompiledModuleCache) {
    return cache.ownersOf(resolvedDependency);
  }

  const owners: string[] = [];
  const requested = new Set(filesystemPathIdentities(resolvedDependency));
  for (const [file, compiled] of cache) {
    if (
      compiled.dependencies?.some((dependency) =>
        filesystemPathIdentities(dependency).some((identity) => requested.has(identity)),
      )
    )
      owners.push(file);
  }
  return owners;
}

/** Cached owner paths that are filesystem aliases of `file`. */
export function ownersOfFile(cache: Map<string, CompiledModule>, file: string): readonly string[] {
  if (cache instanceof CompiledModuleCache) return cache.ownersOfFile(file);
  const requested = new Set(filesystemPathIdentities(file));
  return [...cache.keys()].filter((owner) =>
    filesystemPathIdentities(owner).some((identity) => requested.has(identity)),
  );
}
