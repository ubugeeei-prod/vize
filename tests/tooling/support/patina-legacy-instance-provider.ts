import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

type Require = ReturnType<typeof createRequire>;
type CompilerSource = { packageName: string; path: string; sha256: string };

export function legacyInstanceVueRequire(): Require {
  const tests = createRequire(new URL("../../package.json", import.meta.url));
  return createRequire(
    process.env.VIZE_LEGACY_API_VUE_MANIFEST ??
      tests.resolve("vue-legacy-instance-api-oracle/package.json"),
  );
}

export function assertLegacyInstanceCompilerSources(
  vue: Require,
  compiler: "@vue/compiler-dom" | "@vue/compiler-sfc",
  sources: readonly CompilerSource[],
): void {
  const owner = createRequire(vue.resolve(`${compiler}/package.json`));
  const manifest = owner("./package.json");
  assert.equal(manifest.version, "3.6.0-rc.9", "the pinned compiler must own its dependencies");
  assert.equal(manifest.dependencies["@vue/compiler-core"], "3.6.0-rc.9");
  assert.deepEqual(
    sources.map((source) => source.packageName),
    [
      ...(compiler === "@vue/compiler-sfc" ? [compiler] : []),
      "@vue/compiler-dom",
      "@vue/compiler-core",
    ],
  );
  owner(compiler);
  for (const source of sources) {
    const provider = owner(`${source.packageName}/package.json`);
    assert.equal(provider.version, "3.6.0-rc.9");
    assert.equal(source.path, `dist/${source.packageName.slice("@vue/".length)}.cjs.js`);
    const file = path.join(
      path.dirname(owner.resolve(`${source.packageName}/package.json`)),
      source.path,
    );
    assert.equal(
      owner.cache[file]?.loaded,
      true,
      `actual compiler dependency must be loaded: ${file}`,
    );
    const actual = createHash("sha256").update(readFileSync(file)).digest("hex");
    if (source.packageName === "@vue/compiler-core") {
      // Frozen receipts hashed an unrelated hoisted 3.5.35 file. Retain that
      // historical error, but authenticate only the actual rc.9 dependency.
      assert.equal(
        source.sha256,
        "820601aa19727baa3de3b6eea199d6dfb547911afdb7b22d8081d3c81eff284e",
      );
      assert.equal(actual, "61aa2191bdde10d4e7e9f8224f47913350de3941cf54b2c2430ccdb8f7d1c08f");
    } else {
      assert.equal(actual, source.sha256);
    }
  }
}
