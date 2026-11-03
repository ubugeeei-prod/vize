import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { build, type Plugin, type RollupOutput } from "vite";

import { generateOutput } from "./utils/index.ts";
import { vize } from "./index.ts";

void test("unused template-only SFC and scoped CSS leave a production bundle", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-pure-sfc-"));
  try {
    fs.writeFileSync(
      path.join(root, "entry.js"),
      "import { Used } from './barrel.js'; console.log(Used.render());\n",
    );
    fs.writeFileSync(
      path.join(root, "barrel.js"),
      "export { default as Used } from './Used.vue';\nexport { default as Unused } from './Unused.vue';\n",
    );
    for (const name of ["Used", "Unused"]) {
      fs.writeFileSync(
        path.join(root, `${name}.vue`),
        `<template><div class="${name}" /></template>`,
      );
    }

    const plugin: Plugin = {
      name: "vize-template-output-fixture",
      enforce: "pre",
      resolveId(id, importer) {
        if (id.includes("type=style")) return id;
        if (id.endsWith(".vue")) {
          return path.resolve(importer ? path.dirname(importer) : root, id);
        }
        return null;
      },
      load(id) {
        const name = path.basename(id.split("?", 1)[0]).split(".vue", 1)[0];
        if (name !== "Used" && name !== "Unused") return null;
        if (id.includes("type=style")) return `.${name.toLowerCase()}-child{color:red}`;
        if (id.includes("?")) return null;
        return {
          code: generateOutput(
            {
              code: `export function render() { return "${name.toLowerCase()}-child" }\n`,
              scopeId: name.toLowerCase(),
              hasScoped: true,
              styles: [
                {
                  content: ".child { color: red }",
                  lang: "css",
                  scoped: true,
                  module: false,
                  index: 0,
                },
              ],
            },
            { isProduction: true, isDev: false, extractCss: true, filePath: id },
          ),
          moduleSideEffects: false,
        };
      },
    };
    const result = (await build({
      root,
      configFile: false,
      logLevel: "silent",
      plugins: [plugin],
      build: { write: false, minify: false, rollupOptions: { input: path.join(root, "entry.js") } },
    })) as RollupOutput;
    const output = result.output
      .map((item) => (item.type === "asset" ? String(item.source) : item.code))
      .join("\n");
    assert.match(output, /used-child/);
    assert.doesNotMatch(output, /unused-child/);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// #11544: the real plugin marks only side-effect-free template-only SFCs. One
// imported for its global `<style>` keeps its CSS, as with plugin-vue.
void test("a template-only SFC imported for its global style keeps its CSS", async () => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-theme-sfc-")));
  const write = (file: string, content: string) => fs.writeFileSync(path.join(root, file), content);
  try {
    write(
      "entry.js",
      "import './Theme.vue';\nimport { Used } from './barrel.js';\nconsole.log(Used);\n",
    );
    write(
      "barrel.js",
      "export { default as Used } from './Used.vue';\nexport { default as Unused } from './Unused.vue';\n",
    );
    write(
      "Theme.vue",
      "<template><div /></template>\n<style>body { background: #123456 }</style>\n",
    );
    write("Used.vue", "<template><p>used-text</p></template>\n");
    write("Unused.vue", "<template><p>unused-text</p></template>\n");
    for (const scanPatterns of [undefined, []]) {
      const result = (await build({
        root,
        configFile: false,
        logLevel: "silent",
        plugins: [vize({ configMode: false, ...(scanPatterns ? { scanPatterns } : {}) })],
        build: {
          write: false,
          minify: false,
          rollupOptions: { input: path.join(root, "entry.js"), external: [/^vue/] },
        },
      })) as RollupOutput;
      const output = result.output
        .map((item) => (item.type === "asset" ? String(item.source) : item.code))
        .join("\n");
      assert.match(output, /#123456/, "the side-effect import keeps its global style");
      assert.match(output, /used-text/);
      assert.doesNotMatch(output, /unused-text/, "an unused style-less SFC leaves the bundle");
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
