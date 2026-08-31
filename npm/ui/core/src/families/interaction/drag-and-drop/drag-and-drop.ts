import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef, toValue } from "vue";

import { createDragLiveRegion } from "./drag-and-drop-announce.ts";
import { createDragAutoScroller } from "./drag-and-drop-autoscroll.ts";
import {
  exceedsDistance,
  hitTest,
  readBoolean,
  readDistance,
  resolveEdge,
  validateCallbacks,
  validateEdges,
  validateKey,
} from "./drag-and-drop-internal.ts";
import { createDragEvent } from "./drag-and-drop-internal.ts";
import type { Point } from "./drag-and-drop-internal.ts";
import { handleSourceKeydown } from "./drag-and-drop-keyboard.ts";
import { installDragListeners } from "./drag-and-drop-listeners.ts";
import type { DragListenerSource } from "./drag-and-drop-listeners.ts";
import { createSessionCore } from "./drag-and-drop-session.ts";
import type { MeasuredTarget, SourceRecord, TargetRecord } from "./drag-and-drop-session.ts";
import { capture, pointerTypeOf, surfaceErrors } from "../move/move-internal.ts";
import { disableTextSelection, eventElement, isPrimaryPointer } from "../press/press-event.ts";
import type {
  DragAndDropController,
  DragAndDropOptions,
  DragSourceOptions,
  DragSourceProps,
  DragSourceRegistration,
  DropTargetOptions,
  DropTargetRegistration,
} from "./drag-and-drop-controller-types.ts";
import type { DragPointerType, DropTargetRect } from "./drag-and-drop-types.ts";

const disposedDiagnostic = "VIZE_UI_DRAG_AND_DROP_DISPOSED";
const setupDiagnostic = "VIZE_UI_DRAG_AND_DROP_SETUP";

