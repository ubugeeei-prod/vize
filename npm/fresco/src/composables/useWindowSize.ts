/**
 * useWindowSize - terminal dimensions.
 */

import { reactive } from "vue";
import { useApp } from "./useApp.js";

/** Reactive terminal dimensions exposed as column and row getters. */
export interface WindowSize {
  columns: number;
  rows: number;
}

/** Read application dimensions through reactive column and row getters. */
export function useWindowSize() {
  const { width, height } = useApp();

  return reactive({
    get columns() {
      return width.value;
    },
    get rows() {
      return height.value;
    },
  }) as WindowSize;
}
