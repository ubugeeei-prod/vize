import {
  getCurrentScope,
  nextTick,
  onScopeDispose,
  shallowReadonly,
  shallowRef,
  toValue,
} from "vue";

import type {
  AnnouncerController,
  AnnouncerMessageOptions,
  AnnouncerOptions,
  AnnouncerOwnership,
  AnnouncerPoliteness,
} from "./announcer-types.ts";
import { createContext } from "../../foundations/context/context.ts";
import { createLiveRegion } from "../live-region/live-region-runtime.ts";

const invalidOptionDiagnostic = "VIZE_UI_ANNOUNCER_OPTION";
const disposedDiagnostic = "VIZE_UI_ANNOUNCER_DISPOSED";
const setupDiagnostic = "VIZE_UI_ANNOUNCER_SETUP";
const politenessValues = new Set<AnnouncerPoliteness>(["assertive", "polite"]);

/** Typed announcer context provided by AnnouncerProvider. */
export const announcerContext = createContext<AnnouncerController>("Announcer");

interface PendingAnnouncement {
  text: string;
  politeness: AnnouncerPoliteness;
  readonly key: string | undefined;
}

function readPoliteness(
  value: AnnouncerOptions["politeness"],
  fallback: AnnouncerPoliteness,
): AnnouncerPoliteness {
  const resolved = toValue(value);
  if (resolved === undefined) return fallback;
  if (!politenessValues.has(resolved)) {
    throw new TypeError(
      `${invalidOptionDiagnostic}: politeness must resolve to polite or assertive`,
    );
  }
  return resolved;
}

function validateMessage(text: string, options: AnnouncerMessageOptions): void {
  if (typeof text !== "string") {
    throw new TypeError(`${invalidOptionDiagnostic}: announcement text must be a string`);
  }
  if (options.politeness !== undefined) readPoliteness(options.politeness, "polite");
  if (options.key !== undefined && (typeof options.key !== "string" || options.key.length === 0)) {
    throw new TypeError(`${invalidOptionDiagnostic}: a coalescing key must be a non-empty string`);
  }
}

/**
 * Create an SSR-safe announcement queue.
 *
 * Announcements flush sequentially into one polite and one assertive
 * live-region channel, so status updates and errors never overwrite each
 * other mid-announcement. The server renders both channels empty: queued text
 * only ever reaches the DOM after client-side ticks.
 */
export function createAnnouncer(options: AnnouncerOptions = {}): AnnouncerController {
  if (typeof options.politeness !== "function") readPoliteness(options.politeness, "polite");
  const polite = createLiveRegion({ politeness: "polite" });
  const assertive = createLiveRegion({ politeness: "assertive" });
  const queue: PendingAnnouncement[] = [];
  const pendingCount = shallowRef(0);
  let inFlight: PendingAnnouncement | undefined;
  let disposed = false;
  let flushing = false;

  const assertAlive = (): void => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };

  const syncPending = (): void => {
    pendingCount.value = queue.length;
  };

  const flush = async (): Promise<void> => {
    while (!disposed) {
      const next = queue.shift();
      if (next === undefined) break;
      inFlight = next;
      syncPending();
      (next.politeness === "assertive" ? assertive : polite).announce(next.text);
      await nextTick();
      await nextTick();
    }
    inFlight = undefined;
    flushing = false;
  };

  const scheduleFlush = (): void => {
    if (flushing) return;
    flushing = true;
    void flush();
  };

  const isDuplicate = (text: string, politeness: AnnouncerPoliteness): boolean => {
    if (inFlight !== undefined && inFlight.text === text && inFlight.politeness === politeness) {
      return true;
    }
    return queue.some((pending) => pending.text === text && pending.politeness === politeness);
  };

  return Object.freeze({
    politeMessage: polite.message,
    assertiveMessage: assertive.message,
    pendingCount: shallowReadonly(pendingCount),
    announce: (text: string, messageOptions: AnnouncerMessageOptions = {}) => {
      assertAlive();
      validateMessage(text, messageOptions);
      const politeness = messageOptions.politeness ?? readPoliteness(options.politeness, "polite");
      const keyed =
        messageOptions.key === undefined
          ? undefined
          : queue.find((pending) => pending.key === messageOptions.key);
      if (keyed !== undefined) {
        keyed.text = text;
        keyed.politeness = politeness;
        return true;
      }
      if (isDuplicate(text, politeness)) return false;
      const message: PendingAnnouncement = { text, politeness, key: messageOptions.key };
      if (politeness === "assertive") {
        const firstPolite = queue.findIndex((pending) => pending.politeness === "polite");
        queue.splice(firstPolite === -1 ? queue.length : firstPolite, 0, message);
      } else {
        queue.push(message);
      }
      syncPending();
      scheduleFlush();
      return true;
    },
    cancel: (key: string) => {
      assertAlive();
      const index = queue.findIndex((pending) => pending.key === key);
      if (index === -1) return false;
      queue.splice(index, 1);
      syncPending();
      return true;
    },
    clear: () => {
      assertAlive();
      queue.length = 0;
      syncPending();
      polite.clear();
      assertive.clear();
    },
    dispose: () => {
      if (disposed) return;
      disposed = true;
      queue.length = 0;
      pendingCount.value = 0;
      polite.dispose();
      assertive.dispose();
    },
  });
}

/** Create an announcer disposed with the current Vue effect scope. */
export function useAnnouncer(options: AnnouncerOptions = {}): AnnouncerController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createAnnouncer(options);
  onScopeDispose(controller.dispose);
  return controller;
}

/**
 * Resolve provider ownership during component setup.
 *
 * The outermost provider creates and owns the announcement queue; nested
 * providers reuse it, so a document never renders duplicate live regions no
 * matter how many islands or layout shells declare their own provider.
 */
export function useAnnouncerOwnership(options: AnnouncerOptions = {}): AnnouncerOwnership {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const existing = announcerContext.useOptional();
  if (existing !== undefined) {
    return Object.freeze({ announcer: existing, isOwner: false });
  }
  const announcer = useAnnouncer(options);
  announcerContext.provide(announcer);
  return Object.freeze({ announcer, isOwner: true });
}
