/**
 * useIsScreenReaderEnabled - screen reader mode flag.
 */

export function useIsScreenReaderEnabled(): boolean {
  return process.env.INK_SCREEN_READER === "true" || process.env.FRESCO_SCREEN_READER === "true";
}
