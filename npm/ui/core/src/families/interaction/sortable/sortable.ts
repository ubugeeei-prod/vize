import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef, toValue } from "vue";

import {
  compareDocumentOrder,
  measureRect,
  readBoolean,
  readLabel,
  validateCallbacks,
  validateKey,
} from "../drag-and-drop/drag-and-drop-internal.ts";
import {
  createDragAndDrop,
  type DragAnnouncementContext,
  type DragEndEvent,
  type DragMoveEvent,
  type DragStartEvent,
} from "../drag-and-drop/drag-and-drop.ts";
import { cancelSortableKeyboard, handleSortableKeydown } from "./sortable-keyboard.ts";
import type { SortableKeyboardHost } from "./sortable-keyboard.ts";
import {
  createSortableEvent,
  defaultSortableAnnouncements,
  dispatchSortableEvent,
  edgesFor,
  projectDragContext,
  readColumns,
  readDirection,
  readOrientation,
  sortableContextFor,
  type SortableItemRecord,
  type SortableProjection,
  type SortableSession,
} from "./sortable-internal.ts";
import type {
  SortableAnnouncementContext,
  SortableController,
  SortableEvent,
  SortableIndicatorState,
  SortableItemOptions,
  SortableItemRegistration,
  SortableOptions,
} from "./sortable-types.ts";

const disposedDiagnostic = "VIZE_UI_SORTABLE_DISPOSED";
const setupDiagnostic = "VIZE_UI_SORTABLE_SETUP";

