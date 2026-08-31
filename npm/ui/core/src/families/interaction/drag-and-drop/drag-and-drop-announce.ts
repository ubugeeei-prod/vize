import type { DragAnnouncementContext, DragAnnouncements } from "./drag-and-drop-types.ts";

const visuallyHiddenStyles: readonly (readonly [string, string])[] = [
  ["border", "0"],
  ["clip-path", "inset(50%)"],
  ["height", "1px"],
  ["margin", "-1px"],
  ["overflow", "hidden"],
  ["padding", "0"],
  ["position", "fixed"],
  ["white-space", "nowrap"],
  ["width", "1px"],
];

/** Owned assertive live region created lazily so SSR never touches the DOM. */
export interface DragLiveRegion {
  /** Speak one message; repeated messages are re-announced. */
  readonly announce: (document: Document, message: string) => void;

  /** Remove the live region element when it was created. */
  readonly dispose: () => void;
}

/** Create a lazily mounted `role="status"` live region for drag announcements. */
export function createDragLiveRegion(): DragLiveRegion {
  let element: HTMLElement | null = null;
  return Object.freeze({
    announce(document: Document, message: string) {
      if (!element || !element.isConnected || element.ownerDocument !== document) {
        element?.remove();
        element = document.createElement("div");
        element.setAttribute("role", "status");
        element.setAttribute("aria-live", "assertive");
        element.setAttribute("aria-atomic", "true");
        element.setAttribute("data-vize-ui", "drag-and-drop-live");
        for (const [property, value] of visuallyHiddenStyles) {
          element.style.setProperty(property, value);
        }
        document.body.append(element);
      }
      // Alternate a trailing no-break space so identical messages re-announce.
      element.textContent = element.textContent === message ? `${message} ` : message;
    },
    dispose() {
      element?.remove();
      element = null;
    },
  });
}

function position(context: DragAnnouncementContext): string {
  return context.targetIndex !== null && context.targetCount !== null
    ? `, drop target ${context.targetIndex} of ${context.targetCount}`
    : "";
}

/** Built-in English announcement builders; consumers override to localize. */
export const defaultDragAnnouncements: Required<DragAnnouncements> = Object.freeze({
  grab: (context) =>
    context.pointerType === "keyboard"
      ? `Picked up ${context.sourceLabel}. Use the arrow keys to choose a drop target, ` +
        "Enter to drop, Escape to cancel."
      : `Picked up ${context.sourceLabel}.`,
  move: (context) =>
    context.targetLabel === null ? null : `Over ${context.targetLabel}${position(context)}.`,
  drop: (context) =>
    context.targetLabel === null
      ? `${context.sourceLabel} released without a drop target.`
      : `Dropped ${context.sourceLabel} on ${context.targetLabel}.`,
  cancel: (context) => `Drag canceled. ${context.sourceLabel} was not moved.`,
});
