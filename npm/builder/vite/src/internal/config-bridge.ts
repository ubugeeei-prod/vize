import type { ResolvedConfig } from "vite";

import type { ResolvedVizeConfig } from "../types.ts";

/**
 * Exact Vite-instance registration returned to internal companion plugins.
 *
 * A registered promise resolving to null is intentionally distinct from a
 * missing registration: it means the Vize plugin ran and found no config.
 *
 * @internal
 */
export type ResolvedVizeConfigRegistration =
  | { readonly registered: false }
  | {
      readonly registered: true;
      readonly config: Promise<ResolvedVizeConfig | null>;
    };

const registrations = new WeakMap<ResolvedConfig, Promise<ResolvedVizeConfig | null>>();
const missingRegistration = Object.freeze({ registered: false }) as const;

/** @internal */
export function registerResolvedVizeConfig(
  resolvedConfig: ResolvedConfig,
  config: ResolvedVizeConfig | null | Promise<ResolvedVizeConfig | null>,
): void {
  registrations.set(resolvedConfig, Promise.resolve(config));
}

/** @internal */
export function getResolvedVizeConfigRegistration(
  resolvedConfig: ResolvedConfig,
): ResolvedVizeConfigRegistration {
  if (!registrations.has(resolvedConfig)) {
    return missingRegistration;
  }

  return { registered: true, config: registrations.get(resolvedConfig)! };
}

/** @internal */
export function unregisterResolvedVizeConfig(resolvedConfig: ResolvedConfig): boolean {
  return registrations.delete(resolvedConfig);
}
