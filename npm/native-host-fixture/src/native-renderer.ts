import { createRenderer, type Component } from "vue";
import {
  NATIVE_HOST_MAXIMUM_DEPTH,
  NATIVE_HOST_MAXIMUM_NODES,
  NATIVE_HOST_MAXIMUM_SPACING,
  NATIVE_HOST_MAXIMUM_TEXT,
  NativeHostFailure,
  type NativeHostBridge,
  type NativeViewNode,
} from "./host-contract.ts";

type HostKind = "view" | "text" | "button" | "literal" | "anchor";
interface HostNode {
  readonly id: number;
  readonly kind: HostKind;
  parent: HostNode | null;
  readonly children: HostNode[];
  readonly props: Record<string, unknown>;
  text: string;
}

/** Engine controls retained outside canonical SFC application sources. */
export interface NativeHostRuntime {
  /** Mount one compiler-produced SFC component into the native root. */
  readonly mount: (component: Component) => void;
  /** Deliver an activation only when it belongs to the current installed snapshot. */
  readonly dispatch: (event: unknown) => boolean;
}

/**
 * Creates the first host's private custom-renderer adapter.
 *
 * Component sources remain SFCs. Reconciliation anchors remain private JS nodes;
 * only native view/text/button semantics cross the bridge. Every tree update is
 * authoritative and bounded before it is delivered. Stale or malformed native
 * events cannot invoke an application callback from another installed snapshot.
 */
export function createNativeHostRuntime(bridge: NativeHostBridge): NativeHostRuntime {
  let nextId = 1;
  let sequence = 0;
  let pending = false;
  let failed = false;
  let mounted = false;
  let handlers = new Map<number, () => void>();
  const node = (kind: HostKind, text = ""): HostNode => ({
    id: nextId++,
    kind,
    text,
    parent: null,
    children: [],
    props: {},
  });
  const root = node("view");

  function refuse(error: unknown): void {
    if (failed) return;
    failed = true;
    handlers.clear();
    bridge.postMessage({
      format: "vize.native-diagnostic",
      formatVersion: 1,
      code: error instanceof NativeHostFailure ? error.code : "application-render-failed",
      message:
        error instanceof NativeHostFailure ? error.message : "The application could not render.",
    });
  }

  function schedule(): void {
    if (pending || failed) return;
    pending = true;
    queueMicrotask(() => {
      pending = false;
      if (!mounted || failed) return;
      try {
        const { nodes, handlers: nextHandlers } = captureTree(root);
        if (!Number.isSafeInteger(++sequence)) {
          throw new NativeHostFailure(
            "native-sequence-exhausted",
            "The native update sequence is exhausted.",
          );
        }
        handlers = nextHandlers;
        bridge.postMessage({
          format: "vize.native-view-tree",
          formatVersion: 1,
          sequence,
          root: root.id,
          nodes,
        });
      } catch (error) {
        refuse(error);
      }
    });
  }

  const renderer = createHostRenderer(node, schedule);

  return {
    mount(component) {
      if (mounted)
        throw new NativeHostFailure(
          "native-application-already-mounted",
          "The native application is already mounted.",
        );
      const app = renderer.createApp(component);
      app.config.errorHandler = refuse;
      try {
        app.mount(root);
        mounted = true;
        schedule();
      } catch (error) {
        refuse(error);
      }
    },
    dispatch(event) {
      if (!event || typeof event !== "object" || failed) return false;
      const value = event as Record<string, unknown>;
      if (
        Object.keys(value).sort().join(",") !== "event,node,sequence" ||
        value.event !== "press" ||
        value.sequence !== sequence ||
        !Number.isSafeInteger(value.node)
      )
        return false;
      const callback = handlers.get(value.node as number);
      if (!callback) return false;
      try {
        callback();
        return true;
      } catch (error) {
        refuse(error);
        return false;
      }
    },
  };
}

