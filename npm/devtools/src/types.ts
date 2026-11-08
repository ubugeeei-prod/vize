/** Source-owned Vue file that a trace event or panel belongs to. */
export type DevtoolsSourceSpecifier = `${string}.vue`;

/** Stable event-kind vocabulary emitted by the devtools trace recorder. */
export type DevtoolsEventKind =
  | "render:queued"
  | "render:start"
  | "render:end"
  | "update:scheduled"
  | "update:committed"
  | "reactivity:track"
  | "reactivity:trigger"
  | "provide:set"
  | "inject:resolve"
  | "suspense:pending"
  | "suspense:resolve"
  | "suspense:error";

interface DevtoolsEventBase<Kind extends DevtoolsEventKind> {
  readonly kind: Kind;
  readonly timestamp?: number;
  readonly requestId?: string;
}

/** Source-owned identity shared by component render and update events. */
export interface DevtoolsComponentIdentity {
  readonly componentId: string;
  readonly componentName: string;
  readonly source: DevtoolsSourceSpecifier;
}

/** Component render or update lifecycle event accepted by the recorder. */
export type DevtoolsComponentEvent =
  | (DevtoolsEventBase<"render:queued" | "render:start"> & DevtoolsComponentIdentity)
  | (DevtoolsEventBase<"render:end"> & DevtoolsComponentIdentity & { readonly durationMs: number })
  | (DevtoolsEventBase<"update:scheduled" | "update:committed"> &
      DevtoolsComponentIdentity & { readonly reason: string });

/** Reactive dependency tracking or invalidation event accepted by the recorder. */
export type DevtoolsReactiveEvent =
  | (DevtoolsEventBase<"reactivity:track"> & {
      readonly effectId: string;
      readonly effectLabel: string;
      readonly sourceId: string;
      readonly sourceLabel: string;
      readonly componentId?: string;
      readonly source?: DevtoolsSourceSpecifier;
    })
  | (DevtoolsEventBase<"reactivity:trigger"> & {
      readonly effectId: string;
      readonly sourceId: string;
      readonly operation: "set" | "delete" | "clear" | "array-mutation" | "custom";
      readonly componentId?: string;
      readonly source?: DevtoolsSourceSpecifier;
    });

/** Provide/inject ownership event used to reconstruct the dependency tree. */
export type DevtoolsProvideEvent =
  | (DevtoolsEventBase<"provide:set"> & {
      readonly providerId: string;
      readonly key: string;
      readonly ownerComponentId: string;
      readonly ownerComponentName: string;
      readonly source: DevtoolsSourceSpecifier;
    })
  | (DevtoolsEventBase<"inject:resolve"> & {
      readonly injectorId: string;
      readonly injectorComponentId: string;
      readonly injectorComponentName: string;
      readonly key: string;
      readonly providerId?: string;
      readonly status: "resolved" | "shadowed" | "unresolved";
      readonly source: DevtoolsSourceSpecifier;
    });

/** Suspense boundary state transition accepted by the recorder. */
export type DevtoolsSuspenseEvent =
  | (DevtoolsEventBase<"suspense:pending" | "suspense:resolve"> & {
      readonly boundaryId: string;
      readonly componentId: string;
      readonly componentName: string;
      readonly parentBoundaryId?: string;
      readonly asyncDependency?: string;
      readonly source: DevtoolsSourceSpecifier;
    })
  | (DevtoolsEventBase<"suspense:error"> & {
      readonly boundaryId: string;
      readonly componentId: string;
      readonly componentName: string;
      readonly parentBoundaryId?: string;
      readonly error: string;
      readonly source: DevtoolsSourceSpecifier;
    });

/** Complete discriminated event union accepted by a devtools recorder. */
export type DevtoolsEvent =
  | DevtoolsComponentEvent
  | DevtoolsReactiveEvent
  | DevtoolsProvideEvent
  | DevtoolsSuspenseEvent;

/** Devtools event after the recorder assigns ordering and time metadata. */
export type SerializedDevtoolsEvent = DevtoolsEvent & {
  readonly sequence: number;
  readonly timestamp: number;
};

/**
 * Capability-negotiated observer boundary for opt-in JavaScript visitors.
 * The recorder owns serialization so the default path never imports a plugin SDK.
 */
