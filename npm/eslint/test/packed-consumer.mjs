import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import { ESLint } from "eslint";
import plugin from "eslint-plugin-vize";

const require = createRequire(import.meta.url);
const native = require("@vizejs/native");
const nativeImages = Object.values(require.cache)
  .filter((entry) => entry?.filename.endsWith(".node") && entry.exports === native)
  .map((entry) => ({
    path: fs.realpathSync(entry.filename),
    sha256: createHash("sha256").update(fs.readFileSync(entry.filename)).digest("hex"),
  }));
assert.equal(plugin.meta.name, "eslint-plugin-vize");
const eslint = new ESLint({
  overrideConfigFile: true,
  overrideConfig: [
    {
      ...plugin.configs.recommended,
      rules: { "vize/a11y/alt-text": "error", "vize/vue/no-multi-spaces": "error" },
    },
  ],
  fix: true,
});
const source =
  '<template>😀<img src="logo"><div  id="x"></div></template>\r\n<script setup lang="ts">const n: number = 1;</script>\r\n';
const result = await eslint.lintText(source, { filePath: "Packed.vue" });
assert.equal(result.length, 1);
assert.equal(result[0].messages.length, 1);
assert.equal(result[0].messages[0].ruleId, "vize/a11y/alt-text");
assert.equal(result[0].output, source.replace("div  id", "div id"));
console.log(
  JSON.stringify({
    plugin: plugin.meta,
    eslintVersion: ESLint.version,
    nativeBuildToken: native.getNativeBuildToken(),
    nativeImages,
    pluginEntry: import.meta.resolve("eslint-plugin-vize"),
    source,
    result,
  }),
);
