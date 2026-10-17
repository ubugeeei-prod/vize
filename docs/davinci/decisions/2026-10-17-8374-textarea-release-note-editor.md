# Growing release-note editor

Issue: #8374. One missing practical composable example; the wider audit remains open.
Depends on #9302, which corrects autosizing with normal textarea padding and borders.

Use the public `textarea-autosize` entry in one local release-note editor. The
returned input binds directly with v-model. Native editing and programmatic
insert/clear exercise the same post-render watcher. A compact-width control
exercises the real ResizeObserver, preserving the draft and native selection.
The draft is explicitly in memory only; it is neither saved nor submitted.

Keep the default height mode. Ordinary CSS caps the editor at 12rem and allows
scrolling; min-height mode can override a smaller maximum and is explained as a
separate sizing choice. No invented minimum/maximum options or padding removal
workarounds are needed. Declared rows remain the server fallback.

Reuse the existing complete-SFC source identity, strict SSR/hydration check,
source/dist browser campaigns and desktop/mobile captures. Native checks cover
multiline growth without clipped padding, widening/shrinking, clear with control
focus, retained selection, and keyboard navigation to the last line at the cap.
After screenshots, unmount the actual hydrated app before resizing the viewport.
The product regression tests separately prove observer and watcher disposal.
No browser hosts, readonly geometry, hydration gates or public APIs are changed.
