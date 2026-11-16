// The plugin plan host. The projector imports it only when `vue.plugins` lists
// an entry, so configs without plugins never load or evaluate it.
export { isEmptyPluginPlan, resolvePluginPlan } from "./plugin-plan.js";
export { inspectPluginPlan, refuseUnimplementedCapabilities } from "./plugin-inspect.js";
