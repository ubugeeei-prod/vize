/**
 * useCursor - terminal cursor positioning.
 */

import { inject, onUnmounted, type InjectionKey } from "@vue/runtime-core";

export interface CursorPosition {
  x: number;
  y: number;
}

export interface CursorContext {
  setCursorPosition: (position: CursorPosition | undefined) => void;
}

export const CURSOR_KEY: InjectionKey<CursorContext> = Symbol("fresco-cursor");

async function loadNative() {
  return import("@vizejs/fresco-native");
}

export function createCursorContext(
  setCursorPosition: (position: CursorPosition | undefined) => void,
): CursorContext {
  return {
    setCursorPosition,
  };
}

function setNativeCursorPosition(position: CursorPosition | undefined) {
  void loadNative()
    .then((native) => {
      if (position) {
        native.setCursor(position.x, position.y);
        native.showCursor();
      } else {
        native.hideCursor();
      }
    })
    .catch(() => {
      // Cursor control is best-effort outside a mounted Fresco app.
    });
}

export function useCursor() {
  const context = inject(CURSOR_KEY, null);
  let didSetCursor = false;

  const setCursorPosition = (position: CursorPosition | undefined) => {
    didSetCursor = true;

    if (context) {
      context.setCursorPosition(position);
      return;
    }

    setNativeCursorPosition(position);
  };

  onUnmounted(() => {
    if (didSetCursor) {
      context?.setCursorPosition(undefined);
    }
  });

  return {
    setCursorPosition,
  };
}
