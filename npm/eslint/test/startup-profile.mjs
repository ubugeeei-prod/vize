import assert from "node:assert/strict";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const begin = process.hrtime.bigint();
const { Linter } = await import("eslint");
const eslintReady = process.hrtime.bigint();
const native = require("@vizejs/native");
const nativeReady = process.hrtime.bigint();
let metadataCalls = 0;
let lintCalls = 0;
const originalMetadata = native.getPatinaRules;
const originalLint = native.lintPatinaSfc;
native.getPatinaRules = (...args) => {
  metadataCalls++;
  return originalMetadata(...args);
};
native.lintPatinaSfc = (...args) => {
  lintCalls++;
  return originalLint(...args);
};
const { default: plugin } = await import("../dist/index.mjs");
const pluginReady = process.hrtime.bigint();
const source = '<template>😀<img src="profile"></template>\r\n';
const messages = new Linter().verify(
  source,
  [
    {
      ...plugin.configs.essential,
      rules: { "vize/a11y/alt-text": "error" },
    },
  ],
  { filename: "Startup.vue" },
);
const end = process.hrtime.bigint();
assert.equal(messages.length, 1);
assert.equal(messages[0].ruleId, "vize/a11y/alt-text");
assert.equal(lintCalls, 1);
console.log(
  JSON.stringify({
    eslintVersion: Linter.version,
    nativeBuildToken: native.getNativeBuildToken(),
    phasesNanoseconds: {
      eslintImport: String(eslintReady - begin),
      nativeRequire: String(nativeReady - eslintReady),
      pluginImportIncludingParsers: String(pluginReady - nativeReady),
      firstVerify: String(end - pluginReady),
    },
    metadataCalls,
    lintCalls,
    source,
    messages,
    scope:
      "Fresh Node process; shared filesystem caches are not reset. Node import/NAPI boundaries, not internal Rust phases or a cross-tool speed ratio.",
  }),
);
