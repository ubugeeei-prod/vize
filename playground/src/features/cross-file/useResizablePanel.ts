import { ref, computed } from "vue";

export function useResizablePanel() {
  const sidebarWidth = ref(220);
  const diagnosticsWidth = ref(320);
  const isResizingSidebar = ref(false);
  const isResizingDiagnostics = ref(false);
  const containerRef = ref<HTMLElement | null>(null);

  function startSidebarResize(e: MouseEvent) {
    isResizingSidebar.value = true;
    e.preventDefault();
    document.addEventListener("mousemove", onSidebarResize);
    document.addEventListener("mouseup", stopResize);
  }

  function startDiagnosticsResize(e: MouseEvent) {
    isResizingDiagnostics.value = true;
    e.preventDefault();
    document.addEventListener("mousemove", onDiagnosticsResize);
    document.addEventListener("mouseup", stopResize);
  }

  function onSidebarResize(e: MouseEvent) {
    if (!isResizingSidebar.value || !containerRef.value) return;
    const containerRect = containerRef.value.getBoundingClientRect();
    const newWidth = Math.max(150, Math.min(400, e.clientX - containerRect.left));
    sidebarWidth.value = newWidth;
  }

  function onDiagnosticsResize(e: MouseEvent) {
    if (!isResizingDiagnostics.value || !containerRef.value) return;
    const containerRect = containerRef.value.getBoundingClientRect();
    const newWidth = Math.max(200, Math.min(500, containerRect.right - e.clientX));
    diagnosticsWidth.value = newWidth;
  }

  function stopResize() {
    isResizingSidebar.value = false;
    isResizingDiagnostics.value = false;
    document.removeEventListener("mousemove", onSidebarResize);
    document.removeEventListener("mousemove", onDiagnosticsResize);
    document.removeEventListener("mouseup", stopResize);
  }

  const gridStyle = computed(() => ({
    gridTemplateColumns: `${sidebarWidth.value}px 4px 1fr 4px ${diagnosticsWidth.value}px`,
  }));

  return {
    containerRef,
    isResizingSidebar,
    isResizingDiagnostics,
    gridStyle,
    startSidebarResize,
    startDiagnosticsResize,
  };
}
