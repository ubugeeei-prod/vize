import assert from "node:assert/strict";
import { test } from "node:test";

import { toolArgs, typecheckCorpusGlobs } from "../../tools/fixtures/tool-matrix-command.mjs";

const project = {
  vueGlobs: ["apps/**/*.vue", "playground/src/**/*.vue"],
  tsconfig: "playground/tsconfig.json",
  typecheckPerformance: { corpusGlobs: ["playground/src/**/*.vue"] },
};

test("typechecker args use corpusGlobs and leave compiler on vueGlobs", () => {
  assert.deepEqual(typecheckCorpusGlobs(project), ["playground/src/**/*.vue"]);
  assert.deepEqual(toolArgs(project, "typechecker", "out"), [
    "check",
    "playground/src/**/*.vue",
    "--format",
    "json",
    "--no-config",
    "--tsconfig",
    "playground/tsconfig.json",
  ]);
  const compiler = toolArgs(project, "compiler", "out");
  assert.equal(compiler[0], "build");
  assert.deepEqual(compiler.slice(1, 3), project.vueGlobs);
});

test("typechecker args fall back to vueGlobs when corpusGlobs is omitted", () => {
  const fallback = { vueGlobs: ["src/**/*.vue"], tsconfig: "tsconfig.json" };
  assert.deepEqual(typecheckCorpusGlobs(fallback), ["src/**/*.vue"]);
  assert.deepEqual(toolArgs(fallback, "typechecker", "out"), [
    "check",
    "src/**/*.vue",
    "--format",
    "json",
    "--no-config",
    "--tsconfig",
    "tsconfig.json",
  ]);
});
