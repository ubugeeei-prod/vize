// The Vue release whose Vapor runtime Vize's Vapor codegen targets.
//
// Vue 3.6 changed the insertion model between betas and release candidates
// (`setInsertionState(parent, anchor?)`, trailing blocks without template
// placeholders), so mounted-behavior and upstream-parity harnesses must run
// Vize output and the official compiler-vapor output under the same release
// Vize emits for. It is the `vue-vapor-runtime` alias `@vizejs/ui` pins for
// its Vapor runtime conformance lane; the official compiler is resolved
// through that runtime's own dependency graph so the two cannot drift.
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
const fromSfc = createRequire(fromVue.resolve("@vue/compiler-sfc"));

/** Version of the targeted Vue release. */
export const vueVaporVersion = fromVue("./package.json").version;

/** Absolute path of the targeted release's bundler runtime entry. */
export const vueVaporRuntimeEntry = fromUi.resolve(
  "vue-vapor-runtime/dist/vue.runtime.esm-bundler.js",
);

/** Absolute path of the targeted release's browser build carrying both renderers. */
export const vueVaporBrowserRuntime = fromUi.resolve(
  "vue-vapor-runtime/dist/vue.runtime-with-vapor.esm-browser.js",
);

/** The official `@vue/compiler-vapor` of the targeted release. */
export const officialCompilerVapor = fromSfc("@vue/compiler-vapor");

assert.equal(fromSfc("@vue/compiler-vapor/package.json").version, vueVaporVersion);
