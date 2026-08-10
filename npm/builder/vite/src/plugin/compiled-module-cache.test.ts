import assert from "node:assert/strict";
import path from "node:path";

import type { CompiledModule } from "../types.ts";
import { CompiledModuleCache, ownersOfDependency } from "./compiled-module-cache.ts";

function compiled(dependencies: string[]): CompiledModule {
  return { code: "export default {}", dependencies } as unknown as CompiledModule;
}

/** The scan `getVueFilesDependingOn` performed before the reverse index existed. */
function scanOwners(
  cache: Map<string, CompiledModule>,
  resolvedDependency: string,
): readonly string[] {
  const owners: string[] = [];
  for (const [file, module] of cache) {
    if (module.dependencies?.some((d) => path.resolve(d) === resolvedDependency)) owners.push(file);
  }
  return owners;
}

{
  const cache = new CompiledModuleCache();
  cache.set("/src/App.vue", compiled(["/src/shared.css"]));
  cache.set("/src/Page.vue", compiled(["/src/shared.css", "/src/page.ts"]));
  cache.set("/src/Plain.vue", compiled([]));

  assert.deepEqual(cache.ownersOf("/src/shared.css"), ["/src/App.vue", "/src/Page.vue"]);
  assert.deepEqual(cache.ownersOf("/src/page.ts"), ["/src/Page.vue"]);
  assert.deepEqual(cache.ownersOf("/src/missing.css"), []);
}

{
  // Re-compiling a file must not leave its previous dependencies indexed.
  const cache = new CompiledModuleCache();
  cache.set("/src/App.vue", compiled(["/src/old.css"]));
  cache.set("/src/App.vue", compiled(["/src/new.css"]));

  assert.deepEqual(cache.ownersOf("/src/old.css"), []);
  assert.deepEqual(cache.ownersOf("/src/new.css"), ["/src/App.vue"]);
}

{
  // A dependency save can arrive again before Vite reloads the owner module.
  // Eviction must discard stale compiled code without losing that routing.
  const cache = new CompiledModuleCache();
  cache.set("/src/App.vue", compiled(["/src/old.css"]));

  assert.equal(cache.evict("/src/App.vue"), true);
  assert.equal(cache.has("/src/App.vue"), false);
  assert.deepEqual(cache.ownersOf("/src/old.css"), ["/src/App.vue"]);

  cache.set("/src/App.vue", compiled(["/src/new.css"]));
  assert.deepEqual(cache.ownersOf("/src/old.css"), []);
  assert.deepEqual(cache.ownersOf("/src/new.css"), ["/src/App.vue"]);

  assert.equal(cache.delete("/src/App.vue"), true);
  assert.deepEqual(cache.ownersOf("/src/new.css"), []);

  cache.set("/src/App.vue", compiled(["/src/final.css"]));
  assert.equal(cache.evict("/src/App.vue"), true);
  assert.equal(cache.delete("/src/App.vue"), false);
  assert.deepEqual(
    cache.ownersOf("/src/final.css"),
    [],
    "an explicit delete after eviction must remove retained HMR ownership",
  );
}

{
  const cache = new CompiledModuleCache();
  cache.set("/src/App.vue", compiled(["/src/shared.css"]));
  cache.set("/src/Page.vue", compiled(["/src/shared.css"]));

  assert.equal(cache.delete("/src/App.vue"), true);
  assert.deepEqual(cache.ownersOf("/src/shared.css"), ["/src/Page.vue"]);

  cache.clear();
  assert.deepEqual(cache.ownersOf("/src/shared.css"), []);
  assert.equal(cache.size, 0);
}

{
  // Relative dependencies are indexed through the same `path.resolve` the scan
  // used, so they answer the same question.
  const cache = new CompiledModuleCache();
  cache.set("/src/App.vue", compiled(["./relative.css"]));

  assert.deepEqual(cache.ownersOf(path.resolve("./relative.css")), ["/src/App.vue"]);
  assert.deepEqual(cache.ownersOf("./relative.css"), []);
}

{
  // The indexed cache and a plain Map must answer identically for every
  // dependency in a mixed corpus.
  const entries: [string, CompiledModule][] = [
    ["/src/A.vue", compiled(["/src/a.css"])],
    ["/src/B.vue", compiled(["/src/a.css", "/src/b.ts"])],
    ["/src/C.vue", compiled([])],
    ["/src/D.vue", compiled(["./rel.css"])],
    ["/src/E.vue", compiled(["/src/b.ts"])],
  ];
  const indexed = new CompiledModuleCache();
  const plain = new Map<string, CompiledModule>();
  for (const [file, module] of entries) {
    indexed.set(file, module);
    plain.set(file, module);
  }

  for (const dependency of [
    "/src/a.css",
    "/src/b.ts",
    path.resolve("./rel.css"),
    "/src/absent.css",
  ]) {
    assert.deepEqual(
      [...ownersOfDependency(indexed, dependency)].sort(),
      [...scanOwners(plain, dependency)].sort(),
      `indexed and scanned owners must agree for ${dependency}`,
    );
    assert.deepEqual(
      [...ownersOfDependency(plain, dependency)].sort(),
      [...scanOwners(plain, dependency)].sort(),
      `unindexed fallback must match the original scan for ${dependency}`,
    );
  }
}

console.log("compiled-module-cache tests passed");
