/**
 * useIsScreenReaderEnabled - screen reader mode flag.
 */

import { inject } from "@vue/runtime-core";
import { SCREEN_READER_KEY, isScreenReaderEnabledByDefault } from "../accessibility.js";

export function useIsScreenReaderEnabled(): boolean {
  const enabled = inject(SCREEN_READER_KEY, null);
  return enabled?.value ?? isScreenReaderEnabledByDefault();
}
