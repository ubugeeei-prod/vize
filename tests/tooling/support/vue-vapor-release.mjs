// Historical and current Vue Vapor runtime contracts.
//
// Vue 3.6 changed the insertion model between betas and release candidates
// (`setInsertionState(parent, anchor?)`, trailing blocks without template
// placeholders), so mounted-behavior and upstream-parity harnesses must run
// Each observation pairs Vize and the official compiler with the same exact
// runtime graph. The original rc.9 aliases below are retained for historical
// regression evidence; they do not establish support for current generated
// listener APIs. The current experimental target is rc.11, exposed explicitly
// below. A failed older-runtime row remains failed rather than target credit.
import assert from "node:assert/strict";
import { createRequire } from "node:module";

const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
const fromSfc = createRequire(fromVue.resolve("@vue/compiler-sfc"));

/** Version of the historical pinned regression release. */
export const vueVaporVersion = fromVue("./package.json").version;

/** Historical pinned release's bundler runtime entry. */
export const vueVaporRuntimeEntry = fromUi.resolve(
  "vue-vapor-runtime/dist/vue.runtime.esm-bundler.js",
);

/** Historical pinned release's browser build carrying both renderers. */
export const vueVaporBrowserRuntime = fromUi.resolve(
  "vue-vapor-runtime/dist/vue.runtime-with-vapor.esm-browser.js",
);

/** Historical release's official compiler, retained with its original graph. */
export const officialCompilerVapor = fromSfc("@vue/compiler-vapor");

assert.equal(fromSfc("@vue/compiler-vapor/package.json").version, vueVaporVersion);

// The newest release Vize supports (`vue-vapor-runtime-supported`). Vapor
// expected outputs follow it, so a protocol change is visible as a diff.
const fromSupported = createRequire(fromUi.resolve("vue-vapor-runtime-supported/package.json"));
const fromSupportedSfc = createRequire(fromSupported.resolve("@vue/compiler-sfc"));

/** Version of the newest supported Vue release. */
export const vueVaporSupportedVersion = fromSupported("./package.json").version;

/** The official `@vue/compiler-vapor` of the newest supported release. */
export const supportedCompilerVapor = fromSupportedSfc("@vue/compiler-vapor");

assert.equal(
  fromSupportedSfc("@vue/compiler-vapor/package.json").version,
  vueVaporSupportedVersion,
);

/** Current experimental codegen/runtime API target, independent of old receipts. */
export const vueVaporTargetVersion = vueVaporSupportedVersion;
export const targetCompilerVapor = supportedCompilerVapor;
export const vueVaporTargetBrowserRuntime = fromUi.resolve(
  "vue-vapor-runtime-supported/dist/vue.runtime-with-vapor.esm-browser.js",
);
