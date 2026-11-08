import { defineComponent, h } from "vue";

import { useFrescoTheme } from "./theme.js";

/** Test component that exposes the nearest resolved theme on a render node. */
export const ThemeProbe = defineComponent({
  name: "ThemeProbe",
  setup() {
    const theme = useFrescoTheme();
    return () =>
      h("text", {
        accent: theme.value.colors.accent,
        primary: theme.value.colors.primary,
        reducedMotion: theme.value.motion.reduced,
      });
  },
});
