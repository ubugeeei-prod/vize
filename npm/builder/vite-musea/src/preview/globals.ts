import type { ResolvedMuseaToolbarControl } from "../toolbar.js";

/** Generated only for configured controls; the default preview stays unchanged. */
export function generatePreviewGlobals(toolbar: ResolvedMuseaToolbarControl[]): string {
  if (toolbar.length === 0) return "";
  return `
const __museaToolbar = ${JSON.stringify(toolbar)};
function __museaResolveGlobals(input) {
  const record = input && typeof input === 'object' ? input : {};
  return Object.fromEntries(__museaToolbar.map((control) => {
    const value = Object.hasOwn(record, control.id) ? record[control.id] : undefined;
    return [control.id, control.options.some((option) => option.value === value) ? value : control.default];
  }));
}
let __museaInitialGlobals;
try {
  __museaInitialGlobals = JSON.parse(new URL(window.location.href).searchParams.get('museaGlobals'));
} catch {}
const __museaGlobals = __museaCreateGlobalsRef(__museaResolveGlobals(__museaInitialGlobals));
function __museaUpdateGlobals(event) {
  if (event.origin !== window.location.origin || event.source !== window.parent) return;
  if (event.data?.type !== 'musea:set-globals') return;
  const next = __museaResolveGlobals(event.data.payload);
  if (__museaToolbar.some((control) => next[control.id] !== __museaGlobals.value[control.id])) {
    __museaGlobals.value = next;
  }
}
window.addEventListener('message', __museaUpdateGlobals);
window.addEventListener('pagehide', (event) => {
  if (!event.persisted) window.removeEventListener('message', __museaUpdateGlobals);
});
`;
}
