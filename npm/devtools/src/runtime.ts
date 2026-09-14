import type {
  DevtoolsEvent,
  DevtoolsManifest,
  DevtoolsManifestPanel,
  DevtoolsProvideConsumer,
  DevtoolsProvideTreeNode,
  DevtoolsReactiveEdge,
  DevtoolsReactiveNode,
  DevtoolsRecorder,
  DevtoolsRecorderOptions,
  DevtoolsRenderNode,
  DevtoolsSnapshot,
  DevtoolsSourceSpecifier,
  DevtoolsSuspenseEvent,
  DevtoolsSuspenseNode,
  SerializedDevtoolsEvent,
} from "./types.ts";

const DEVTOOLS_ERROR = {
  invalidPanel: "VIZE_DEVTOOLS_PANEL_NOT_VUE",
  invalidSource: "VIZE_DEVTOOLS_SOURCE_NOT_VUE",
} as const;

/** Create a request-local devtools recorder with no module-global event state. */
export function createDevtoolsTraceRecorder(
  options: DevtoolsRecorderOptions = {},
): DevtoolsRecorder {
  const enabled = options.enabled ?? true;
  const maxEvents = options.maxEvents ?? 2_000;
  const now = options.now ?? Date.now;
  const sessionId = options.sessionId ?? options.snapshot?.sessionId ?? "vize-devtools";
  let sequence = options.snapshot?.events.at(-1)?.sequence ?? 0;
  let events = [...(options.snapshot?.events ?? [])];

  return {
    record(event) {
      validateEventSource(event);
      if (!enabled) return event;
      sequence += 1;
      events.push({ ...event, sequence, timestamp: event.timestamp ?? now() });
      if (events.length > maxEvents) events = events.slice(events.length - maxEvents);
      return event;
    },
    clear() {
      events = [];
    },
    snapshot() {
      const recorded = [...events];
      return {
        schemaVersion: 1,
        enabled,
        sessionId,
        createdAt: now(),
        events: recorded,
        renderTree: collectRenderTree(recorded),
        reactiveGraph: collectReactiveGraph(recorded),
        provideTree: collectProvideTree(recorded),
        suspenseTree: collectSuspenseTree(recorded),
      };
    },
  };
}

/** Return a serializable trace snapshot for SSR payloads and trace artifacts. */
export function serializeDevtoolsSnapshot(recorder: Pick<DevtoolsRecorder, "snapshot">) {
  return recorder.snapshot();
}

/** Hydrate a recorder from a serialized server payload without sharing state globally. */
export function hydrateDevtoolsSnapshot(
  snapshot: DevtoolsSnapshot,
  options: Omit<DevtoolsRecorderOptions, "snapshot"> = {},
) {
  return createDevtoolsTraceRecorder({ ...options, snapshot });
}

/** Emit generator metadata for host devtools integrations. */
export function createDevtoolsManifest(
  panels: readonly DevtoolsManifestPanel[] = [
    {
      id: "trace",
      title: "Trace",
      source: "src/panel/DevtoolsTracePanel.vue",
      surfaces: ["render", "reactivity", "provide", "suspense"],
    },
  ],
): DevtoolsManifest {
  for (const panel of panels) validateVueSource(panel.source, DEVTOOLS_ERROR.invalidPanel);
  return { schemaVersion: 1, panels };
}

/** Stream deterministic trace records for reports, tests, or remote devtools transports. */
export async function* streamDevtoolsTrace(
  snapshot: DevtoolsSnapshot,
): AsyncGenerator<SerializedDevtoolsEvent, void, void> {
  for (const event of snapshot.events) yield event;
}

function collectRenderTree(events: readonly SerializedDevtoolsEvent[]): DevtoolsRenderNode[] {
  const nodes = new Map<string, DevtoolsRenderNode>();
  for (const event of events) {
    if (!("componentId" in event) || !("componentName" in event) || !("source" in event)) continue;
    const current = nodes.get(event.componentId);
    nodes.set(event.componentId, {
      componentId: event.componentId,
      componentName: event.componentName,
      source: event.source,
      renderCount: (current?.renderCount ?? 0) + (event.kind === "render:end" ? 1 : 0),
      updateCount: (current?.updateCount ?? 0) + (event.kind.startsWith("update:") ? 1 : 0),
      lastDurationMs:
        event.kind === "render:end" ? event.durationMs : (current?.lastDurationMs ?? null),
      lastEventKind: event.kind,
      lastTimestamp: event.timestamp,
    });
  }
  return [...nodes.values()].sort((left, right) =>
    left.componentName.localeCompare(right.componentName),
  );
}

