import assert from "node:assert/strict";
import path from "node:path";
import { FilesystemEpoch } from "./filesystem-epoch.ts";

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
  const requested = path.resolve(resolvedDependency);
  for (const [file, module] of cache) {
    if (module.dependencies?.some((dependency) => path.resolve(dependency) === requested))
      owners.push(file);
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
  // Evicted files await a recompile until something compiles or removes them.
  const cache = new CompiledModuleCache();
  for (const file of ["/src/A.vue", "/src/B.vue", "/src/C.vue", "/src/D.vue"]) {
    cache.set(file, compiled(["/src/types.ts"]));
  }
  assert.deepEqual([...cache.evictedFiles()], []);

  for (const file of ["/src/B.vue", "/src/A.vue", "/src/C.vue"]) cache.evict(file);
  assert.equal(cache.evict("/src/Missing.vue"), false);
  assert.equal(cache.evict("/src/A.vue"), false, "a second eviction finds nothing to evict");
  assert.deepEqual([...cache.evictedFiles()], ["/src/B.vue", "/src/A.vue", "/src/C.vue"]);

  cache.set("/src/A.vue", compiled([]));
  cache.delete("/src/C.vue");
  assert.deepEqual([...cache.evictedFiles()], ["/src/B.vue"]);

  cache.clear();
  assert.deepEqual([...cache.evictedFiles()], []);
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
  // tinyglobby uses forward slashes on Windows while Vite may hand transform
  // hooks backslashes for the same file. Both must address one cache entry.
  const cache = new CompiledModuleCache();
  const forward = "C:/project/src/App.vue";
  const backward = String.raw`C:\project\src\App.vue`;
  const module = compiled([]);
  cache.set(forward, module);

  assert.equal(cache.get(backward), module);
  assert.equal(cache.has(backward), true);
  assert.equal(cache.delete(backward), true);
  assert.equal(cache.has(forward), false);
}

{
  // Relative dependencies and queries share the same resolved identity.
  const cache = new CompiledModuleCache();
  cache.set("/src/App.vue", compiled(["./relative.css"]));

  assert.deepEqual(cache.ownersOf(path.resolve("./relative.css")), ["/src/App.vue"]);
  assert.deepEqual(cache.ownersOf("./relative.css"), ["/src/App.vue"]);
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

{
  // Large lists keep HMR ownership across eviction, then replace and remove it.
  const epoch = new FilesystemEpoch();
  const cache = new CompiledModuleCache(epoch);
  const inputs = Array.from({ length: 32 }, (_, index) => `/src/shared-${index}.ts`);
  cache.set("/src/First.vue", compiled(inputs));
  cache.set("/src/Second.vue", compiled(inputs.slice()));
  cache.evict("/src/First.vue");
  assert.deepEqual(cache.ownersOf(inputs[0]), ["/src/First.vue", "/src/Second.vue"]);
  epoch.advance();
  cache.set("/src/Second.vue", compiled(["/src/changed.ts"]));
  assert.deepEqual(cache.ownersOf(inputs[0]), ["/src/First.vue"]);
  cache.set("/src/First.vue", compiled(inputs.slice()));
  cache.evict("/src/First.vue");
  cache.delete("/src/First.vue");
  assert.deepEqual(cache.ownersOf(inputs[0]), []);
  assert.deepEqual(cache.ownersOf("/src/changed.ts"), ["/src/Second.vue"]);
  cache.clear();
  assert.deepEqual(cache.ownersOf("/src/changed.ts"), []);
}