/** Create an SSR-safe pointer, touch, and keyboard drag-and-drop coordinator. */
export function createDragAndDrop<Data = unknown>(
  options: DragAndDropOptions<Data> = {},
): DragAndDropController<Data> {
  validateCallbacks(options, ["onDragStart", "onDragMove", "onDragEnd"]);
  if (typeof options.startDistance !== "function") {
    readDistance(options.startDistance, "startDistance", 4);
  }
  const liveRegion = createDragLiveRegion();
  const autoScroller = createDragAutoScroller(options.autoScroll);
  const core = createSessionCore(options, liveRegion, autoScroller);
  let disposed = false;
  let lastTouchTime = Number.NEGATIVE_INFINITY;

  const pointerPoint = (event: Event, point: Point): void => {
    const contact = core.getContact();
    if (disposed || !contact) return;
    if (
      readBoolean(options.isDisabled, "isDisabled") ||
      readBoolean(core.sources.get(contact.sourceKey)?.options.isDisabled, "isDisabled")
    ) {
      core.finishSession(event, true);
      return;
    }
    const errors: unknown[] = [];
    let session = core.getSession();
    if (!session) {
      const threshold = readDistance(options.startDistance, "startDistance", 4);
      if (!exceedsDistance(contact.origin, point, threshold)) return;
      session = core.beginSession(contact.pointerType, contact.sourceKey, event, point, errors);
    }
    event.preventDefault();
    session.point = point;
    const hit = hitTest(
      core
        .orderedTargets(session.payload)
        .filter(
          (candidate): candidate is MeasuredTarget<Data> & { rect: DropTargetRect } =>
            candidate.rect !== null,
        ),
      point,
    );
    core.updateOver(
      session,
      hit,
      hit ? resolveEdge(hit.rect, point, hit.record.readEdges()) : null,
      event,
      errors,
    );
    if (core.getSession() === session) {
      const move = createDragEvent(
        "dragmove",
        session.pointerType,
        session.sourceKey,
        session.payload,
        session.targetKey,
        session.edge,
        point,
        event,
      );
      capture(errors, () => options.onDragMove?.(move));
    }
    autoScroller.update(point);
    surfaceErrors(errors, "Drag callbacks failed");
  };

  const armPointer = (
    event: Event,
    source: DragListenerSource,
    pointerType: Exclude<DragPointerType, "keyboard">,
    contactId: number | null,
    origin: Point,
    key: string,
  ): void => {
    if (disposed || core.getContact() || core.getSession()) return;
    if (
      readBoolean(options.isDisabled, "isDisabled") ||
      readBoolean(core.sources.get(key)?.options.isDisabled, "isDisabled")
    ) {
      return;
    }
    const host = eventElement(event);
    if (!host) return;
    event.preventDefault();
    event.stopPropagation();
    let releaseListeners: () => void = () => undefined;
    let restoreSelection: () => void = () => undefined;
    try {
      releaseListeners = installDragListeners(host.ownerDocument, source, {
        getContactId: () => core.getContact()?.contactId ?? null,
        onPoint: pointerPoint,
        onFinish: (finishEvent, canceled) => core.finishSession(finishEvent, canceled),
      });
      restoreSelection = disableTextSelection(host);
    } catch (error) {
      const errors: unknown[] = [error];
      capture(errors, releaseListeners);
      capture(errors, restoreSelection);
      surfaceErrors(errors, "Drag start cleanup failed");
      throw error;
    }
    core.setContact({
      sourceKey: key,
      source,
      contactId,
      origin,
      pointerType,
      releaseListeners,
      restoreSelection,
    });
  };

  const buildSourceProps = (key: string): Readonly<DragSourceProps> =>
    Object.freeze({
      onDragstart(event: DragEvent) {
        if (core.getContact()?.sourceKey === key || core.getSession()?.sourceKey === key) {
          core.finishSession(event, true);
        }
      },
      onFocusout(event: FocusEvent) {
        const session = core.getSession();
        if (session?.pointerType === "keyboard" && session.sourceKey === key) {
          core.finishSession(event, true);
        }
      },
      onKeydown(event: KeyboardEvent) {
        if (!disposed) handleSourceKeydown(core, options, event, key);
      },
      onMousedown(event: MouseEvent) {
        if (event.button !== 0 || (event.view && "PointerEvent" in event.view)) return;
        const elapsed = event.timeStamp - lastTouchTime;
        if (elapsed >= 0 && elapsed < 800) return;
        armPointer(event, "mouse", "mouse", null, { x: event.clientX, y: event.clientY }, key);
      },
      onPointerdown(event: PointerEvent) {
        if (!isPrimaryPointer(event)) return;
        const origin = { x: event.clientX, y: event.clientY };
        armPointer(event, "pointer", pointerTypeOf(event), event.pointerId, origin, key);
      },
      onTouchstart(event: TouchEvent) {
        lastTouchTime = event.timeStamp;
        if ((event.view && "PointerEvent" in event.view) || event.changedTouches.length !== 1) {
          return;
        }
        const touch = event.changedTouches.item(0);
        if (touch) {
          const origin = { x: touch.clientX, y: touch.clientY };
          armPointer(event, "touch", "touch", touch.identifier, origin, key);
        }
      },
    });

  return Object.freeze({
    isDragging: shallowReadonly(core.dragging),
    sourceKey: shallowReadonly(core.activeSourceKey),
    targetKey: shallowReadonly(core.activeTargetKey),
    indicator: shallowReadonly(core.indicator),
    registerSource(sourceOptions: DragSourceOptions<Data>): DragSourceRegistration {
      if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
      const key = validateKey(sourceOptions.key, core.sources);
      const record: SourceRecord<Data> = { options: sourceOptions, dragging: shallowRef(false) };
      core.sources.set(key, record);
      return Object.freeze({
        key,
        isDragging: shallowReadonly(record.dragging),
        sourceProps: buildSourceProps(key),
        dispose: () => {
          if (core.sources.get(key) !== record) return;
          if (core.getContact()?.sourceKey === key || core.getSession()?.sourceKey === key) {
            core.finishSession(null, true);
          }
          core.sources.delete(key);
        },
      });
    },
    registerTarget(targetOptions: DropTargetOptions<Data>): DropTargetRegistration {
      if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
      const key = validateKey(targetOptions.key, core.targets);
      validateCallbacks(targetOptions, [
        "accepts",
        "getRect",
        "onDrop",
        "onEnter",
        "onLeave",
        "onMove",
      ]);
      if (Array.isArray(targetOptions.edges)) validateEdges(targetOptions.edges);
      const record: TargetRecord<Data> = {
        options: targetOptions,
        readEdges: () => validateEdges(toValue(targetOptions.edges)),
        over: shallowRef(false),
      };
      core.targets.set(key, record);
      return Object.freeze({
        key,
        isOver: shallowReadonly(record.over),
        dispose: () => {
          if (core.targets.get(key) !== record) return;
          core.targets.delete(key);
          const session = core.getSession();
          if (session?.targetKey === key) {
            record.over.value = false;
            session.targetKey = null;
            session.edge = null;
            core.activeTargetKey.value = null;
            core.indicator.value = null;
          }
        },
      });
    },
    announce(message: string) {
      if (disposed || typeof message !== "string" || message.length === 0) return;
      if (typeof document !== "undefined") liveRegion.announce(document, message);
    },
    cancel: () => {
      if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
      return core.finishSession(null, true);
    },
    dispose: () => {
      if (disposed) return;
      disposed = true;
      const errors: unknown[] = [];
      capture(errors, () => core.finishSession(null, true, true));
      capture(errors, liveRegion.dispose);
      surfaceErrors(errors, "Drag disposal failed");
    },
  });
}

/** Create a drag-and-drop coordinator disposed with the current effect scope. */
export function useDragAndDrop<Data = unknown>(
  options: DragAndDropOptions<Data> = {},
): DragAndDropController<Data> {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createDragAndDrop(options);
  onScopeDispose(controller.dispose);
  return controller;
}

export { defaultDragAnnouncements } from "./drag-and-drop-announce.ts";
export {
  DRAG_TRANSFER_TYPE,
  readClipboardTransfer,
  readDragTransfer,
  writeClipboardTransfer,
  writeDragTransfer,
} from "./drag-and-drop-transfer.ts";
export type {
  DragAndDropController,
  DragAndDropOptions,
  DragAutoScrollOptions,
  DragSourceOptions,
  DragSourceProps,
  DragSourceRegistration,
  DropTargetOptions,
  DropTargetRegistration,
} from "./drag-and-drop-controller-types.ts";
export type {
  DragAnnouncementContext,
  DragAnnouncements,
  DragEndEvent,
  DragEventType,
  DragLifecycleEvent,
  DragMoveEvent,
  DragPayload,
  DragPointerType,
  DragStartEvent,
  DropEdge,
  DropIndicatorState,
  DropTargetEvent,
  DropTargetEventType,
  DropTargetRect,
} from "./drag-and-drop-types.ts";
