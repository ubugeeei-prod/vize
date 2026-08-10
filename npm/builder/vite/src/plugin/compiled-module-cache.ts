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
 * The keys are `path.resolve`d exactly as the previous scan resolved them, so a
 * dependency recorded as a relative path indexes and looks up identically.
 */

import path from "node:path";

import type { CompiledModule } from "../types.ts";

export class CompiledModuleCache extends Map<string, CompiledModule> {
  readonly #ownersByDependency = new Map<string, Set<string>>();
  readonly #dependenciesByOwner = new Map<string, Set<string>>();

  /**
   * Deliberately takes no entries: `Map`'s constructor would call the
   * overridden `set` from inside `super()`, before `#ownersByDependency` exists.
   */
  constructor() {
    super();
  }

  override set(file: string, compiled: CompiledModule): this {
    this.#unindex(file);
    super.set(file, compiled);
    const dependencies = new Set<string>();
    for (const dependency of compiled.dependencies ?? []) {
      const key = path.resolve(dependency);
      dependencies.add(key);
      const owners = this.#ownersByDependency.get(key);
      if (owners) {
        owners.add(file);
      } else {
        this.#ownersByDependency.set(key, new Set([file]));
      }
    }
    this.#dependenciesByOwner.set(file, dependencies);
    return this;
  }

  override delete(file: string): boolean {
    this.#unindex(file);
    return super.delete(file);
  }

  override clear(): void {
    this.#ownersByDependency.clear();
    this.#dependenciesByOwner.clear();
    super.clear();
  }

  /**
   * Evict a compiled value while retaining the dependency ownership that is
   * required to route another hot update before Vite reloads the owner module.
   * The next `set` reconciles that retained snapshot with the new compilation.
   */
  evict(file: string): boolean {
    return super.delete(file);
  }

  /** The cached SFCs that `src`-import `resolvedDependency`. */
  ownersOf(resolvedDependency: string): readonly string[] {
    const owners = this.#ownersByDependency.get(resolvedDependency);
    return owners ? [...owners] : [];
  }

  #unindex(file: string): void {
    for (const dependency of this.#dependenciesByOwner.get(file) ?? []) {
      const owners = this.#ownersByDependency.get(dependency);
      if (!owners) continue;
      owners.delete(file);
      if (owners.size === 0) this.#ownersByDependency.delete(dependency);
    }
    this.#dependenciesByOwner.delete(file);
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
  for (const [file, compiled] of cache) {
    if (
      compiled.dependencies?.some((dependency) => path.resolve(dependency) === resolvedDependency)
    )
      owners.push(file);
  }
  return owners;
}
