import { computed, type Ref } from "vue";

import {
  defaultFrescoTheme,
  provideFrescoTheme,
  resolveFrescoTheme,
  type FrescoTheme,
  type FrescoThemeOverride,
} from "./theme.js";

export const validOverride: FrescoThemeOverride = {
  colors: { accent: "yellow" },
  spacing: { md: 3 },
  borders: { strong: "heavy" },
  motion: { reduced: true },
};

export const resolvedTheme: FrescoTheme = resolveFrescoTheme(defaultFrescoTheme, validOverride);
export const reactiveTheme: Readonly<Ref<FrescoTheme>> = provideFrescoTheme(
  computed(() => validOverride),
);

// @ts-expect-error - spacing tokens are terminal-cell counts.
export const invalidSpacing: FrescoThemeOverride = { spacing: { md: "2" } };

// @ts-expect-error - borders remain inside the native border protocol.
export const invalidBorder: FrescoThemeOverride = { borders: { regular: "dotted" } };
