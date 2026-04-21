import { ref, onMounted, onUnmounted } from "vue";

export interface ResizableOptions {
  direction: "horizontal" | "vertical";
  minSize: number;
  maxSize: number | (() => number);
  storageKey?: string;
  defaultSize: number;
  invert?: boolean;
  documentClass?: string;
}

export function useResizable(options: ResizableOptions) {
  const {
    direction,
    minSize,
    maxSize,
    storageKey,
    defaultSize,
    invert = false,
    documentClass,
  } = options;

  const size = ref(defaultSize);
  const isResizing = ref(false);
  const startPos = ref(0);
  const startSize = ref(0);
  let activePointerId: number | null = null;
  let activePointerTarget: HTMLElement | null = null;

  const resolveMaxSize = () =>
    typeof maxSize === "function" ? Math.max(minSize, maxSize()) : Math.max(minSize, maxSize);

  const persistSize = () => {
    if (storageKey) {
      localStorage.setItem(storageKey, String(size.value));
    }
  };

  // Load from localStorage
  onMounted(() => {
    if (storageKey) {
      const saved = localStorage.getItem(storageKey);
      if (saved) {
        const parsed = parseInt(saved, 10);
        if (!isNaN(parsed)) {
          size.value = Math.max(minSize, Math.min(resolveMaxSize(), parsed));
        }
      }
    }
  });

  const startDrag = (clientX: number, clientY: number) => {
    isResizing.value = true;
    startPos.value = direction === "horizontal" ? clientX : clientY;
    startSize.value = size.value;
    if (documentClass) {
      document.body.classList.add(documentClass);
    }
  };

  const updateSize = (clientX: number, clientY: number) => {
    const currentPos = direction === "horizontal" ? clientX : clientY;
    const delta = currentPos - startPos.value;
    let newSize = startSize.value + (invert ? -delta : delta);

    newSize = Math.max(minSize, Math.min(resolveMaxSize(), newSize));
    size.value = newSize;
  };

  const clearPointerCapture = () => {
    if (!activePointerTarget || activePointerId === null) {
      activePointerTarget = null;
      activePointerId = null;
      return;
    }

    if (typeof activePointerTarget.hasPointerCapture === "function") {
      try {
        if (activePointerTarget.hasPointerCapture(activePointerId)) {
          activePointerTarget.releasePointerCapture(activePointerId);
        }
      } catch {
        // Ignore release errors if the pointer is already gone.
      }
    }

    activePointerTarget = null;
    activePointerId = null;
  };

  const clearDocumentClass = () => {
    if (documentClass) {
      document.body.classList.remove(documentClass);
    }
  };

  const clearPointerListeners = () => {
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", onPointerUp);
    window.removeEventListener("pointercancel", onPointerUp);
    window.removeEventListener("blur", stopDrag);
    document.removeEventListener("visibilitychange", onVisibilityChange);
  };

  const restoreDocumentState = () => {
    clearDocumentClass();
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
  };

  const stopDrag = () => {
    if (!isResizing.value) return;
    isResizing.value = false;
    clearPointerCapture();
    clearPointerListeners();
    persistSize();
    restoreDocumentState();
  };

  const onVisibilityChange = () => {
    if (document.visibilityState !== "visible") {
      stopDrag();
    }
  };

  const onMouseDown = (e: MouseEvent) => {
    e.preventDefault();
    startDrag(e.clientX, e.clientY);
    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
    document.body.style.cursor = direction === "horizontal" ? "col-resize" : "row-resize";
    document.body.style.userSelect = "none";
  };

  const onPointerMove = (e: PointerEvent) => {
    if (!isResizing.value) return;
    if (activePointerId !== null && e.pointerId !== activePointerId) return;

    updateSize(e.clientX, e.clientY);
  };

  const onPointerUp = (e?: PointerEvent) => {
    if (e && activePointerId !== null && e.pointerId !== activePointerId) return;
    stopDrag();
  };

  const onPointerDown = (e: PointerEvent) => {
    e.preventDefault();

    if (e.currentTarget instanceof HTMLElement) {
      try {
        e.currentTarget.setPointerCapture(e.pointerId);
        activePointerTarget = e.currentTarget;
        activePointerId = e.pointerId;
      } catch {
        activePointerTarget = null;
        activePointerId = null;
      }
    } else {
      activePointerTarget = null;
      activePointerId = null;
    }

    startDrag(e.clientX, e.clientY);
    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
    window.addEventListener("pointercancel", onPointerUp);
    window.addEventListener("blur", stopDrag);
    document.addEventListener("visibilitychange", onVisibilityChange);
    document.body.style.cursor = direction === "horizontal" ? "col-resize" : "row-resize";
    document.body.style.userSelect = "none";
  };

  const onMouseMove = (e: MouseEvent) => {
    if (!isResizing.value) return;
    updateSize(e.clientX, e.clientY);
  };

  const onMouseUp = () => {
    isResizing.value = false;
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("mouseup", onMouseUp);
    persistSize();
    restoreDocumentState();
  };

  onUnmounted(() => {
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("mouseup", onMouseUp);
    stopDrag();
    restoreDocumentState();
  });

  const reset = () => {
    size.value = Math.max(minSize, Math.min(resolveMaxSize(), defaultSize));
    if (storageKey) {
      localStorage.removeItem(storageKey);
    }
  };

  return {
    size,
    isResizing,
    onMouseDown,
    onPointerDown,
    reset,
  };
}

// Composable for the full gallery layout with multiple resizable panels
export function useResizableLayout() {
  const sidebarWidth = useResizable({
    direction: "horizontal",
    minSize: 180,
    maxSize: 400,
    storageKey: "musea-sidebar-width",
    defaultSize: 240,
  });

  const propsWidth = useResizable({
    direction: "horizontal",
    minSize: 200,
    maxSize: 500,
    storageKey: "musea-props-width",
    defaultSize: 280,
  });

  const eventPanelHeight = useResizable({
    direction: "vertical",
    minSize: 100,
    maxSize: 400,
    storageKey: "musea-event-height",
    defaultSize: 200,
  });

  return {
    sidebarWidth,
    propsWidth,
    eventPanelHeight,
  };
}
