import type { ShallowRef } from "vue";

import { indicatorFor, readBoolean } from "../drag-and-drop/drag-and-drop-internal.ts";
import { eventElement } from "../press/press-event.ts";
import {
  createSortableEvent,
  edgeForPosition,
  keyboardDelta,
  readColumns,
  readDirection,
  readOrientation,
} from "./sortable-internal.ts";
import type { SortableSession } from "./sortable-internal.ts";
import type { DropTargetRect } from "../drag-and-drop/drag-and-drop-types.ts";
import type {
  SortableAnnouncementContext,
  SortableAnnouncements,
  SortableEvent,
  SortableIndicatorState,
  SortableOptions,
  SortablePosition,
} from "./sortable-types.ts";

/** Controller internals the keyboard state machine drives. */
export interface SortableKeyboardHost {
  readonly options: SortableOptions;
  readonly announcements: Required<SortableAnnouncements>;
  readonly indicator: ShallowRef<SortableIndicatorState | null>;
  readonly getSession: () => SortableSession | null;
  readonly startSession: (session: SortableSession) => void;
  readonly clearSession: () => void;
  readonly orderedKeys: () => readonly string[];
  readonly labelOf: (key: string) => string;
  readonly measureItem: (key: string) => DropTargetRect | null;
  readonly isItemDisabled: (key: string) => boolean;
  readonly announce: (message: string | null) => void;
  readonly emit: (event: SortableEvent) => void;
}

function context(
  host: SortableKeyboardHost,
  session: SortableSession,
  fromIndex: number,
  toIndex: number,
): SortableAnnouncementContext {
  return {
    pointerType: session.pointerType,
    key: session.key,
    label: host.labelOf(session.key),
    fromIndex,
    toIndex,
    count: host.orderedKeys().length,
    overKey: session.overKey,
    overLabel: session.overKey === null ? null : host.labelOf(session.overKey),
    position: session.position,
  };
}

function setIndicator(
  host: SortableKeyboardHost,
  referenceKey: string,
  position: SortablePosition,
  toIndex: number,
): void {
  const orientation = readOrientation(host.options.orientation);
  const direction = readDirection(host.options.direction);
  const edge = edgeForPosition(position, orientation, direction);
  const shape = indicatorFor(referenceKey, edge, host.measureItem(referenceKey));
  host.indicator.value = {
    key: referenceKey,
    position,
    toIndex,
    rect: shape.rect,
    line: shape.line,
  };
}

function moveTo(host: SortableKeyboardHost, event: KeyboardEvent, requested: number): void {
  const session = host.getSession();
  if (!session) return;
  const keys = host.orderedKeys();
  const next = Math.min(Math.max(requested, 0), keys.length - 1);
  const changed = next !== session.toIndex || session.position !== null;
  session.toIndex = next;
  session.overKey = null;
  session.position = null;
  const currentIndex = Math.max(keys.indexOf(session.key), 0);
  const referenceKey = keys[next];
  if (referenceKey !== undefined) {
    setIndicator(host, referenceKey, next >= currentIndex ? "after" : "before", next);
  }
  if (!changed) return;
  host.emit(
    createSortableEvent(
      "sortpreview",
      session.pointerType,
      session.key,
      currentIndex,
      next,
      null,
      null,
      event,
    ),
  );
  host.announce(host.announcements.move(context(host, session, currentIndex, next)));
}

function nest(host: SortableKeyboardHost, event: KeyboardEvent): void {
  const session = host.getSession();
  if (!session || session.position === "inside") return;
  const keys = host.orderedKeys();
  let referenceIndex = session.toIndex - 1;
  while (referenceIndex >= 0 && keys[referenceIndex] === session.key) referenceIndex -= 1;
  const referenceKey = referenceIndex >= 0 ? keys[referenceIndex] : undefined;
  if (referenceKey === undefined) return;
  session.overKey = referenceKey;
  session.position = "inside";
  setIndicator(host, referenceKey, "inside", referenceIndex);
  const currentIndex = Math.max(keys.indexOf(session.key), 0);
  host.emit(
    createSortableEvent(
      "sortpreview",
      session.pointerType,
      session.key,
      currentIndex,
      referenceIndex,
      referenceKey,
      "inside",
      event,
    ),
  );
  host.announce(host.announcements.move(context(host, session, currentIndex, referenceIndex)));
}

