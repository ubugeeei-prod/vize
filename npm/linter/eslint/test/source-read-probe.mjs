import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { ESLint } from "eslint";
import plugin from "../dist/index.mjs";

const filename = path.resolve(process.argv[2]);
fs.mkdirSync(path.dirname(filename), { recursive: true });
const diskSource = '<template>😀<img src="disk"></template>\r\n';
fs.writeFileSync(filename, diskSource);
const eslint = new ESLint({
  cwd: path.dirname(filename),
  overrideConfigFile: true,
  overrideConfig: [
    {
      ...plugin.configs.essential,
      rules: { "vize/a11y/alt-text": "error", "vize/vue/no-multi-spaces": "error" },
    },
  ],
});
const unsavedSource = '<template>😀<img alt="unsaved"></template>\r\n';
const clean = await eslint.lintText(unsavedSource, { filePath: filename });
const bad = await eslint.lintText(diskSource, { filePath: filename });
assert.deepEqual(clean[0].messages, []);
assert.equal(bad[0].messages.length, 1);
// Do not read the physical source here: strace independently checks every
// child open, including calls originating in Rust, against this exact path.
console.log(JSON.stringify({ filename, diskSource, unsavedSource, clean, bad }));
