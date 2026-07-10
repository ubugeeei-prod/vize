import assert from "node:assert/strict";

import {
  resolveExperimentalCompilerOptions,
  resolveExperimentalOptions,
} from "./experimentals.ts";

const options = resolveExperimentalOptions(
  {
    selfReference: {},
    "server script": {},
  },
  {
    self_reference: false,
  },
);

assert.equal(options.selfReference, true);
assert.equal(options.serverScript, true);

const compilerOptions = resolveExperimentalCompilerOptions(
  {
    experimentals: {
      "self reference": {},
    },
  },
  {},
  undefined,
);

assert.equal(compilerOptions.experimentalSelfReference, true);

console.log("vite-plugin-vize experimental option tests passed!");
