/** Compile-only assertions for the Fresco plugin and provider API. */

import { defineComponent } from "vue";

import { createApp } from "./app.js";
import { defineFrescoPlugin, defineFrescoProvider } from "./plugins.js";

const density = defineFrescoProvider<"compact" | "comfortable">("density", "comfortable");
const component = defineComponent({ name: "ExternalComponent", template: "" });
const plugin = defineFrescoPlugin({
  components: { ExternalComponent: component },
  setup(app) {
    density.provide(app, "compact");
  },
});

export const configuredApp = createApp(component).use(plugin);
export const injectedDensity: "compact" | "comfortable" = density.use();

// @ts-expect-error - provider values retain their declared union.
density.provide({} as Parameters<typeof density.provide>[0], "wide");

// @ts-expect-error - Fresco app plugins use Vue's option tuple contract.
createApp(component).use(plugin, "unexpected-option");