function createHostRenderer(
  node: (kind: HostKind, text?: string) => HostNode,
  schedule: () => void,
) {
  return createRenderer<HostNode, HostNode>({
    createElement(tag) {
      const kind =
        tag === "native-view"
          ? "view"
          : tag === "native-text"
            ? "text"
            : tag === "native-button"
              ? "button"
              : undefined;
      if (kind === undefined)
        throw new NativeHostFailure(
          "unsupported-native-primitive",
          "The first host does not implement this native primitive.",
        );
      return node(kind);
    },
    createText: (text) => node("literal", text),
    createComment: () => node("anchor"),
    setText(target, text) {
      target.text = text;
      schedule();
    },
    setElementText(target, text) {
      for (const child of target.children) child.parent = null;
      target.children.length = 0;
      if (text !== "") {
        const child = node("literal", text);
        child.parent = target;
        target.children.push(child);
      }
      schedule();
    },
    parentNode: (target) => target.parent,
    nextSibling(target) {
      const siblings = target.parent?.children;
      return siblings?.[siblings.indexOf(target) + 1] ?? null;
    },
    insert(target, parent, anchor = null) {
      if (target.parent) {
        const siblings = target.parent.children;
        siblings.splice(siblings.indexOf(target), 1);
      }
      const index = anchor === null ? parent.children.length : parent.children.indexOf(anchor);
      if (index < 0)
        throw new NativeHostFailure(
          "invalid-native-anchor",
          "A native reconciliation anchor is not owned by its parent.",
        );
      parent.children.splice(index, 0, target);
      target.parent = parent;
      schedule();
    },
    remove(target) {
      if (target.parent) {
        const siblings = target.parent.children;
        siblings.splice(siblings.indexOf(target), 1);
      }
      target.parent = null;
      schedule();
    },
    patchProp(target, key, _previous, value) {
      const supported =
        target.kind === "view"
          ? ["gap", "padding"]
          : target.kind === "text"
            ? ["text"]
            : ["label", "onPress"];
      if (!supported.includes(key))
        throw new NativeHostFailure(
          "unsupported-native-property",
          "The first host does not implement this native property.",
        );
      if (value === null || value === undefined) delete target.props[key];
      else target.props[key] = value;
      schedule();
    },
    setScopeId() {
      // The build refuses non-empty styles; empty authored style sections need no native rule.
    },
  });
}

function captureTree(root: HostNode) {
  const nodes: NativeViewNode[] = [];
  const handlers = new Map<number, () => void>();
  function visit(current: HostNode, depth: number): void {
    if (!physical(current)) return;
    if (depth > NATIVE_HOST_MAXIMUM_DEPTH || nodes.length >= NATIVE_HOST_MAXIMUM_NODES) {
      throw new NativeHostFailure(
        "native-tree-budget-exceeded",
        "The native view tree exceeds the host limits.",
      );
    }
    const children = current.children.filter(physical);
    nodes.push(
      serialize(
        current,
        children.map(({ id }) => id),
      ),
    );
    const press = current.props.onPress;
    if (current.kind === "button" && press !== undefined) {
      if (typeof press !== "function") {
        throw new NativeHostFailure(
          "invalid-native-handler",
          "Native activation requires a callable handler.",
        );
      }
      handlers.set(current.id, () => press());
    }
    for (const child of children) visit(child, depth + 1);
  }
  visit(root, 1);
  return { nodes, handlers };
}

function physical(node: HostNode): boolean {
  return node.kind !== "anchor" && (node.kind !== "literal" || node.text !== "");
}

function spacing(value: unknown): number {
  if (value === undefined) return 0;
  if (
    typeof value !== "number" ||
    !Number.isFinite(value) ||
    value < 0 ||
    value > NATIVE_HOST_MAXIMUM_SPACING
  ) {
    throw new NativeHostFailure(
      "invalid-native-layout",
      "Native gap and padding must be bounded nonnegative point values.",
    );
  }
  return value;
}

function label(value: unknown, nonempty: boolean): string {
  if (
    typeof value !== "string" ||
    value.length > NATIVE_HOST_MAXIMUM_TEXT ||
    (nonempty && value.trim() === "")
  ) {
    throw new NativeHostFailure(
      "invalid-native-text",
      "Native text must fit the host limits and button labels must name their action.",
    );
  }
  for (const character of value) {
    const point = character.codePointAt(0)!;
    if (point >= 0xd800 && point <= 0xdfff) {
      throw new NativeHostFailure(
        "invalid-native-text",
        "Native text must contain well-formed Unicode.",
      );
    }
  }
  return value;
}

function serialize(node: HostNode, children: readonly number[]): NativeViewNode {
  if (!Number.isSafeInteger(node.id) || node.id < 1)
    throw new NativeHostFailure("invalid-native-node-id", "A native node identifier is invalid.");
  if (node.kind === "view")
    return {
      id: node.id,
      kind: "view",
      children,
      gap: spacing(node.props.gap),
      padding: spacing(node.props.padding),
    };
  if (children.length !== 0)
    throw new NativeHostFailure(
      "invalid-native-children",
      "Only native view primitives can own child views.",
    );
  if (node.kind === "button")
    return { id: node.id, kind: "button", children, label: label(node.props.label, true) };
  return {
    id: node.id,
    kind: "text",
    children,
    text: label(node.kind === "literal" ? node.text : node.props.text, false),
  };
}
