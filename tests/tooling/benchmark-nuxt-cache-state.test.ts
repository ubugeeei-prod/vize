import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  NUXT_SHARED_CACHE_DIRS,
  assertNuxtBuildCachesCold,
  clearNuxtBuildCaches,
  linkColdNodeModules,
  readNuxtBuildCaches,
  resolveNodeModulesDir,
} from "../../tools/benchmarks/scripts/nuxt-build-cache.mjs";

/**
 * Mirrors the benchmark's own layout: several app directories whose
 * `node_modules` are symlinks to one shared installed tree.
 */
function withSharedNodeModules(
  fn: (context: { shared: string; apps: string[] }) => void,
  appCount = 2,
): void {
  const base = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bench-nuxt-cache-"));
  try {
    const shared = path.join(base, "shared-node-modules");
    fs.mkdirSync(path.join(shared, "nuxt"), { recursive: true });
    fs.writeFileSync(path.join(shared, "nuxt", "package.json"), "{}");

    const apps: string[] = [];
    for (let i = 0; i < appCount; i++) {
      const app = path.join(base, `app-${i}`);
      fs.mkdirSync(app, { recursive: true });
      fs.symlinkSync(shared, path.join(app, "node_modules"), "dir");
      apps.push(app);
    }
    fn({ shared, apps });
  } finally {
    fs.rmSync(base, { recursive: true, force: true });
  }
}

function seedCaches(shared: string, names: string[]): void {
  for (const name of names) {
    fs.mkdirSync(path.join(shared, name, "nested"), { recursive: true });
    fs.writeFileSync(path.join(shared, name, "nested", "entry"), "cached");
  }
}

test("the shared cache list names every build cache under node_modules", () => {
  assert.deepEqual(NUXT_SHARED_CACHE_DIRS, [".vize", ".cache", ".vite"]);
});

test("node_modules is reported at its real path, not the symlink", () => {
  withSharedNodeModules(({ shared, apps }) => {
    assert.deepEqual(
      apps.map((app) => resolveNodeModulesDir(app)),
      apps.map(() => fs.realpathSync(shared)),
    );
  });
});

test("an app without node_modules has nothing to read or clear", () => {
  const app = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bench-nuxt-bare-"));
  try {
    assert.equal(resolveNodeModulesDir(app), null);
    assert.deepEqual(readNuxtBuildCaches(app), []);
    assert.deepEqual(clearNuxtBuildCaches(app), []);
    assert.equal(assertNuxtBuildCachesCold(app, "@vizejs/nuxt"), undefined);
  } finally {
    fs.rmSync(app, { recursive: true, force: true });
  }
});

// The defect: the benchmark's app directories are fresh every run but share one
// `node_modules`, so a cache one run wrote is visible to the next run and to the
// other variant. Only `@vizejs/nuxt` can use `.vize/vite-precompile`, so leaving
// it warm gives that variant a restore the Nuxt default compiler cannot get.
test("a cache written through one app is visible through every other app", () => {
  withSharedNodeModules(({ shared, apps }) => {
    seedCaches(shared, [".vize", ".cache"]);

    assert.deepEqual(
      apps.map((app) => readNuxtBuildCaches(app)),
      apps.map(() => [".cache", ".vize"]),
    );
  });
});

test("clearing through one app clears it for all of them and keeps the packages", () => {
  withSharedNodeModules(({ shared, apps }) => {
    seedCaches(shared, [".vize", ".cache", ".vite"]);

    assert.deepEqual(clearNuxtBuildCaches(apps[0]), [".cache", ".vite", ".vize"]);
    assert.deepEqual(
      apps.map((app) => readNuxtBuildCaches(app)),
      apps.map(() => []),
    );
    assert.deepEqual(fs.readdirSync(shared), ["nuxt"]);
    assert.deepEqual(fs.readdirSync(path.join(shared, "nuxt")), ["package.json"]);
  });
});

// Linking and clearing are one operation: the symlink is what makes the caches
// shared, so a caller that creates one must not be able to forget the clear.
test("linking a shared node_modules leaves it cold, picking the first existing candidate", () => {
  withSharedNodeModules(({ shared }) => {
    seedCaches(shared, [".vize", ".cache", ".vite"]);
    const app = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bench-nuxt-link-"));
    try {
      linkColdNodeModules(app, [path.join(shared, "missing"), shared], "@vizejs/nuxt");

      assert.equal(resolveNodeModulesDir(app), fs.realpathSync(shared));
      assert.deepEqual(readNuxtBuildCaches(app), []);
      assert.deepEqual(fs.readdirSync(shared), ["nuxt"]);
    } finally {
      fs.rmSync(app, { recursive: true, force: true });
    }
  });
});

test("measuring against a warm shared cache is refused, naming what is left", () => {
  withSharedNodeModules(({ shared, apps }) => {
    seedCaches(shared, [".vize", ".vite"]);

    assert.throws(
      () => assertNuxtBuildCachesCold(apps[1], "@vizejs/nuxt"),
      (error: unknown) => {
        assert.ok(error instanceof Error);
        assert.equal(
          error.message,
          "Refusing to measure @vizejs/nuxt against warm build caches: " +
            `${fs.realpathSync(shared)} still holds .vite, .vize. ` +
            "That directory is shared by every measured run and by both variants, " +
            "so this would not be the same measurement.",
        );
        return true;
      },
    );

    clearNuxtBuildCaches(apps[1]);
    assert.equal(assertNuxtBuildCachesCold(apps[0], "Nuxt default compiler"), undefined);
  });
});