function settle(host: SortableKeyboardHost, event: Event | null, canceled: boolean): void {
  const session = host.getSession();
  if (!session) return;
  const keys = host.orderedKeys();
  const currentIndex = Math.max(keys.indexOf(session.key), 0);
  const toIndex =
    session.position === "inside" && session.overKey !== null
      ? Math.max(keys.indexOf(session.overKey), 0)
      : session.toIndex;
  host.clearSession();
  if (canceled) {
    host.emit(
      createSortableEvent(
        "sortcancel",
        session.pointerType,
        session.key,
        currentIndex,
        session.originIndex,
        null,
        null,
        event,
      ),
    );
    const restored: SortableSession = {
      ...session,
      toIndex: session.originIndex,
      overKey: null,
      position: null,
    };
    host.announce(
      host.announcements.cancel(context(host, restored, currentIndex, session.originIndex)),
    );
  } else {
    host.emit(
      createSortableEvent(
        "sortcommit",
        session.pointerType,
        session.key,
        session.originIndex,
        toIndex,
        session.overKey,
        session.position,
        event,
      ),
    );
    host.announce(host.announcements.drop(context(host, session, session.originIndex, toIndex)));
  }
}

function grab(host: SortableKeyboardHost, event: KeyboardEvent, key: string): void {
  if (event.altKey || event.ctrlKey || event.metaKey || event.isComposing) return;
  if (readBoolean(host.options.isDisabled, "isDisabled") || host.isItemDisabled(key)) return;
  const target = eventElement(event);
  if (!target || event.target !== target) return;
  const index = host.orderedKeys().indexOf(key);
  if (index < 0) return;
  event.preventDefault();
  event.stopPropagation();
  const session: SortableSession = {
    key,
    pointerType: "keyboard",
    originIndex: index,
    toIndex: index,
    overKey: null,
    position: null,
  };
  host.startSession(session);
  host.emit(createSortableEvent("sortstart", "keyboard", key, index, index, null, null, event));
  host.announce(host.announcements.grab(context(host, session, index, index)));
}

/** Handle one keydown from an item: grab, move, nest, drop, or cancel. */
export function handleSortableKeydown(
  host: SortableKeyboardHost,
  event: KeyboardEvent,
  key: string,
): void {
  const session = host.getSession();
  if (session?.pointerType === "keyboard" && session.key === key) {
    if (readBoolean(host.options.isDisabled, "isDisabled")) {
      settle(host, event, true);
      return;
    }
    const orientation = readOrientation(host.options.orientation);
    const nesting = readBoolean(host.options.nesting, "nesting");
    switch (event.key) {
      case "Enter":
      case " ":
        settle(host, event, false);
        break;
      case "Escape":
        settle(host, event, true);
        break;
      case "Home":
        moveTo(host, event, 0);
        break;
      case "End":
        moveTo(host, event, host.orderedKeys().length - 1);
        break;
      case "Tab":
        settle(host, event, true);
        return;
      default: {
        if (nesting && orientation === "vertical" && event.key === "ArrowRight") {
          nest(host, event);
          break;
        }
        if (nesting && orientation === "vertical" && event.key === "ArrowLeft") {
          moveTo(host, event, session.toIndex);
          break;
        }
        const delta = keyboardDelta(
          event.key,
          orientation,
          readDirection(host.options.direction),
          readColumns(host.options.columns),
        );
        if (delta === null) return;
        moveTo(host, event, session.toIndex + delta);
        break;
      }
    }
    event.preventDefault();
    event.stopPropagation();
  } else if (!session && (event.key === "Enter" || event.key === " ")) {
    grab(host, event, key);
  }
}

/** Cancel an owned keyboard sort from focus loss or programmatic teardown. */
export function cancelSortableKeyboard(host: SortableKeyboardHost, event: Event | null): boolean {
  const session = host.getSession();
  if (session?.pointerType !== "keyboard") return false;
  settle(host, event, true);
  return true;
}
