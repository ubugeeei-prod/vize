/**
 * useIsScreenReaderEnabled - screen reader mode flag.
 */

import { inject } from "vue";
import { SCREEN_READER_KEY, isScreenReaderEnabledByDefault } from "../accessibility.js";

/** Read the current screen-reader mode as a boolean snapshot. */
export function useIsScreenReaderEnabled(): boolean {
  const enabled = inject(SCREEN_READER_KEY, null);
  return enabled?.value ?? isScreenReaderEnabledByDefault();
}