function collectReactiveGraph(events: readonly SerializedDevtoolsEvent[]) {
  const nodes = new Map<string, DevtoolsReactiveNode>();
  const edges = new Map<string, DevtoolsReactiveEdge>();

  for (const event of events) {
    if (event.kind === "reactivity:track") {
      nodes.set(event.sourceId, { id: event.sourceId, label: event.sourceLabel, kind: "source" });
      nodes.set(event.effectId, { id: event.effectId, label: event.effectLabel, kind: "effect" });
      incrementEdge(edges, event.sourceId, event.effectId, "track");
    } else if (event.kind === "reactivity:trigger") {
      if (!nodes.has(event.sourceId)) {
        nodes.set(event.sourceId, { id: event.sourceId, label: event.sourceId, kind: "source" });
      }
      if (!nodes.has(event.effectId)) {
        nodes.set(event.effectId, { id: event.effectId, label: event.effectId, kind: "effect" });
      }
      incrementEdge(edges, event.sourceId, event.effectId, "trigger");
    }
  }

  return {
    nodes: [...nodes.values()].sort((left, right) => left.id.localeCompare(right.id)),
    edges: [...edges.values()].sort((left, right) =>
      `${left.sourceId}:${left.targetId}`.localeCompare(`${right.sourceId}:${right.targetId}`),
    ),
  };
}

function collectProvideTree(events: readonly SerializedDevtoolsEvent[]): DevtoolsProvideTreeNode[] {
  const providers = new Map<string, MutableProvider>();

  for (const event of events) {
    if (event.kind === "provide:set") {
      const provider = providers.get(event.providerId) ?? {
        providerId: event.providerId,
        ownerComponentId: event.ownerComponentId,
        ownerComponentName: event.ownerComponentName,
        providedKeys: new Set<string>(),
        consumers: [],
      };
      provider.providedKeys.add(event.key);
      providers.set(event.providerId, provider);
    } else if (event.kind === "inject:resolve") {
      const providerId = event.providerId ?? `unresolved:${event.injectorId}:${event.key}`;
      const provider = providers.get(providerId) ?? {
        providerId,
        ownerComponentId: event.providerId ?? "unresolved",
        ownerComponentName: event.providerId == null ? "Unresolved" : event.providerId,
        providedKeys: new Set<string>(),
        consumers: [],
      };
      provider.consumers.push({
        injectorId: event.injectorId,
        injectorComponentId: event.injectorComponentId,
        injectorComponentName: event.injectorComponentName,
        key: event.key,
        status: event.status,
      });
      providers.set(providerId, provider);
    }
  }

  return [...providers.values()]
    .map((provider) => ({
      providerId: provider.providerId,
      ownerComponentId: provider.ownerComponentId,
      ownerComponentName: provider.ownerComponentName,
      providedKeys: [...provider.providedKeys].sort(),
      consumers: [...provider.consumers].sort((left, right) =>
        left.injectorId.localeCompare(right.injectorId),
      ),
    }))
    .sort((left, right) => left.providerId.localeCompare(right.providerId));
}

function collectSuspenseTree(events: readonly SerializedDevtoolsEvent[]): DevtoolsSuspenseNode[] {
  const nodes = new Map<string, DevtoolsSuspenseNode>();
  for (const event of events) {
    if (!isSuspenseEvent(event)) continue;
    nodes.set(event.boundaryId, {
      boundaryId: event.boundaryId,
      componentId: event.componentId,
      componentName: event.componentName,
      parentBoundaryId: event.parentBoundaryId ?? null,
      state:
        event.kind === "suspense:error"
          ? "error"
          : event.kind === "suspense:resolve"
            ? "resolved"
            : "pending",
      asyncDependency: "asyncDependency" in event ? (event.asyncDependency ?? null) : null,
      error: "error" in event ? event.error : null,
      updatedAt: event.timestamp,
    });
  }
  return [...nodes.values()].sort((left, right) => left.boundaryId.localeCompare(right.boundaryId));
}

function isSuspenseEvent(
  event: SerializedDevtoolsEvent,
): event is SerializedDevtoolsEvent & DevtoolsSuspenseEvent {
  return (
    event.kind === "suspense:pending" ||
    event.kind === "suspense:resolve" ||
    event.kind === "suspense:error"
  );
}

function incrementEdge(
  edges: Map<string, DevtoolsReactiveEdge>,
  sourceId: string,
  targetId: string,
  operation: DevtoolsReactiveEdge["operation"],
) {
  const key = `${sourceId}:${targetId}:${operation}`;
  const current = edges.get(key);
  edges.set(key, { sourceId, targetId, operation, count: (current?.count ?? 0) + 1 });
}

function validateEventSource(event: DevtoolsEvent): void {
  if ("source" in event && event.source != null) {
    validateVueSource(event.source, DEVTOOLS_ERROR.invalidSource);
  }
}

function validateVueSource(source: DevtoolsSourceSpecifier, code: string): void {
  if (!source.endsWith(".vue")) {
    throw new Error(`[${code}] Devtools sources must be authored as .vue files`);
  }
}

type MutableProvider = {
  readonly providerId: string;
  readonly ownerComponentId: string;
  readonly ownerComponentName: string;
  readonly providedKeys: Set<string>;
  readonly consumers: DevtoolsProvideConsumer[];
};
