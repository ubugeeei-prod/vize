import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef } from "vue";

import type {
  AnnouncerController,
  AnnouncerPoliteness,
  BusyAnnouncement,
  BusyAnnouncementOptions,
} from "./announcer-types.ts";

const busyDiagnostic = "VIZE_UI_ANNOUNCER_BUSY";
const setupDiagnostic = "VIZE_UI_ANNOUNCER_SETUP";
const busyKeySequences = new WeakMap<AnnouncerController, number>();

function nextBusyKey(announcer: AnnouncerController): string {
  const sequence = (busyKeySequences.get(announcer) ?? 0) + 1;
  busyKeySequences.set(announcer, sequence);
  return `vize-ui-busy-${sequence}`;
}

function readLabel(value: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new TypeError(`${busyDiagnostic}: a busy announcement needs a non-empty label`);
  }
  return value;
}

/**
 * Begin announcing one busy task: loading, streaming, or background work.
 *
 * The label is announced immediately. Progress updates coalesce onto one
 * queue slot so a stream cannot flood assistive technology, and `end` cancels
 * unspoken progress before optionally announcing the outcome. Bind the
 * busy scope's `aria-busy` to `isBusy` so partial content is not read early.
 */
export function createBusyAnnouncement(
  announcer: AnnouncerController,
  options: BusyAnnouncementOptions,
): BusyAnnouncement {
  const label = readLabel(options.label);
  const politeness: AnnouncerPoliteness = options.politeness ?? "polite";
  const key = nextBusyKey(announcer);
  const isBusy = shallowRef(true);
  announcer.announce(label, { key, politeness });

  return Object.freeze({
    isBusy: shallowReadonly(isBusy),
    update: (text: string) => {
      if (!isBusy.value) {
        throw new Error(`${busyDiagnostic}: the busy announcement has already ended`);
      }
      announcer.announce(text, { key, politeness });
    },
    end: (text?: string) => {
      if (!isBusy.value) return;
      isBusy.value = false;
      announcer.cancel(key);
      if (text !== undefined) announcer.announce(text, { politeness });
    },
  });
}

/** Begin a busy announcement that silently ends with the current effect scope. */
export function useBusyAnnouncement(
  announcer: AnnouncerController,
  options: BusyAnnouncementOptions,
): BusyAnnouncement {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const busy = createBusyAnnouncement(announcer, options);
  onScopeDispose(() => busy.end());
  return busy;
}
