// The plugin plan host. The projector imports it only when `vue.plugins` lists
// an entry, so configs without plugins never load or evaluate it.
export {
  isEmptyPluginPlan,
  refuseUnimplementedCapabilities,
  resolvePluginPlan,
} from "./plugin-plan.js";
