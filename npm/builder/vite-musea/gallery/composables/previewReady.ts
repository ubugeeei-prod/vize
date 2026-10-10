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
