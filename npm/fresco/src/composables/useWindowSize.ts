/**
 * useWindowSize - terminal dimensions.
 */

import { reactive } from "@vue/runtime-core";
import { useApp } from "./useApp.js";

export interface WindowSize {
  columns: number;
  rows: number;
}

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
