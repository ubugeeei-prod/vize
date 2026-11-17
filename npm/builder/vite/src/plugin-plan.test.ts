/**
 * A resolved `vue.plugins` plan (#13074) is registered with the native
 * compiler once and named by its digest in every compile request.
 */
import assert from "node:assert/strict";
import fs from "node:fs";

import { buildCompileBatchOptions, buildCompileFileOptions } from "./compile-options.ts";
import { compileFile, registerPluginPlan } from "./compiler.ts";

const fixture = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../../../tests/_fixtures/plugin-plan/resolve/lint-rule-complete.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as { expect: { digest: string; plan: unknown } };
const projected = { digest: fixture.expect.digest, plan: fixture.expect.plan };

// No plan: nothing is registered and no option is sent.
assert.equal(registerPluginPlan(null), undefined);
assert.equal(registerPluginPlan({ entries: [] }), undefined);
assert.equal(
  "pluginPlan" in
    buildCompileFileOptions("/src/App.vue", { sourceMap: false, ssr: false, vapor: false }),
  false,
);

// One registration per resolved config object; the digest is the plan's.
const config = { entries: [], __vizePluginPlan: projected };
const digest = registerPluginPlan(config);
assert.equal(digest, fixture.expect.digest);
assert.equal(registerPluginPlan(config), digest);

const options = { sourceMap: false, ssr: false, vapor: false, pluginPlan: digest };
assert.equal(buildCompileFileOptions("/src/App.vue", options).pluginPlan, digest);
assert.equal(buildCompileBatchOptions(options).pluginPlan, digest);

// A lint-only plan leaves the compile unchanged.
const source = `<template><a v-focus.x="1">go</a></template>`;
assert.equal(
  compileFile("/src/Plain.vue", new Map(), options, source).code,
  compileFile("/src/Plain.vue", new Map(), { ...options, pluginPlan: undefined }, source).code,
);

// A tampered projection and an unknown digest fail closed.
assert.throws(
  () => registerPluginPlan({ __vizePluginPlan: { digest: "0".repeat(64), plan: projected.plan } }),
  /plugin\/plan-stale/,
);
assert.throws(
  () =>
    compileFile("/src/Stale.vue", new Map(), { ...options, pluginPlan: "f".repeat(64) }, source),
  /plugin\/plan-stale/,
);
