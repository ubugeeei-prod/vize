/**
 * Kitty keyboard protocol constants compatible with Ink.
 */

/** Bit values defined by the Kitty keyboard progressive-enhancement protocol. */
export const kittyFlags = {
  disambiguateEscapeCodes: 1,
  reportEventTypes: 2,
  reportAlternateKeys: 4,
  reportAllKeysAsEscapeCodes: 8,
  reportAssociatedText: 16,
} as const;

/** Supported Kitty keyboard protocol flag name. */
export type KittyFlagName = keyof typeof kittyFlags;

/** Kitty keyboard protocol negotiation options. */
export interface KittyKeyboardOptions {
  /** Whether protocol negotiation is automatic, forced, or disabled. @default "auto" */
  mode?: "auto" | "enabled" | "disabled";
  /** Progressive-enhancement flags requested from the terminal. @default [] */
  flags?: KittyFlagName[];
}

/** Combine named Kitty keyboard flags into the protocol bit mask. */
export function resolveKittyFlags(flags: KittyFlagName[]): number {
  return flags.reduce((bits, flag) => bits | kittyFlags[flag], 0);
}

/** Modifier bit values reported by the Kitty keyboard protocol. */
export const kittyModifiers = {
  shift: 1,
  alt: 2,
  ctrl: 4,
  super: 8,
  hyper: 16,
  meta: 32,
  capsLock: 64,
  numLock: 128,
} as const;
