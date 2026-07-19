import type {
  ApplicationMarquette,
  BackendId,
  CapabilityDefinition,
  CapabilityId,
  EnvironmentId,
  MarquetteBackend,
  MarquetteEnvironment,
  MarquetteProtocol,
  MarquetteRoute,
  ProtocolId,
  ServerEnvironmentId,
  TargetId,
} from "./model.js";

/**
 * Cross-reference constraints derived from the literals in one marquette.
 *
 * Keeping this type separate makes editor diagnostics point at the authored
 * reference instead of widening every identifier to `string`.
 */
export type MarquetteReferenceConstraints<Marquette extends ApplicationMarquette> = {
  readonly capabilities?: {
    readonly [Id in CapabilityId<Marquette>]: CapabilityDefinition & { readonly id: Id };
  };
  readonly environments?: readonly (MarquetteEnvironment<string, EnvironmentId<Marquette>> & {
    readonly capabilities?: readonly CapabilityId<Marquette>[];
    readonly target: TargetId<Marquette>;
  })[];
  readonly backends?: readonly (MarquetteBackend<string, ServerEnvironmentId<Marquette>> & {
    readonly capabilities?: readonly CapabilityId<Marquette>[];
  })[];
  readonly protocols?: readonly (MarquetteProtocol<string, BackendId<Marquette>> & {
    readonly capabilities?: readonly CapabilityId<Marquette>[];
  })[];
  readonly routes?: readonly (MarquetteRoute<
    string,
    EnvironmentId<Marquette>,
    BackendId<Marquette>,
    ProtocolId<Marquette>
  > & { readonly capabilities?: readonly CapabilityId<Marquette>[] })[];
};

/**
 * Defines an application marquette while preserving every authored literal.
 *
 * Environment dependencies, backend owners, protocol owners, and route
 * references are checked against identifiers declared in the same object.
 * The function returns its input without allocation or runtime work.
 *
 * @example
 * ```ts
 * const app = defineApplicationMarquette({
 *   application: "shop",
 *   targets: ["web"],
 *   environments: [
 *     { id: "web", target: "web", consumer: "client", runtime: "browser" },
 *   ],
 *   routes: [
 *     { id: "home", path: "/", environment: "web", rendering: "client" },
 *   ],
 * });
 * ```
 */
export function defineApplicationMarquette<const Marquette extends ApplicationMarquette>(
  marquette: Marquette & MarquetteReferenceConstraints<NoInfer<Marquette>>,
): Marquette {
  return marquette;
}
