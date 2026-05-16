import { onMounted, onUnmounted } from "vue";
import { getWasm } from "../wasm/index";

/**
 * Workaround for vite-plugin-vize prop reactivity issue.
 * Uses getWasm() directly instead of props since prop updates aren't detected.
 * Polls for compiler availability and calls onReady when found.
 */
export function useCompilerInit(onReady: () => void) {
  let hasInitialized = false;
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  function tryInitialize() {
    const compiler = getWasm();
    if (compiler && !hasInitialized) {
      hasInitialized = true;
      if (pollInterval) {
        clearInterval(pollInterval);
        pollInterval = null;
      }
      onReady();
    }
  }

  onMounted(() => {
    tryInitialize();
    if (!hasInitialized) {
      pollInterval = setInterval(tryInitialize, 100);
      setTimeout(() => {
        if (pollInterval) {
          clearInterval(pollInterval);
          pollInterval = null;
        }
      }, 10000);
    }
  });

  onUnmounted(() => {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
  });
}
