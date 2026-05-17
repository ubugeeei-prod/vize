/**
 * useCursor - terminal cursor positioning.
 */

export interface CursorPosition {
  x: number;
  y: number;
}

async function loadNative() {
  return import("@vizejs/fresco-native");
}

export function useCursor() {
  return {
    setCursorPosition: (position: CursorPosition | undefined) => {
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
    },
  };
}
