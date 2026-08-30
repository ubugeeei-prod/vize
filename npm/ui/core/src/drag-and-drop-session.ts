import { shallowRef, toValue } from "vue";
import type { ShallowRef } from "vue";

import { defaultDragAnnouncements } from "./drag-and-drop-announce.ts";
import type { DragLiveRegion } from "./drag-and-drop-announce.ts";
import type { DragAutoScroller } from "./drag-and-drop-autoscroll.ts";
import type {
  DragAndDropOptions,
  DragSourceOptions,
  DropTargetOptions,
} from "./drag-and-drop-controller-types.ts";
import {
  compareDocumentOrder,
  createDragEvent,
  createDropTargetEvent,
  indicatorFor,
  measureRect,
  readBoolean,
  readLabel,
} from "./drag-and-drop-internal.ts";
import type { Point } from "./drag-and-drop-internal.ts";
import type { DragListenerSource } from "./drag-and-drop-listeners.ts";
import { capture, surfaceErrors } from "./families/interaction/move/move-internal.ts";
import type {
  DragAnnouncementContext,
  DragPayload,
  DragPointerType,
  DropEdge,
  DropIndicatorState,
  DropTargetEvent,
  DropTargetEventType,
  DropTargetRect,
} from "./drag-and-drop-types.ts";

export interface SourceRecord<Data> {
  readonly options: DragSourceOptions<Data>;
  readonly dragging: ShallowRef<boolean>;
}

export interface TargetRecord<Data> {
  readonly options: DropTargetOptions<Data>;
  readonly readEdges: () => readonly DropEdge[];
  readonly over: ShallowRef<boolean>;
}

export interface MeasuredTarget<Data> {
  readonly key: string;
  readonly record: TargetRecord<Data>;
  readonly element: Element | null;
  readonly rect: DropTargetRect | null;
}

export interface Contact {
  readonly sourceKey: string;
  readonly source: DragListenerSource;
  readonly contactId: number | null;
  readonly origin: Point;
  readonly pointerType: Exclude<DragPointerType, "keyboard">;
  readonly releaseListeners: () => void;
  readonly restoreSelection: () => void;
}

export interface Session<Data> {
  readonly sourceKey: string;
  readonly pointerType: DragPointerType;
  readonly payload: DragPayload<Data> | null;
  targetKey: string | null;
  edge: DropEdge | null;
  point: Point | null;
}

type AnnouncementPhase = "cancel" | "drop" | "grab" | "move";

