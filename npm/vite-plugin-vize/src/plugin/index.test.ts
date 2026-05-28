import assert from "node:assert/strict";

import { createLegacyVueCompatibilityPlugin, isLegacyVueCompatibilityMode } from "./vue-version.ts";

{
  const plugin = createLegacyVueCompatibilityPlugin({ vueVersion: 2 });

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
}

console.log("✅ vite-plugin-vize index tests passed!");
