import { writeSync } from "node:fs";
import assert from "node:assert/strict";
import { loadRuntime } from "./davinci-mounted-trace.mjs";
import { forbidMountedRuntimeAddons } from "./mounted-runtime-native-boundary.ts";

// Both real bundler producers must exit naturally before this pure-JS
// observer may pass. Its own exit must also remain a real, unmodified 0.
forbidMountedRuntimeAddons();
const development = await loadRuntime({ production: false });
const production = await loadRuntime({ production: true });
assert.equal(typeof development.createApp, "function");
assert.equal(typeof development.createVaporApp, "function");
assert.equal(production.version, development.version);
assert.notEqual(production, development, "runtime modes must not share a module");
assert.equal(await loadRuntime({ production: false }), development);
assert.equal(await loadRuntime({ production: true }), production);
assert.throws(() => process.dlopen({}, "unexpected-observer-addon.node"), {
  message: "mounted observer must not load native addon: unexpected-observer-addon.node",
});
forbidMountedRuntimeAddons();
writeSync(1, '{"passed":1}');
