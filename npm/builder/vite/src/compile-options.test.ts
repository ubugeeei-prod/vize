import assert from "node:assert/strict";

import { buildCompileBatchOptions, buildCompileFileOptions } from "./compile-options.ts";

const fileOptions = buildCompileFileOptions("/src/App.vue", {
  sourceMap: false,
  ssr: false,
  vapor: false,
  experimentalInTagComments: true,
  experimentalPatternedTemplate: true,
  experimentalServerScript: true,
});

assert.equal(fileOptions.experimentalInTagComments, true);
assert.equal(fileOptions.experimentalPatternedTemplate, true);
assert.equal(fileOptions.experimentalServerScript, true);

const batchInput = {
  sourceMap: false,
  ssr: false,
  vapor: false,
  experimentalInTagComments: true,
  experimentalPatternedTemplate: true,
  experimentalServerScript: true,
};
const batchOptions = buildCompileBatchOptions(batchInput);

assert.equal(batchOptions.experimentalInTagComments, true);
assert.equal(batchOptions.experimentalPatternedTemplate, true);
assert.equal(batchOptions.experimentalServerScript, true);
assert.equal(batchOptions.includeSourceMap, false);

// The batch options object is also the pre-compile cache key material, so the
// source-map decision has to be visible in it (#3399).
assert.equal(buildCompileBatchOptions({ ...batchInput, sourceMap: true }).includeSourceMap, true);

console.log("vite-plugin-vize compile option tests passed!");
