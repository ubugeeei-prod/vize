import { toValue } from "vue";

import type { DragAndDropOptions } from "./drag-and-drop-controller-types.ts";
import { createDragEvent, readBoolean } from "./drag-and-drop-internal.ts";
import type { SessionCore } from "./drag-and-drop-session.ts";
import { capture, surfaceErrors } from "./move-internal.ts";
import { eventElement } from "./press-event.ts";

type KeyboardAction = "drop" | "first" | "last" | "next" | "previous";

function keyboardStep<Data>(
  core: SessionCore<Data>,
  options: DragAndDropOptions<Data>,
  event: KeyboardEvent,
  action: KeyboardAction,
): void {
  const sess = core.getSession();
  if (!sess) return;
  if (action === "drop") {
    core.finishSession(event, false);
    return;
  }
  const errors: unknown[] = [];
  const ordered = core.orderedTargets(sess.payload);
  if (ordered.length === 0) {
    core.updateOver(sess, null, null, event, errors);
  } else {
    const currentIndex = ordered.findIndex((candidate) => candidate.key === sess.targetKey);
    let nextIndex = 0;
    if (action === "last") nextIndex = ordered.length - 1;
    else if (action === "next") nextIndex = (currentIndex + 1) % ordered.length;
    else if (action === "previous") {
      nextIndex = currentIndex <= 0 ? ordered.length - 1 : currentIndex - 1;
    }
    const next = ordered[nextIndex];
    if (next) {
      const edges = next.record.readEdges();
      const edge = edges.includes("inside") ? "inside" : (edges[0] ?? "inside");
      core.updateOver(sess, next, edge, event, errors);
      const move = createDragEvent(
        "dragmove",
        sess.pointerType,
        sess.sourceKey,
        sess.payload,
        sess.targetKey,
        sess.edge,
        null,
        event,
      );
      capture(errors, () => options.onDragMove?.(move));
    }
  }
  surfaceErrors(errors, "Drag callbacks failed");
}

function keyboardGrab<Data>(
  core: SessionCore<Data>,
  options: DragAndDropOptions<Data>,
  event: KeyboardEvent,
  key: string,
): void {
  const record = core.sources.get(key);
  if (!record || event.altKey || event.ctrlKey || event.metaKey || event.isComposing) return;
  if (
    readBoolean(options.isDisabled, "isDisabled") ||
    readBoolean(record.options.isDisabled, "isDisabled") ||
    !readBoolean(record.options.keyboard, "keyboard", true)
  ) {
    return;
  }
  const host = eventElement(event);
  if (!host || event.target !== host) return;
  const payload = toValue(record.options.payload) ?? null;
  if (core.orderedTargets(payload).length === 0) return;
  event.preventDefault();
  event.stopPropagation();
  const errors: unknown[] = [];
  const sess = core.beginSession("keyboard", key, event, null, errors);
  keyboardStep(core, options, event, "first");
  if (core.getSession() === sess) {
    const grab = core.buildMessage("grab", sess);
    const move = core.buildMessage("move", sess);
    const combined = [grab, move].filter((part) => part !== null).join(" ");
    if (combined) capture(errors, () => core.announceMessage(sess, combined));
  }
  surfaceErrors(errors, "Drag callbacks failed");
}

/** Handle one keydown from a source handle: grab, navigate, drop, or cancel. */
export function handleSourceKeydown<Data>(
  core: SessionCore<Data>,
  options: DragAndDropOptions<Data>,
  event: KeyboardEvent,
  key: string,
): void {
  const session = core.getSession();
  if (session?.pointerType === "keyboard" && session.sourceKey === key) {
    switch (event.key) {
      case "ArrowDown":
      case "ArrowRight":
        keyboardStep(core, options, event, "next");
        break;
      case "ArrowUp":
      case "ArrowLeft":
        keyboardStep(core, options, event, "previous");
        break;
      case "Home":
        keyboardStep(core, options, event, "first");
        break;
      case "End":
        keyboardStep(core, options, event, "last");
        break;
      case "Enter":
      case " ":
        keyboardStep(core, options, event, "drop");
        break;
      case "Escape":
        core.finishSession(event, true);
        break;
      case "Tab":
        core.finishSession(event, true);
        return;
      default:
        return;
    }
    event.preventDefault();
    event.stopPropagation();
  } else if (!session && !core.getContact() && (event.key === "Enter" || event.key === " ")) {
    keyboardGrab(core, options, event, key);
  }
}
