import type { LayoutResultNapi } from "@vizejs/fresco-native";
import type { FrescoInputEvent } from "./protocol.js";
import type {
  FrescoDeclaredSize,
  FrescoHostNode,
  FrescoNodeLifecycleHooks,
  FrescoNodeUpdateKind,
  FrescoRenderNodeKind,
} from "./renderer.js";

/** One immutable native layout rectangle joined to its host node. */
export interface FrescoLayoutInspection {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/** Flat, immutable host-node record suitable for tree inspectors. */
export interface FrescoNodeInspection {
  readonly id: number;
  readonly type: FrescoRenderNodeKind;
  readonly parentId: number | null;
  readonly childIds: readonly number[];
  readonly depth: number;
  readonly props: Readonly<Record<string, unknown>>;
  readonly text?: string;
  readonly layout: FrescoLayoutInspection | null;
}

/** One inspected Fresco host tree and the native layouts joined to it. */
export interface FrescoTreeInspection {
  readonly rootId: number;
  readonly nodes: readonly FrescoNodeInspection[];
  readonly nodeCount: number;
  readonly layoutCount: number;
  /** Layout ids that did not belong to the inspected host tree. */
  readonly unmatchedLayoutIds: readonly number[];
}

/** Metrics recorded after one application frame. */
export interface FrescoDevtoolsRenderMetrics {
  readonly renderTime: number;
  readonly nodeCount: number;
  readonly layoutCount: number;
}

interface FrescoDevtoolsEventBase {
  /** Monotonic event order local to this devtools instance. */
  readonly sequence: number;
  readonly nodeId?: number;
}

/** Bounded, discriminated event records emitted by Fresco devtools. */
export type FrescoDevtoolsEvent =
  | (FrescoDevtoolsEventBase & {
      readonly type: "create";
      readonly nodeId: number;
      readonly nodeType: FrescoRenderNodeKind;
    })
  | (FrescoDevtoolsEventBase & {
      readonly type: "update";
      readonly nodeId: number;
      readonly updateKind: FrescoNodeUpdateKind;
    })
  | (FrescoDevtoolsEventBase & {
      readonly type: "resize";
      readonly nodeId: number;
      readonly previous: FrescoDeclaredSize;
      readonly current: FrescoDeclaredSize;
    })
  | (FrescoDevtoolsEventBase & {
      readonly type: "remove";
      readonly nodeId: number;
      readonly nodeType: FrescoRenderNodeKind;
    })
  | (FrescoDevtoolsEventBase & {
      readonly type: "input";
      readonly input: FrescoInputEvent;
    })
  | (FrescoDevtoolsEventBase & {
      readonly type: "render";
      readonly metrics: FrescoDevtoolsRenderMetrics;
    });

type WithoutSequence<Event> = Event extends unknown ? Omit<Event, "sequence"> : never;
type FrescoDevtoolsEventInput = WithoutSequence<FrescoDevtoolsEvent>;

/** Point-in-time state returned by {@link FrescoDevtools.snapshot}. */
export interface FrescoDevtoolsSnapshot {
  readonly tree: FrescoTreeInspection | null;
  readonly events: readonly FrescoDevtoolsEvent[];
  readonly latestRender: FrescoDevtoolsRenderMetrics | null;
}

/** Configuration for {@link createFrescoDevtools}. */
export interface FrescoDevtoolsOptions {
  /** Maximum retained trace events. Older records are discarded first. @default 200 */
  eventLimit?: number;
}

/** Inspector and bounded trace collector accepted by `createApp`. */
export interface FrescoDevtools {
  /** Lifecycle hooks wired into the public Fresco renderer. */
  readonly lifecycle: FrescoNodeLifecycleHooks;
  /** Record one normalized terminal input event. */
  recordInput(input: FrescoInputEvent): void;
  /** Join a rendered host tree with its native layouts and render metrics. */
  recordRender(
    root: FrescoHostNode,
    layouts: readonly LayoutResultNapi[],
    metrics: FrescoDevtoolsRenderMetrics,
  ): FrescoTreeInspection;
  /** Read an immutable copy of the current inspector state and bounded trace. */
  snapshot(): FrescoDevtoolsSnapshot;
  /** Remove trace records without discarding the latest tree or metrics. */
  clearEvents(): void;
}

/** Options for the terminal-friendly debug overlay formatter. */
export interface FrescoDebugOverlayOptions {
  /** Maximum tree rows included after the summary. @default 20 */
  maxNodes?: number;
  /** Maximum recent trace rows included after the tree. @default 8 */
  maxEvents?: number;
}

function copyLayout(layout: LayoutResultNapi): FrescoLayoutInspection {
  return {
    x: layout.x,
    y: layout.y,
    width: layout.width,
    height: layout.height,
  };
}

/**
 * Join a live host tree with the layout results produced by the same native frame.
 *
 * The returned records are flat and immutable, so devtools can index them without
 * retaining parent back-references or the mutable Vue renderer tree.
 */
export function inspectFrescoTree(
  root: FrescoHostNode,
  layouts: readonly LayoutResultNapi[] = [],
): FrescoTreeInspection {
  const layoutsById = new Map(layouts.map((layout) => [layout.id, layout]));
  const matchedLayoutIds = new Set<number>();
  const nodes: FrescoNodeInspection[] = [];

  const visit = (node: FrescoHostNode, depth: number) => {
    const layout = layoutsById.get(node.id);
    if (layout) matchedLayoutIds.add(node.id);
    nodes.push({
      id: node.id,
      type: node.type,
      parentId: node.parent?.id ?? null,
      childIds: node.children.map((child) => child.id),
      depth,
      props: { ...node.props },
      ...(node.text === undefined ? {} : { text: node.text }),
      layout: layout ? copyLayout(layout) : null,
    });
    for (const child of node.children) visit(child, depth + 1);
  };

  visit(root, 0);
  return {
    rootId: root.id,
    nodes,
    nodeCount: nodes.length,
    layoutCount: matchedLayoutIds.size,
    unmatchedLayoutIds: layouts
      .map((layout) => layout.id)
      .filter((id) => !matchedLayoutIds.has(id)),
  };
}

function retainedLimit(limit: number | undefined): number {
  if (limit === undefined) return 200;
  if (!Number.isFinite(limit)) return 200;
  return Math.max(0, Math.floor(limit));
}

/** Create one opt-in tree inspector and bounded event trace collector. */
export function createFrescoDevtools(options: FrescoDevtoolsOptions = {}): FrescoDevtools {
  const eventLimit = retainedLimit(options.eventLimit);
  const events: FrescoDevtoolsEvent[] = [];
  let sequence = 0;
  let tree: FrescoTreeInspection | null = null;
  let latestRender: FrescoDevtoolsRenderMetrics | null = null;

  const append = (event: FrescoDevtoolsEventInput) => {
    if (eventLimit === 0) return;
    events.push({ ...event, sequence: sequence++ } as FrescoDevtoolsEvent);
    const overflow = events.length - eventLimit;
    if (overflow > 0) events.splice(0, overflow);
  };

  const lifecycle: FrescoNodeLifecycleHooks = {
    onCreate(node) {
      append({ type: "create", nodeId: node.id, nodeType: node.type });
    },
    onUpdate(node, updateKind) {
      append({ type: "update", nodeId: node.id, updateKind });
    },
    onResize(node, previous, current) {
      append({ type: "resize", nodeId: node.id, previous, current });
    },
    onRemove(node) {
      append({ type: "remove", nodeId: node.id, nodeType: node.type });
    },
  };

  return {
    lifecycle,
    recordInput(input) {
      append({ type: "input", input: { ...input } as FrescoInputEvent });
    },
    recordRender(root, layouts, metrics) {
      tree = inspectFrescoTree(root, layouts);
      latestRender = { ...metrics };
      append({ type: "render", metrics: latestRender });
      return tree;
    },
    snapshot() {
      return {
        tree,
        events: events.map((event) => ({ ...event })),
        latestRender,
      };
    },
    clearEvents() {
      events.length = 0;
    },
  };
}

function formatNode(node: FrescoNodeInspection): string {
  const indent = "  ".repeat(node.depth);
  const layout = node.layout
    ? ` ${node.layout.width}x${node.layout.height}@${node.layout.x},${node.layout.y}`
    : " layout:pending";
  const text = node.text === undefined ? "" : ` ${JSON.stringify(node.text)}`;
  return `${indent}#${node.id} ${node.type}${layout}${text}`;
}

function formatEvent(event: FrescoDevtoolsEvent): string {
  switch (event.type) {
    case "create":
      return `${event.sequence} create #${event.nodeId} ${event.nodeType}`;
    case "update":
      return `${event.sequence} update #${event.nodeId} ${event.updateKind}`;
    case "resize":
      return `${event.sequence} resize #${event.nodeId}`;
    case "remove":
      return `${event.sequence} remove #${event.nodeId} ${event.nodeType}`;
    case "input":
      return `${event.sequence} input ${event.input.type}`;
    case "render":
      return `${event.sequence} render ${event.metrics.renderTime.toFixed(2)}ms`;
  }
}

/** Render an inspector snapshot as a compact terminal debug overlay. */
export function formatFrescoDebugOverlay(
  snapshot: FrescoDevtoolsSnapshot,
  options: FrescoDebugOverlayOptions = {},
): string {
  const maxNodes = Math.max(0, Math.floor(options.maxNodes ?? 20));
  const maxEvents = Math.max(0, Math.floor(options.maxEvents ?? 8));
  const render = snapshot.latestRender;
  const summary = render
    ? `Fresco ${render.nodeCount} nodes · ${render.layoutCount} layouts · ${render.renderTime.toFixed(2)}ms`
    : "Fresco devtools · waiting for first render";
  const treeRows = snapshot.tree?.nodes.slice(0, maxNodes).map(formatNode) ?? [];
  const eventRows = snapshot.events.slice(-maxEvents).map(formatEvent);
  return [summary, ...treeRows, ...(eventRows.length > 0 ? ["events", ...eventRows] : [])].join(
    "\n",
  );
}
