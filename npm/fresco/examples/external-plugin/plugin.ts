import { defineFrescoPlugin } from "@vizejs/fresco";

import ExternalMeter from "./ExternalMeter.vue";
import { externalMeterTheme } from "./provider.js";

export const externalMeterPlugin = defineFrescoPlugin({
  components: { ExternalMeter },
  setup(app) {
    externalMeterTheme.provide(app, {
      filled: "■",
      empty: "·",
      foreground: "green",
    });
  },
});
