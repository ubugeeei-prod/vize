import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { MediaPreferences } from "./media-preferences-types.ts";

/** Effective media preferences published by MediaPreferencesProvider. */
export const mediaPreferencesContext =
  createContext<ComputedRef<MediaPreferences>>("MediaPreferences");
