import { ref, watch, type Ref } from "vue";
import { useMessageListener } from "./usePostMessage";

/** Register before navigation: HTML load can precede the dynamic preview and async setup. */
export function waitForPreviewReady(
  iframe: HTMLIFrameElement,
  navigate: () => Promise<void>,
): Promise<void> {
  return new Promise((resolve, reject) => {
    const cleanup = () => {
      clearTimeout(timeout);
      window.removeEventListener("message", onMessage);
    };
    const onMessage = (event: MessageEvent) => {
      if (
        event.origin !== window.location.origin ||
        event.source !== iframe.contentWindow ||
        event.data?.type !== "musea:ready"
      )
        return;
      cleanup();
      resolve();
    };
    const timeout = setTimeout(() => {
      cleanup();
      reject(new Error("Preview did not become ready within 10s"));
    }, 10000);
    window.addEventListener("message", onMessage);
    void navigate().catch((error) => {
      cleanup();
      reject(error);
    });
  });
}

/** Track the same readiness signal for an iframe mounted by Vue. */
export function usePreviewReady(iframe: Ref<HTMLIFrameElement | null>, previewUrl: Ref<string>) {
  const ready = ref(false);
  useMessageListener("musea:ready", (_payload, event) => {
    if (event.source === iframe.value?.contentWindow) ready.value = true;
  });
  watch(previewUrl, () => {
    ready.value = false;
  });
  return ready;
}
