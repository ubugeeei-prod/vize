import type { AdapterCapabilitySupport } from "./adapter-model.js";

/** Current contract version shared by the native-engine capability profile. */
export const NATIVE_ENGINE_CAPABILITY_VERSION = 1 as const;

/** Closed set of capability identifiers required from a native rendering engine. */
export const NATIVE_ENGINE_CAPABILITY_IDS = [
  "native.rendering",
  "native.events",
  "native.layout",
  "native.text",
  "native.images",
  "native.animation",
  "native.accessibility",
  "native.lifecycle",
] as const;

/** One identifier from the canonical native-engine capability profile. */
export type NativeEngineCapabilityId = (typeof NATIVE_ENGINE_CAPABILITY_IDS)[number];

/**
 * Creates the canonical version-one native-engine capability profile.
 *
 * A fresh list is returned so an adapter can extend its manifest without
 * mutating the shared profile seen by another consumer.
 */
export function nativeEngineCapabilityProfile(): readonly AdapterCapabilitySupport[] {
  return NATIVE_ENGINE_CAPABILITY_IDS.map((id) => ({
    id,
    minVersion: NATIVE_ENGINE_CAPABILITY_VERSION,
    maxVersion: NATIVE_ENGINE_CAPABILITY_VERSION,
  }));
}
