// Real Vite builds of SFCs whose scoped and module blocks use sass, scss, less
// or stylus. Vite runs the preprocessor; Vize must scope its output (not the
// source), resolve `v-bind()` to the `useCssVars` names and wire `$style` to
// Vite's CSS-module class map. The preprocessors are the stand-ins from
// npm/builder/shared/test-preprocessors.cjs.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { build, type Rollup } from "vite";

import { vize } from "./index.ts";

const require = createRequire(import.meta.url);
const { installStandIns } = require("../../shared/test-preprocessors.cjs") as {
  installStandIns: (root: string) => void;
};

const BRACED = {
  scoped: `.root { color: v-bind(accent); .title { color: blue; } :deep(.inner) { color: green; } }`,
  module: `.badge { .icon { color: red; } }`,
};
const INDENTED = {
  scoped: `.root\n  color: v-bind(accent)\n  .title\n    color: blue\n  :deep(.inner)\n    color: green\n`,
  module: `.badge\n  .icon\n    color: red\n`,
};
const BODIES: Record<string, typeof BRACED> = {
  scss: BRACED,
  sass: INDENTED,
  less: BRACED,
  stylus: INDENTED,
};

function sfc(lang: string): string {
  const body = BODIES[lang];
  return `<script setup>
const accent = "red";
</script>

<template>
  <div class="root"><p class="title">x</p><span :class="$style.badge" /></div>
</template>

<style scoped lang="${lang}">
${body.scoped}
</style>

<style module lang="${lang}">
${body.module}
</style>
`;
}

async function buildSfc(lang: string): Promise<{ css: string; js: string }> {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), `vize-vite-${lang}-`));
  try {
    installStandIns(root);
    fs.writeFileSync(path.join(root, "App.vue"), sfc(lang));
    fs.writeFileSync(path.join(root, "entry.js"), 'export { default } from "./App.vue";\n');
    const result = (await build({
      root,
      configFile: false,
      logLevel: "silent",
      plugins: [vize({ configMode: false })],
      build: {
        write: false,
        minify: false,
        cssCodeSplit: false,
        lib: { entry: path.join(root, "entry.js"), formats: ["es"], fileName: "app" },
        rollupOptions: { external: ["vue"] },
      },
    })) as Rollup.RollupOutput | Rollup.RollupOutput[];
    const output = (Array.isArray(result) ? result : [result]).flatMap((item) => item.output);
    return {
      css: output
        .filter((item) => item.type === "asset" && item.fileName.endsWith(".css"))
        .map((item) => String((item as Rollup.OutputAsset).source))
        .join("\n"),
      js: output
        .filter((item) => item.type === "chunk")
        .map((item) => (item as Rollup.OutputChunk).code)
        .join("\n"),
    };
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

/** Rule selectors in output order, ignoring the declarations. */
function selectors(css: string): string[] {
  return [...css.matchAll(/([^{}]+)\{[^{}]*\}/g)]
    .map((match) => match[1].replace(/\/\*[^]*?\*\//g, "").trim())
    .filter(Boolean);
}

for (const lang of Object.keys(BODIES)) {
  void test(`vite + ${lang}: preprocessed, then scoped; v-bind and $style line up`, async () => {
    const { css, js } = await buildSfc(lang);
    const scope = /\[(data-v-[0-9a-f]+)\]/.exec(css)?.[1];
    const varName = /useCssVars\)?\(\(?_ctx\)? => \(\{\s*"([\w-]+)": accent/.exec(js)?.[1];
    const badge = /badge: "([\w-]+)"/.exec(js)?.[1];
    const icon = /icon: "([\w-]+)"/.exec(js)?.[1];
    assert.ok(scope && varName && badge && icon, `missing pieces in:\n${css}\n${js}`);

    assert.deepEqual(selectors(css), [
      `.root[${scope}]`,
      `.root .title[${scope}]`,
      `.root[${scope}] .inner`,
      `.${badge} .${icon}`,
    ]);
    assert.match(
      css,
      new RegExp(`\\.root\\[${scope}\\]\\s*\\{\\s*color: var\\(--${varName}\\);?\\s*\\}`),
    );
  });
}
