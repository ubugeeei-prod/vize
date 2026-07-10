import assert from "node:assert/strict";

import { buildCompileBatchOptions, buildCompileFileOptions } from "./compile-options.ts";

const fileOptions = buildCompileFileOptions("/src/App.vue", {
  sourceMap: false,
  ssr: false,
  vapor: false,
  experimentalInTagComments: true,
  experimentalPatternedTemplate: true,
  experimentalServerScript: true,
  experimentalSelfReference: true,
});

assert.equal(fileOptions.experimentalInTagComments, true);
assert.equal(fileOptions.experimentalPatternedTemplate, true);
assert.equal(fileOptions.experimentalServerScript, true);
assert.equal(fileOptions.experimentalSelfReference, true);

const batchOptions = buildCompileBatchOptions({
  ssr: false,
  vapor: false,
  experimentalInTagComments: true,
  experimentalPatternedTemplate: true,
  experimentalServerScript: true,
  experimentalSelfReference: true,
});

assert.equal(batchOptions.experimentalInTagComments, true);
assert.equal(batchOptions.experimentalPatternedTemplate, true);
assert.equal(batchOptions.experimentalServerScript, true);
assert.equal(batchOptions.experimentalSelfReference, true);

console.log("vite-plugin-vize compile option tests passed!");
