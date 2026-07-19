/** Current serialized application-marquette format. */
export const MARQUETTE_FORMAT_VERSION = 1 as const;

/** User-visible target produced by an application marquette. */
export type MarquetteTarget = "web" | "native" | "desktop" | "terminal";

/** Runtime family that executes one environment. */
export type RuntimeFamily =
  | "browser"
  | "javascript"
  | "rust"
  | "go"
  | "jvm"
  | "native"
  | "desktop"
  | "terminal"
  | "external";

/** Side of the application graph that consumes an environment. */
export type EnvironmentConsumer = "client" | "server";

/** Backend implementation family selected by an application. */
export type BackendFamily = "javascript" | "rust" | "go" | "jvm" | "external";

/** Semantic family used by one data or procedure endpoint. */
export type ProtocolFamily = "schema-query" | "typed-rpc" | "binary-rpc";

/** Rendering strategy selected for one route or target view. */
export type RenderingMode =
  | "client"
  | "static"
  | "server"
  | "stream"
  | "partial"
  | "hybrid"
  | "native"
  | "desktop"
  | "terminal";

/** Versioned capability understood by adapters and developer tools. */
export interface CapabilityDefinition {
  /** Stable lowercase capability identifier. */
  readonly id: string;
  /** Summary shown by diagnostics, documentation, and AI tooling. */
  readonly description: string;
  /**
   * Capability contract version.
   *
   * @default 1
   */
  readonly version?: number;
}

/** Independently transformed and cached application environment. */
export interface MarquetteEnvironment<
  Id extends string = string,
  DependencyId extends string = string,
> {
  /** Stable identifier, unique within the application. */
  readonly id: Id;
  /** User-visible target served by this environment. */
  readonly target: MarquetteTarget;
  /** Whether output is consumed by an end user or a trusted runtime. */
  readonly consumer: EnvironmentConsumer;
  /** Runtime family that executes the output. */
  readonly runtime: RuntimeFamily;
  /**
   * Authored entry module.
   *
   * @default undefined
   */
  readonly entry?: string;
  /**
   * Environments that must build first.
   *
   * @default []
   */
  readonly dependsOn?: readonly DependencyId[];
  /**
   * Capabilities required from the environment adapter.
   *
   * @default []
   */
  readonly capabilities?: readonly string[];
}

/** Backend application or independently deployed service. */
export interface MarquetteBackend<
  Id extends string = string,
  EnvironmentId extends string = string,
> {
  /** Stable backend identifier. */
  readonly id: Id;
  /** Backend implementation family. */
  readonly family: BackendFamily;
  /**
   * Server environment that owns local execution.
   *
   * @default undefined
   */
  readonly environment?: EnvironmentId;
  /**
   * Capabilities required from the backend adapter.
   *
   * @default []
   */
  readonly capabilities?: readonly string[];
}

/** Protocol endpoint exposed by a backend. */
export interface MarquetteProtocol<Id extends string = string, BackendId extends string = string> {
  /** Stable endpoint identifier. */
  readonly id: Id;
  /** Protocol semantic family. */
  readonly family: ProtocolFamily;
  /** Backend that owns the endpoint. */
  readonly backend: BackendId;
  /**
   * Capabilities required from the protocol adapter.
   *
   * @default []
   */
  readonly capabilities?: readonly string[];
}

/** Typed application route or target view. */
export interface MarquetteRoute<
  Id extends string = string,
  EnvironmentId extends string = string,
  BackendId extends string = string,
  ProtocolId extends string = string,
> {
  /** Stable identifier used by generated navigation APIs. */
  readonly id: Id;
  /** Logical path, always beginning with `/`. */
  readonly path: `/${string}`;
  /** Environment that owns the rendered route. */
  readonly environment: EnvironmentId;
  /** Rendering strategy used by the route. */
  readonly rendering: RenderingMode;
  /**
   * Backend used by data loading, mutations, or server rendering.
   *
   * @default undefined
   */
  readonly backend?: BackendId;
  /**
   * Protocol endpoint used as the route data contract.
   *
   * @default undefined
   */
  readonly protocol?: ProtocolId;
  /**
   * Capabilities required by the route.
   *
   * @default []
   */
  readonly capabilities?: readonly string[];
}

/** Complete, versioned application marquette. */
export interface ApplicationMarquette<
  Environments extends readonly MarquetteEnvironment[] = readonly MarquetteEnvironment[],
  Backends extends readonly MarquetteBackend[] = readonly MarquetteBackend[],
  Protocols extends readonly MarquetteProtocol[] = readonly MarquetteProtocol[],
  Routes extends readonly MarquetteRoute[] = readonly MarquetteRoute[],
> {
  /**
   * Serialized contract format.
   *
   * @default 1
   */
  readonly formatVersion?: typeof MARQUETTE_FORMAT_VERSION;
  /** Stable lowercase application identifier. */
  readonly application: string;
  /**
   * User-visible application targets.
   *
   * @default []
   */
  readonly targets?: readonly MarquetteTarget[];
  /**
   * Versioned capability definitions keyed by identifier.
   *
   * @default {}
   */
  readonly capabilities?: Readonly<Record<string, CapabilityDefinition>>;
  /**
   * Independently transformed application environments.
   *
   * @default []
   */
  readonly environments?: Environments;
  /**
   * Backend applications and services.
   *
   * @default []
   */
  readonly backends?: Backends;
  /**
   * Protocol endpoints.
   *
   * @default []
   */
  readonly protocols?: Protocols;
  /**
   * Typed routes and target views.
   *
   * @default []
   */
  readonly routes?: Routes;
}

/** Extracts identifiers only when a collection is present in the authored object. */
type DeclaredId<Marquette, Collection extends PropertyKey> = Marquette extends {
  readonly [Key in Collection]: readonly { readonly id: infer Id extends string }[];
}
  ? Id
  : never;

/** Exact environment identifier union inferred from a marquette. */
export type EnvironmentId<Marquette extends ApplicationMarquette> = DeclaredId<
  Marquette,
  "environments"
>;

/** Exact server-owned environment identifier union inferred from a marquette. */
export type ServerEnvironmentId<Marquette extends ApplicationMarquette> = Marquette extends {
  readonly environments: readonly (infer Environment)[];
}
  ? Environment extends {
      readonly consumer: "server";
      readonly id: infer Id extends string;
    }
    ? Id
    : never
  : never;

/** Exact backend identifier union inferred from a marquette. */
export type BackendId<Marquette extends ApplicationMarquette> = DeclaredId<Marquette, "backends">;

/** Exact protocol identifier union inferred from a marquette. */
export type ProtocolId<Marquette extends ApplicationMarquette> = DeclaredId<Marquette, "protocols">;

/** Exact route identifier union inferred from a marquette. */
export type RouteId<Marquette extends ApplicationMarquette> = DeclaredId<Marquette, "routes">;

/** Exact target union inferred from the authored target list. */
export type TargetId<Marquette extends ApplicationMarquette> = Marquette extends {
  readonly targets: readonly (infer Target extends MarquetteTarget)[];
}
  ? Target
  : never;

/** Exact capability identifier union inferred from authored capability keys. */
export type CapabilityId<Marquette extends ApplicationMarquette> = Marquette extends {
  readonly capabilities: Readonly<Record<infer Id extends string, CapabilityDefinition>>;
}
  ? Id
  : never;
