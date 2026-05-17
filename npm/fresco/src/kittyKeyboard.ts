/**
 * Kitty keyboard protocol constants compatible with Ink.
 */

export const kittyFlags = {
  disambiguateEscapeCodes: 1,
  reportEventTypes: 2,
  reportAlternateKeys: 4,
  reportAllKeysAsEscapeCodes: 8,
  reportAssociatedText: 16,
} as const;

export type KittyFlagName = keyof typeof kittyFlags;

export interface KittyKeyboardOptions {
  mode?: "auto" | "enabled" | "disabled";
  flags?: KittyFlagName[];
}

export function resolveKittyFlags(flags: KittyFlagName[]): number {
  return flags.reduce((bits, flag) => bits | kittyFlags[flag], 0);
}

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
