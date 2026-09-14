/** Source-owned Vue file that a trace event or panel belongs to. */
export type DevtoolsSourceSpecifier = `${string}.vue`;

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

export interface DevtoolsComponentIdentity {
  readonly componentId: string;
  readonly componentName: string;
  readonly source: DevtoolsSourceSpecifier;
}

export type DevtoolsComponentEvent =
  | (DevtoolsEventBase<"render:queued" | "render:start"> & DevtoolsComponentIdentity)
  | (DevtoolsEventBase<"render:end"> & DevtoolsComponentIdentity & { readonly durationMs: number })
  | (DevtoolsEventBase<"update:scheduled" | "update:committed"> &
      DevtoolsComponentIdentity & { readonly reason: string });

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

export type DevtoolsEvent =
  | DevtoolsComponentEvent
  | DevtoolsReactiveEvent
  | DevtoolsProvideEvent
  | DevtoolsSuspenseEvent;

export type SerializedDevtoolsEvent = DevtoolsEvent & {
  readonly sequence: number;
  readonly timestamp: number;
};

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

export interface DevtoolsReactiveNode {
  readonly id: string;
  readonly label: string;
  readonly kind: "effect" | "source";
}

export interface DevtoolsReactiveEdge {
  readonly sourceId: string;
  readonly targetId: string;
  readonly operation: "track" | "trigger";
  readonly count: number;
}

export interface DevtoolsProvideTreeNode {
  readonly providerId: string;
  readonly ownerComponentId: string;
  readonly ownerComponentName: string;
  readonly providedKeys: readonly string[];
  readonly consumers: readonly DevtoolsProvideConsumer[];
}

export interface DevtoolsProvideConsumer {
  readonly injectorId: string;
  readonly injectorComponentId: string;
  readonly injectorComponentName: string;
  readonly key: string;
  readonly status: "resolved" | "shadowed" | "unresolved";
}

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

export interface DevtoolsRecorderOptions {
  readonly enabled?: boolean;
  readonly maxEvents?: number;
  readonly now?: () => number;
  readonly sessionId?: string;
  readonly snapshot?: DevtoolsSnapshot;
}

export interface DevtoolsRecorder {
  readonly record: <Event extends DevtoolsEvent>(event: Event) => Event;
  readonly clear: () => void;
  readonly snapshot: () => DevtoolsSnapshot;
}

export interface DevtoolsManifest {
  readonly schemaVersion: 1;
  readonly panels: readonly DevtoolsManifestPanel[];
}

export interface DevtoolsManifestPanel {
  readonly id: string;
  readonly title: string;
  readonly source: DevtoolsSourceSpecifier;
  readonly surfaces: readonly ("render" | "reactivity" | "provide" | "suspense")[];
}
