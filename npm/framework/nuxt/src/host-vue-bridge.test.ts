import assert from "node:assert/strict";
import test from "node:test";

import {
  patchNuxtHostVuePluginForCompilerExcludes,
  placeVizeCompileStage,
} from "./host-vue-bridge.ts";
import { NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE } from "./utils.ts";

void test("Nuxt host Vue bridge delegates only compiler-excluded SFCs", () => {
  const calls: string[] = [];
  const plugin = {
    name: "vite:vue",
    resolveId(id: string, importer?: string) {
      calls.push(`resolve:${id}:${importer ?? ""}`);
      return `resolved:${id}`;
    },
    load(id: string) {
      calls.push(`load:${id}`);
      return `loaded:${id}`;
    },
    transform(code: string, id: string) {
      calls.push(`transform:${id}`);
      return { code: `/* host */\n${code}`, map: null };
    },
    handleHotUpdate(context: { file: string }) {
      calls.push(`hot:${context.file}`);
      return ["module"];
    },
  };

  assert.equal(
    patchNuxtHostVuePluginForCompilerExcludes(plugin, {
      exclude: NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
    }),
    true,
  );

  assert.equal(plugin.transform("export default {}", "/repo/app/components/Page.vue"), null);
  assert.equal(
    plugin.transform("export default {}", "/repo/app/components/Page.vue?vue&type=script"),
    null,
  );
  assert.equal(plugin.load("\0plugin-vue:export-helper"), null);
  assert.deepEqual(calls, []);

  assert.deepEqual(plugin.transform("export default {}", "/repo/app/OgImage/Page.takumi.vue"), {
    code: "/* host */\nexport default {}",
    map: null,
  });
  assert.deepEqual(
    plugin.transform("export default {}", "/repo/app/components/OgLayout.vue?og-image-depth=1"),
    {
      code: "/* host */\nexport default {}",
      map: null,
    },
  );
  assert.deepEqual(
    plugin.transform(
      '<script setup lang="ts"></script>',
      "/repo/app/components/OgLayout.vue?vue&type=script&setup=true&lang.ts",
    ),
    {
      code: '/* host */\n<script setup lang="ts"></script>',
      map: null,
    },
  );
  assert.equal(plugin.load("\0plugin-vue:export-helper"), "loaded:\0plugin-vue:export-helper");
  assert.equal(
    plugin.transform(
      "export default {}",
      "/repo/app/components/Other.vue?vue&type=script&setup=true&lang.ts",
    ),
    null,
  );
  assert.equal(
    plugin.load("/repo/app/OgImage/Page.takumi.vue?vue&type=template"),
    "loaded:/repo/app/OgImage/Page.takumi.vue?vue&type=template",
  );
  assert.equal(
    plugin.resolveId("vue", "/repo/app/OgImage/Page.takumi.vue?vue&type=script"),
    "resolved:vue",
  );
  assert.equal(plugin.handleHotUpdate({ file: "/repo/app/components/Page.vue" }), undefined);
  assert.deepEqual(plugin.handleHotUpdate({ file: "/repo/app/OgImage/Page.takumi.vue" }), [
    "module",
  ]);
  assert.deepEqual(calls, [
    "transform:/repo/app/OgImage/Page.takumi.vue",
    "transform:/repo/app/components/OgLayout.vue?og-image-depth=1",
    "transform:/repo/app/components/OgLayout.vue?vue&type=script&setup=true&lang.ts",
    "load:\0plugin-vue:export-helper",
    "load:/repo/app/OgImage/Page.takumi.vue?vue&type=template",
    "resolve:vue:/repo/app/OgImage/Page.takumi.vue?vue&type=script",
    "hot:/repo/app/OgImage/Page.takumi.vue",
  ]);
});

void test("Nuxt host Vue bridge supports hook objects and query-bearing glob excludes", () => {
  let calls = 0;
  const plugin = {
    name: "vite:vue",
    transform: {
      handler(code: string, id: string) {
        calls++;
        return `compiled:${id}:${code}`;
      },
    },
  };

  assert.equal(
    patchNuxtHostVuePluginForCompilerExcludes(plugin, {
      exclude: "**/*.takumi.vue",
    }),
    true,
  );

  assert.equal(plugin.transform.handler("code", "/repo/app/OgImage/Page.vue"), null);
  assert.equal(calls, 0);
  assert.equal(
    plugin.transform.handler("code", "/@fs/repo/app/OgImage/Page.takumi.vue?vue&type=script"),
    "compiled:/@fs/repo/app/OgImage/Page.takumi.vue?vue&type=script:code",
  );
  assert.equal(calls, 1);
});

void test("Nuxt host Vue bridge leaves Vize's compatibility shim removable", () => {
  assert.equal(
    patchNuxtHostVuePluginForCompilerExcludes(
      { name: "vite:vue" },
      { exclude: /\.takumi\.vue(?:\?|$)/ },
    ),
    false,
  );
});

void test("#11641 Vize compiles at the host compiler's place in every Nuxt config", () => {
  // Nuxt 4.6.4 lists `vite:vue` first and `nuxt:ssr-styles` (which parses each
  // module as JavaScript) before the modules' plugins, where Vize registers.
  const stage = { name: "vite-plugin-vize:sfc", transform() {} };
  const vizePlugins = [{ name: "vite:vue" }, { name: "vite-plugin-vize" }, stage];
  const before = [...vizePlugins];
  const hostVue = { name: "vite:vue", transform() {} };
  const ssrStyles = { name: "nuxt:ssr-styles", transform() {} };
  const prehydrate = { name: "nuxt:prehydrate-transform", transform() {} };
  for (const config of [
    [hostVue, ssrStyles, vizePlugins, prehydrate],
    [hostVue, ssrStyles, vizePlugins, prehydrate],
  ]) {
    placeVizeCompileStage(config, stage);
    assert.deepEqual(config, [stage, hostVue, ssrStyles, before.slice(0, 2), prehydrate]);
  }
  // The registered array is shared by the client and server configs.
  assert.deepEqual(vizePlugins, before);

  // Without a host compiler, or without Vize in the config, nothing moves.
  const alone = [ssrStyles, vizePlugins];
  placeVizeCompileStage(alone, stage);
  assert.deepEqual(alone, [ssrStyles, vizePlugins]);
  const foreign = [hostVue, ssrStyles];
  placeVizeCompileStage(foreign, stage);
  assert.deepEqual(foreign, [hostVue, ssrStyles]);
  // Vize's own hookless `vite:vue` shim is not a host compiler.
  const shimOnly = [{ name: "vite:vue" }, vizePlugins];
  placeVizeCompileStage(shimOnly, stage);
  assert.deepEqual(shimOnly, [{ name: "vite:vue" }, vizePlugins]);
});
