import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { vizeUnplugin } from "./unplugin.ts";
import { packageRoot } from "./test/helpers.ts";

// The #11481 report's component.
const SOURCE = `<script setup lang="ts">
defineProps<{ label: string }>();
</script>

<template>
  <button>{{ label }}</button>
</template>
`;

async function transform(options: { isProduction: boolean; ssr?: boolean }): Promise<string> {
  const meta = { framework: "rolldown" } as Parameters<typeof vizeUnplugin.raw>[1];
  const plugin = vizeUnplugin.raw(options, meta) as unknown as {
    transform: (this: unknown, code: string, id: string) => Promise<{ code: string } | null>;
  };
  const result = await plugin.transform.call(
    { warn() {}, addWatchFile() {} },
    SOURCE,
    path.join(packageRoot, "Foo.vue"),
  );
  assert.ok(result);
  return result.code;
}

void test("#11481 a production build inlines the render and drops dev prop types", async () => {
  const code = await transform({ isProduction: true });
  assert.match(code, /setup\(__props\) \{\s*return \(_ctx, _cache\) =>/);
  assert.match(code, /props: \{\s*label: \{\}\s*\}/);
  assert.match(code, /toDisplayString\(__props\.label\)/);
  assert.doesNotMatch(code, /_sfc_render|__expose|required: true|render:/);
});

void test("#11481 development and SSR builds keep a separate render", async () => {
  const dev = await transform({ isProduction: false });
  assert.match(dev, /function _sfc_render\(/);
  assert.match(dev, /render: _sfc_render/);
  assert.match(dev, /required: true/);

  // Like @vizejs/vite-plugin, a server build keeps the render separate but drops dev types.
  const ssr = await transform({ isProduction: true, ssr: true });
  assert.match(ssr, /ssrRender/);
  assert.doesNotMatch(ssr, /return \(_ctx, _cache\) =>/);
  assert.doesNotMatch(ssr, /required: true/);
});
