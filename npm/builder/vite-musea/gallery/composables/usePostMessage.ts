import { onUnmounted, toRaw, type Ref } from "vue";

export interface MuseaMessage {
  type: string;
  payload: unknown;
}

export function sendMessage(iframe: HTMLIFrameElement, type: string, payload: unknown = {}): void {
  const targetOrigin = resolvePreviewPostMessageOrigin(iframe.src, window.location.origin);
  if (!targetOrigin) return;
  iframe.contentWindow?.postMessage({ type, payload }, targetOrigin);
}

/** Snapshot a props/attrs record without reading Vue's instrumented property getters. */
export function snapshotMessageRecord(value: Record<string, unknown>): Record<string, unknown> {
  const raw = toRaw(value);
  return Object.fromEntries(
    Object.keys(raw).map((key) => {
      const descriptor = Object.getOwnPropertyDescriptor(raw, key);
      return [key, descriptor && "value" in descriptor ? descriptor.value : Reflect.get(raw, key)];
    }),
  );
}

/**
 * Gallery → iframe commands stay on the gallery origin. A wildcard target
 * would deliver `musea:set-props` / `musea:run-a11y` to a preview frame that
 * had been navigated cross-origin.
 */
export function resolvePreviewPostMessageOrigin(
  iframeSrc: string | undefined,
  pageOrigin: string,
): string | null {
  if (!iframeSrc) return pageOrigin;
  try {
    const srcOrigin = new URL(iframeSrc, pageOrigin).origin;
    return srcOrigin === pageOrigin ? pageOrigin : null;
  } catch {
    return null;
  }
}

export function sendMessageToAll(
  iframes: Ref<HTMLIFrameElement[]>,
  type: string,
  payload: unknown = {},
): void {
  for (const iframe of iframes.value) {
    sendMessage(iframe, type, payload);
  }
}

export function useMessageListener(
  type: string,
  callback: (payload: unknown, event: MessageEvent) => void,
): void {
  const handler = (event: MessageEvent) => {
    if (event.origin !== window.location.origin) return;
    const data = event.data as MuseaMessage | undefined;
    if (!data?.type?.startsWith("musea:")) return;
    if (data.type === type) {
      callback(data.payload, event);
    }
  };

  window.addEventListener("message", handler);
  onUnmounted(() => {
    window.removeEventListener("message", handler);
  });
}
