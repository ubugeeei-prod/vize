import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { build, type Plugin, type RollupOutput } from "vite";

import { generateOutput } from "./utils/index.ts";

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