export interface DevtoolsTraceVisitorHost {
  readonly name: string;
  readonly version: string;
  readonly fingerprint: string;
  readonly run: (deliveryJson: string) => string;
}

/** Aggregated render state for one source-owned component. */
export interface DevtoolsRenderNode {
  readonly componentId: string;
  readonly componentName: string;
  readonly source: DevtoolsSourceSpecifier;
  readonly renderCount: number;
  readonly updateCount: number;
  readonly lastDurationMs: number | null;
  readonly lastEventKind: DevtoolsEventKind;
  readonly lastTimestamp: number;
}

/** Source or effect node in the derived reactive dependency graph. */
export interface DevtoolsReactiveNode {
  readonly id: string;
  readonly label: string;
  readonly kind: "effect" | "source";
}

/** Counted tracking or trigger relationship in the reactive dependency graph. */
export interface DevtoolsReactiveEdge {
  readonly sourceId: string;
  readonly targetId: string;
  readonly operation: "track" | "trigger";
  readonly count: number;
}

/** Provider node with its keys and resolved or unresolved consumers. */
export interface DevtoolsProvideTreeNode {
  readonly providerId: string;
  readonly ownerComponentId: string;
  readonly ownerComponentName: string;
  readonly providedKeys: readonly string[];
  readonly consumers: readonly DevtoolsProvideConsumer[];
}

/** Consumer resolution recorded beneath a provider node. */
export interface DevtoolsProvideConsumer {
  readonly injectorId: string;
  readonly injectorComponentId: string;
  readonly injectorComponentName: string;
  readonly key: string;
  readonly status: "resolved" | "shadowed" | "unresolved";
}

/** Latest derived state for one Suspense boundary. */
export interface DevtoolsSuspenseNode {
  readonly boundaryId: string;
  readonly componentId: string;
  readonly componentName: string;
  readonly parentBoundaryId: string | null;
  readonly state: "pending" | "resolved" | "error";
  readonly asyncDependency: string | null;
  readonly error: string | null;
  readonly updatedAt: number;
}

/** Serializable, request-local snapshot derived from recorded trace events. */
export interface DevtoolsSnapshot {
  readonly schemaVersion: 1;
  readonly enabled: boolean;
  readonly sessionId: string;
  readonly createdAt: number;
  readonly events: readonly SerializedDevtoolsEvent[];
  readonly renderTree: readonly DevtoolsRenderNode[];
  readonly reactiveGraph: {
    readonly nodes: readonly DevtoolsReactiveNode[];
    readonly edges: readonly DevtoolsReactiveEdge[];
  };
  readonly provideTree: readonly DevtoolsProvideTreeNode[];
  readonly suspenseTree: readonly DevtoolsSuspenseNode[];
}

/** Options controlling recorder state, retention, time, and opt-in visitors. */
export interface DevtoolsRecorderOptions {
  /** Whether record calls retain and deliver events. @default true */
  readonly enabled?: boolean;
  /** Maximum retained events before the oldest events are discarded. @default 2000 */
  readonly maxEvents?: number;
  /** Clock used to timestamp events and snapshots. @default Date.now */
  readonly now?: () => number;
  /** Stable trace session identifier. @default snapshot.sessionId or "vize-devtools" */
  readonly sessionId?: string;
  /** Snapshot used to hydrate initial recorder state. @default undefined */
  readonly snapshot?: DevtoolsSnapshot;
  /** Explicit visitor hosts; omission keeps the recorder plugin-free. @default undefined */
  readonly visitorHosts?: readonly DevtoolsTraceVisitorHost[];
}

/** Request-local recorder API for events, reset, and immutable snapshots. */
export interface DevtoolsRecorder {
  readonly record: <Event extends DevtoolsEvent>(event: Event) => Event;
  readonly clear: () => void;
  readonly snapshot: () => DevtoolsSnapshot;
}

/** Generator-facing manifest of the package devtools panels. */
export interface DevtoolsManifest {
  readonly schemaVersion: 1;
  readonly panels: readonly DevtoolsManifestPanel[];
}

/** One source-owned devtools panel and the trace surfaces it consumes. */
export interface DevtoolsManifestPanel {
  readonly id: string;
  readonly title: string;
  readonly source: DevtoolsSourceSpecifier;
  readonly surfaces: readonly ("render" | "reactivity" | "provide" | "suspense")[];
}
