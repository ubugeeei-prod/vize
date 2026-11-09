import type { FrescoInputEvent } from "./protocol.js";
import type { FrescoHostNode, FrescoInputDispatchResult, createRenderer } from "./renderer.js";

/** Renderer surface needed to route terminal input to the active focus target. */
export type FocusRoutingRenderer = Pick<
  ReturnType<typeof createRenderer>,
  "dispatchInput" | "findNodeByFocusId"
>;

export type DefaultInputAction = "focus-next" | "focus-previous" | "exit" | undefined;

interface DefaultInputPacket {
  eventType: string;
  key?: string;
  char?: string;
  modifiers?: { ctrl?: boolean };
}

/** Find the live host node registered for one logical focus target. */
export function findHostNodeByFocusId(
  root: FrescoHostNode,
  focusId: string,
): FrescoHostNode | undefined {
  if (root.props.focusId === focusId) return root;
  for (const child of root.children) {
    const match = findHostNodeByFocusId(child, focusId);
    if (match) return match;
  }
  return undefined;
}

/** Derive the application fallback after focused handlers have had first refusal. */
export function defaultInputAction(
  input: DefaultInputPacket,
  result: Pick<FrescoInputDispatchResult, "defaultPrevented"> | undefined,
  exitOnCtrlC: boolean,
): DefaultInputAction {
  if (result?.defaultPrevented || input.eventType !== "key") return undefined;
  if (input.key === "tab") return "focus-next";
  if (input.key === "backtab") return "focus-previous";
  if (exitOnCtrlC && input.char === "c" && input.modifiers?.ctrl) return "exit";
  return undefined;
}

/**
 * Route one normalized terminal packet to the live host node associated with
 * the active logical focus ID.
 */
export function routeFocusedInput(
  renderer: FocusRoutingRenderer,
  root: FrescoHostNode | null,
  focusId: string | undefined,
  input: FrescoInputEvent,
): FrescoInputDispatchResult | undefined {
  if (!root || focusId === undefined) return undefined;
  const target = renderer.findNodeByFocusId(root, focusId);
  return target ? renderer.dispatchInput(target.id, input) : undefined;
}
