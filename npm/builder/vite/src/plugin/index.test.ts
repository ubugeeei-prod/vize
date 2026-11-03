import assert from "node:assert/strict";

import {
  createLegacyVueCompatibilityPlugin,
  hasHostVueSfcCompilerPlugin,
  isLegacyVueCompatibilityMode,
  removeHostVueSfcCompilers,
} from "./vue-version.ts";

{
  const plugin = createLegacyVueCompatibilityPlugin({ vueVersion: 2 });

  assert.equal(
    isLegacyVueCompatibilityMode({ vueVersion: 0.1 }),
    true,
    "vueVersion: 0.10 should enable legacy Vue compatibility mode",
  );

  assert.equal(
    isLegacyVueCompatibilityMode({ vueVersion: 0.11 }),
    true,
    "vueVersion: 0.11 should enable legacy Vue compatibility mode",
  );
  assert.equal(
    isLegacyVueCompatibilityMode({ vueVersion: 1 }),
    true,
    "vueVersion: 1 should enable legacy Vue compatibility mode",
  );
  assert.equal(
    isLegacyVueCompatibilityMode({ vueVersion: 2 }),
    true,
    "vueVersion: 2 should enable legacy Vue compatibility mode",
  );
  assert.equal(
    isLegacyVueCompatibilityMode({ vueVersion: "2.7" }),
    true,
    "vueVersion: 2.7 should enable legacy Vue compatibility mode",
  );
  assert.equal(
    isLegacyVueCompatibilityMode({ vueVersion: "legacy" }),
    true,
    "vueVersion: legacy should enable legacy Vue compatibility mode",
  );
  assert.equal(
    isLegacyVueCompatibilityMode({ vueVersion: 3 }),
    false,
    "vueVersion: 3 should keep Vize's Vue 3 compiler pipeline enabled",
  );
  assert.equal(
    isLegacyVueCompatibilityMode({ compatibility: { vueVersion: 2 } }),
    true,
    "compatibility.vueVersion should enable legacy Vue compatibility mode",
  );
  assert.equal(
    isLegacyVueCompatibilityMode({ vueVersion: 2, compatibility: { hostCompiler: false } }),
    false,
    "hostCompiler: false should let advanced legacy users force the Vize compiler pipeline",
  );
  assert.equal(
    plugin.name,
    "vite-plugin-vize:legacy-vue-compat",
    "Legacy Vue compatibility mode should expose a non-invasive marker plugin",
  );
  assert.equal(
    "resolveId" in plugin,
    false,
    "Legacy Vue compatibility mode must not resolve .vue IDs",
  );
  assert.equal("load" in plugin, false, "Legacy Vue compatibility mode must not load .vue modules");
  assert.equal(
    "transform" in plugin,
    false,
    "Legacy Vue compatibility mode must not transform .vue code",
  );
  assert.equal(hasHostVueSfcCompilerPlugin([{ name: "vite:vue2" }]), true);
  assert.equal(
    hasHostVueSfcCompilerPlugin([{ name: "vite-plugin-vize:legacy-vue-compat" }]),
    false,
  );
  assert.throws(
    () =>
      plugin.configResolved?.({
        root: "/repo",
        plugins: [{ name: "vite-plugin-vize:legacy-vue-compat" }],
      } as never),
    /host Vue SFC compiler plugin/,
  );
}

{
  // `@void/vue`'s `voidVue()` bundles @vitejs/plugin-vue. Both compiling the
  // same SFC made plugin-vue read Vize's style virtual IDs from disk (ENOENT).
  const hostVue = { name: "vite:vue", transform() {} };
  const vizeShim = { name: "vite:vue" };
  const pages = { name: "void-vue:pages", transform() {} };
  const plugins = [hostVue, vizeShim, pages];

  assert.equal(removeHostVueSfcCompilers([...plugins], { exclude: /legacy/ }), 0);
  assert.equal(removeHostVueSfcCompilers(plugins, {}), 1);
  assert.deepEqual(plugins, [vizeShim, pages], "only the compiling plugin-vue is removed");
}

{
  // #11641: Vize compiles where the host compiler did, after the host's
  // transforms of the SFC text and before those of the compiled module.
  const hostVue = { name: "vite:vue", transform() {} };
  const sourceTransform = { name: "host:sfc-source", transform() {} };
  const compiledTransform = { name: "host:compiled", transform() {} };
  const stage = { name: "vite-plugin-vize:sfc", transform() {} };
  const replaced = [sourceTransform, hostVue, compiledTransform, stage];
  assert.equal(removeHostVueSfcCompilers(replaced, {}, stage), 1);
  assert.deepEqual(replaced, [sourceTransform, stage, compiledTransform]);

  const kept = [stage, sourceTransform, hostVue, compiledTransform];
  assert.equal(removeHostVueSfcCompilers(kept, { exclude: /legacy/ }, stage), 0);
  assert.deepEqual(kept, [sourceTransform, stage, hostVue, compiledTransform]);

  // Without a host compiler the stage keeps the place the config gave it.
  const alone = [sourceTransform, stage, compiledTransform];
  assert.equal(removeHostVueSfcCompilers(alone, {}, stage), 0);
  assert.deepEqual(alone, [sourceTransform, stage, compiledTransform]);
}

console.log("✅ vite-plugin-vize index tests passed!");
