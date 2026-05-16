import { watch, onUnmounted, type WatchSource } from "vue";

/**
 * A watch that debounces the callback and cleans up the timer on unmount.
 */
export function useDebouncedWatch<T>(
  source: WatchSource<T> | WatchSource<T>[],
  callback: () => void,
  options: { delay?: number; deep?: boolean; immediate?: boolean } = {},
) {
  const { delay = 300, deep = false, immediate = false } = options;
  let timer: ReturnType<typeof setTimeout> | null = null;

  watch(
    source as WatchSource<T>,
    () => {
      if (timer) clearTimeout(timer);
      timer = setTimeout(callback, delay);
    },
    { deep, immediate },
  );

  onUnmounted(() => {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
  });
}