/** Create an SSR-safe sortable coordinator for lists, grids, and trees. */
export function createSortable(options: SortableOptions = {}): SortableController {
  validateCallbacks(options, ["onSortStart", "onSortPreview", "onSortCommit", "onSortCancel"]);
  if (typeof options.orientation !== "function") readOrientation(options.orientation);
  if (typeof options.direction !== "function") readDirection(options.direction);
  if (typeof options.columns !== "function") readColumns(options.columns);
  const announcements = { ...defaultSortableAnnouncements, ...options.announcements };
  const items = new Map<string, SortableItemRecord>();
  const sorting = shallowRef(false);
  const activeKey = shallowRef<string | null>(null);
  const indicator = shallowRef<SortableIndicatorState | null>(null);
  let session: SortableSession | null = null;
  let disposed = false;

  const orderedKeys = (): string[] => {
    const list = [...items.entries()].map(([key, record]) => ({
      key,
      element: toValue(record.options.element) ?? null,
    }));
    list.sort((left, right) =>
      left.element && right.element ? compareDocumentOrder(left.element, right.element) : 0,
    );
    return list.map((entry) => entry.key);
  };

  const labelOf = (key: string): string => readLabel(items.get(key)?.options.label, key);

  const emit = (event: SortableEvent): void => dispatchSortableEvent(options, event);

  const projection = (
    dragContext: Pick<DragAnnouncementContext, "edge" | "sourceKey" | "targetKey">,
  ): SortableProjection =>
    projectDragContext(
      orderedKeys(),
      readOrientation(options.orientation),
      readDirection(options.direction),
      dragContext,
    );

  const sortableContext = (
    phase: "cancel" | "drop" | "grab" | "move",
    dragContext: DragAnnouncementContext,
  ): SortableAnnouncementContext =>
    sortableContextFor(
      phase,
      dragContext,
      projection(dragContext),
      orderedKeys().length,
      session?.originIndex ?? null,
    );

  const dnd = createDragAndDrop({
    isDisabled: options.isDisabled,
    startDistance: options.startDistance,
    announcements: {
      grab: (context) => announcements.grab(sortableContext("grab", context)),
      move: (context) => announcements.move(sortableContext("move", context)),
      drop: (context) => announcements.drop(sortableContext("drop", context)),
      cancel: (context) => announcements.cancel(sortableContext("cancel", context)),
    },
    onDragStart(event: DragStartEvent) {
      const index = Math.max(orderedKeys().indexOf(event.sourceKey), 0);
      session = {
        key: event.sourceKey,
        pointerType: event.pointerType,
        originIndex: index,
        toIndex: index,
        overKey: null,
        position: null,
      };
      sorting.value = true;
      activeKey.value = event.sourceKey;
      emit(
        createSortableEvent(
          "sortstart",
          event.pointerType,
          event.sourceKey,
          index,
          index,
          null,
          null,
          event.originalEvent,
        ),
      );
    },
    onDragMove(event: DragMoveEvent) {
      const current = session;
      if (!current) return;
      if (event.targetKey === null || event.edge === null) {
        indicator.value = null;
        current.overKey = null;
        current.position = null;
        return;
      }
      const { currentIndex, overIndex, toIndex, position } = projection(event);
      if (overIndex < 0 || position === null) return;
      if (position !== "inside" && toIndex === currentIndex) {
        indicator.value = null;
        current.overKey = null;
        current.position = null;
        current.toIndex = currentIndex;
        return;
      }
      const changed =
        current.overKey !== event.targetKey ||
        current.position !== position ||
        current.toIndex !== toIndex;
      current.overKey = event.targetKey;
      current.position = position;
      current.toIndex = toIndex;
      const shape = dnd.indicator.value;
      indicator.value = {
        key: event.targetKey,
        position,
        toIndex,
        rect: shape?.targetKey === event.targetKey ? shape.rect : null,
        line: shape?.targetKey === event.targetKey ? shape.line : null,
      };
      if (changed) {
        emit(
          createSortableEvent(
            "sortpreview",
            current.pointerType,
            current.key,
            currentIndex,
            toIndex,
            event.targetKey,
            position,
            event.originalEvent,
          ),
        );
      }
    },
    onDragEnd(event: DragEndEvent) {
      const current = session;
      if (!current) return;
      session = null;
      sorting.value = false;
      activeKey.value = null;
      indicator.value = null;
      const keys = orderedKeys();
      const currentIndex = Math.max(keys.indexOf(current.key), 0);
      if (event.isCanceled || event.targetKey === null) {
        emit(
          createSortableEvent(
            "sortcancel",
            current.pointerType,
            current.key,
            currentIndex,
            current.originIndex,
            null,
            null,
            event.originalEvent,
          ),
        );
        return;
      }
      const { overIndex, toIndex, position } = projection(event);
      const insideIndex = position === "inside" ? Math.max(overIndex, 0) : toIndex;
      emit(
        createSortableEvent(
          "sortcommit",
          current.pointerType,
          current.key,
          current.originIndex,
          insideIndex,
          event.targetKey,
          position,
          event.originalEvent,
        ),
      );
    },
  });

  const keyboardHost: SortableKeyboardHost = {
    options,
    announcements,
    indicator,
    getSession: () => session,
    startSession: (next) => {
      session = next;
      sorting.value = true;
      activeKey.value = next.key;
    },
    clearSession: () => {
      session = null;
      sorting.value = false;
      activeKey.value = null;
      indicator.value = null;
    },
    orderedKeys,
    labelOf,
    measureItem: (key) => {
      const record = items.get(key);
      if (!record) return null;
      return measureRect(toValue(record.options.element) ?? null, record.options.getRect);
    },
    isItemDisabled: (key) => readBoolean(items.get(key)?.options.isDisabled, "isDisabled"),
    announce: (message) => {
      if (message !== null) dnd.announce(message);
    },
    emit,
  };

  return Object.freeze({
    isSorting: shallowReadonly(sorting),
    activeKey: shallowReadonly(activeKey),
    indicator: shallowReadonly(indicator),
    registerItem(itemOptions: SortableItemOptions): SortableItemRegistration {
      if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
      const key = validateKey(itemOptions.key, items);
      validateCallbacks(itemOptions, ["getRect"]);
      const source = dnd.registerSource({
        key,
        element: itemOptions.element,
        label: itemOptions.label,
        isDisabled: itemOptions.isDisabled,
        keyboard: false,
      });
      const target = dnd.registerTarget({
        key,
        element: itemOptions.element,
        label: itemOptions.label,
        edges: () =>
          edgesFor(readOrientation(options.orientation), readBoolean(options.nesting, "nesting")),
        ...(itemOptions.getRect ? { getRect: itemOptions.getRect } : {}),
      });
      const record: SortableItemRecord = { options: itemOptions, source, target };
      items.set(key, record);
      const itemProps = Object.freeze({
        ...source.sourceProps,
        onKeydown: (event: KeyboardEvent) => {
          if (!disposed) handleSortableKeydown(keyboardHost, event, key);
        },
        onFocusout: (event: FocusEvent) => {
          if (disposed) return;
          if (session?.pointerType === "keyboard" && session.key === key) {
            cancelSortableKeyboard(keyboardHost, event);
          }
        },
      });
      return Object.freeze({
        key,
        isDragging: source.isDragging,
        itemProps,
        dispose: () => {
          if (items.get(key) !== record) return;
          if (session?.pointerType === "keyboard" && session.key === key) {
            cancelSortableKeyboard(keyboardHost, null);
          }
          source.dispose();
          target.dispose();
          items.delete(key);
        },
      });
    },
    cancel: () => {
      if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
      if (cancelSortableKeyboard(keyboardHost, null)) return true;
      return dnd.cancel();
    },
    dispose: () => {
      if (disposed) return;
      disposed = true;
      session = null;
      sorting.value = false;
      activeKey.value = null;
      indicator.value = null;
      dnd.dispose();
    },
  });
}

/** Create a sortable coordinator disposed with the current Vue effect scope. */
export function useSortable(options: SortableOptions = {}): SortableController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createSortable(options);
  onScopeDispose(controller.dispose);
  return controller;
}

export type {
  SortableAnnouncementContext,
  SortableAnnouncements,
  SortableController,
  SortableDirection,
  SortableEvent,
  SortableEventType,
  SortableIndicatorState,
  SortableItemOptions,
  SortableItemRegistration,
  SortableOptions,
  SortableOrientation,
  SortablePosition,
  SortCancelEvent,
  SortCommitEvent,
  SortPreviewEvent,
  SortStartEvent,
} from "./sortable-types.ts";
export { defaultSortableAnnouncements } from "./sortable-internal.ts";