/** Shared mutable session state and settlement logic for one controller. */
export function createSessionCore<Data>(
  options: DragAndDropOptions<Data>,
  liveRegion: DragLiveRegion,
  autoScroller: DragAutoScroller,
) {
  const announcements = { ...defaultDragAnnouncements, ...options.announcements };
  const sources = new Map<string, SourceRecord<Data>>();
  const targets = new Map<string, TargetRecord<Data>>();
  const dragging = shallowRef(false);
  const activeSourceKey = shallowRef<string | null>(null);
  const activeTargetKey = shallowRef<string | null>(null);
  const indicator = shallowRef<DropIndicatorState | null>(null);
  let contact: Contact | null = null;
  let session: Session<Data> | null = null;

  const orderedTargets = (payload: DragPayload<Data> | null): MeasuredTarget<Data>[] => {
    const list: MeasuredTarget<Data>[] = [];
    for (const [key, record] of targets) {
      if (readBoolean(record.options.isDisabled, "isDisabled")) continue;
      if (record.options.accepts && !record.options.accepts(payload)) continue;
      const element = toValue(record.options.element) ?? null;
      list.push({ key, record, element, rect: measureRect(element, record.options.getRect) });
    }
    return list.sort((left, right) =>
      left.element && right.element ? compareDocumentOrder(left.element, right.element) : 0,
    );
  };

  const resolveDocument = (sess: Session<Data>): Document | null => {
    const sourceElement = toValue(sources.get(sess.sourceKey)?.options.element);
    if (sourceElement) return sourceElement.ownerDocument;
    return typeof document === "undefined" ? null : document;
  };

  const buildMessage = (phase: AnnouncementPhase, sess: Session<Data>): string | null => {
    const currentKey = sess.targetKey;
    const target = currentKey === null ? undefined : targets.get(currentKey);
    const ordered = orderedTargets(sess.payload);
    const index = ordered.findIndex((candidate) => candidate.key === currentKey);
    const context: DragAnnouncementContext<Data> = {
      pointerType: sess.pointerType,
      sourceKey: sess.sourceKey,
      sourceLabel: readLabel(sources.get(sess.sourceKey)?.options.label, sess.sourceKey),
      payload: sess.payload,
      targetKey: currentKey,
      targetLabel:
        target && currentKey !== null ? readLabel(target.options.label, currentKey) : null,
      targetIndex: index >= 0 ? index + 1 : null,
      targetCount: ordered.length > 0 ? ordered.length : null,
      edge: sess.edge,
    };
    const builder = announcements[phase] as (value: DragAnnouncementContext<Data>) => string | null;
    return builder(context) ?? null;
  };

  const announceMessage = (sess: Session<Data>, message: string | null): void => {
    const owner = resolveDocument(sess);
    if (owner && message) liveRegion.announce(owner, message);
  };

  const speak = (phase: AnnouncementPhase, sess: Session<Data>): void => {
    announceMessage(sess, buildMessage(phase, sess));
  };

  const targetEvent = (
    type: DropTargetEventType,
    key: string,
    sess: Session<Data>,
    edge: DropEdge | null,
    event: Event | null,
  ): DropTargetEvent<Data> =>
    createDropTargetEvent(
      type,
      key,
      sess.sourceKey,
      sess.pointerType,
      sess.payload,
      edge,
      sess.point,
      event,
    );

  const updateOver = (
    sess: Session<Data>,
    hit: MeasuredTarget<Data> | null,
    edge: DropEdge | null,
    event: Event | null,
    errors: unknown[],
  ): void => {
    const previousKey = sess.targetKey;
    if (previousKey !== null && previousKey !== (hit?.key ?? null)) {
      const previous = targets.get(previousKey);
      if (previous) {
        previous.over.value = false;
        const leave = targetEvent("dropleave", previousKey, sess, null, event);
        capture(errors, () => previous.options.onLeave?.(leave));
      }
    }
    if (!hit || edge === null) {
      sess.targetKey = null;
      sess.edge = null;
      activeTargetKey.value = null;
      indicator.value = null;
      return;
    }
    const entered = previousKey !== hit.key;
    const edgeChanged = !entered && sess.edge !== edge;
    sess.targetKey = hit.key;
    sess.edge = edge;
    activeTargetKey.value = hit.key;
    indicator.value = indicatorFor(hit.key, edge, hit.rect);
    if (entered) {
      hit.record.over.value = true;
      const enter = targetEvent("dropenter", hit.key, sess, edge, event);
      capture(errors, () => hit.record.options.onEnter?.(enter));
      capture(errors, () => speak("move", sess));
    } else {
      const move = targetEvent("dropmove", hit.key, sess, edge, event);
      capture(errors, () => hit.record.options.onMove?.(move));
      if (edgeChanged) capture(errors, () => speak("move", sess));
    }
  };

  const beginSession = (
    pointerType: DragPointerType,
    key: string,
    event: Event | null,
    point: Point | null,
    errors: unknown[],
  ): Session<Data> => {
    const record = sources.get(key);
    const payload = toValue(record?.options.payload) ?? null;
    const started: Session<Data> = {
      sourceKey: key,
      pointerType,
      payload,
      targetKey: null,
      edge: null,
      point,
    };
    session = started;
    dragging.value = true;
    activeSourceKey.value = key;
    if (record) record.dragging.value = true;
    if (pointerType !== "keyboard") capture(errors, () => speak("grab", started));
    const start = createDragEvent("dragstart", pointerType, key, payload, null, null, point, event);
    capture(errors, () => options.onDragStart?.(start));
    return started;
  };

  const finishSession = (event: Event | null, isCanceled: boolean, silent = false): boolean => {
    const currentContact = contact;
    const currentSession = session;
    if (!currentContact && !currentSession) return false;
    contact = null;
    session = null;
    const errors: unknown[] = [];
    if (currentContact) {
      capture(errors, currentContact.releaseListeners);
      capture(errors, currentContact.restoreSelection);
    }
    autoScroller.stop();
    if (currentSession) {
      const record = sources.get(currentSession.sourceKey);
      const overKey = currentSession.targetKey;
      const target = overKey === null ? undefined : targets.get(overKey);
      if (target) target.over.value = false;
      dragging.value = false;
      activeSourceKey.value = null;
      activeTargetKey.value = null;
      indicator.value = null;
      if (record) record.dragging.value = false;
      if (!silent) {
        if (isCanceled) {
          if (target && overKey !== null) {
            const leave = targetEvent("dropleave", overKey, currentSession, null, event);
            capture(errors, () => target.options.onLeave?.(leave));
          }
          capture(errors, () => speak("cancel", currentSession));
        } else {
          if (target && overKey !== null) {
            const drop = targetEvent("drop", overKey, currentSession, currentSession.edge, event);
            capture(errors, () => target.options.onDrop?.(drop));
          }
          capture(errors, () => speak("drop", currentSession));
        }
        const end = createDragEvent(
          "dragend",
          currentSession.pointerType,
          currentSession.sourceKey,
          currentSession.payload,
          isCanceled ? null : overKey,
          isCanceled ? null : currentSession.edge,
          currentSession.point,
          event,
          isCanceled,
        );
        capture(errors, () => options.onDragEnd?.(end));
      }
    }
    surfaceErrors(errors, "Drag settlement failed");
    return true;
  };

  return {
    sources,
    targets,
    dragging,
    activeSourceKey,
    activeTargetKey,
    indicator,
    getContact: () => contact,
    setContact: (next: Contact | null) => {
      contact = next;
    },
    getSession: () => session,
    orderedTargets,
    buildMessage,
    announceMessage,
    updateOver,
    beginSession,
    finishSession,
  };
}

/** Stateful internals shared by the entry, pointer, and keyboard modules. */
export type SessionCore<Data> = ReturnType<typeof createSessionCore<Data>>;
