import { shallowRef, type Directive } from "vue";
import type { Router } from "vue-router";
import {
  MUSEA_GLOBALS_QUERY,
  parseToolbarGlobals,
  resolveToolbarGlobals,
  type MuseaGlobalValue,
  type ResolvedMuseaToolbarControl,
} from "../../src/toolbar.js";
import { sendMessage } from "./usePostMessage";

const config = window as unknown as {
  __MUSEA_TOOLBAR__?: ResolvedMuseaToolbarControl[];
  __MUSEA_BASE_PATH__?: string;
};
export const toolbar = config.__MUSEA_TOOLBAR__ ?? [];
const storageKey = `musea:globals:${config.__MUSEA_BASE_PATH__ ?? "/__musea__"}`;
let persisted: unknown;
if (toolbar.length > 0) {
  try {
    persisted = parseToolbarGlobals(localStorage.getItem(storageKey));
  } catch {
    // Storage may be disabled. URL state and in-memory navigation still work.
  }
}
const urlValue = new URL(window.location.href).searchParams.get(MUSEA_GLOBALS_QUERY);
let snapshot = resolveToolbarGlobals(
  toolbar,
  urlValue === null ? persisted : parseToolbarGlobals(urlValue),
);
const globals = shallowRef(snapshot);
const frames = new Set<HTMLIFrameElement>();
let activeRouter: Router | undefined;

function syncFrame(frame: HTMLIFrameElement) {
  sendMessage(frame, "musea:set-globals", snapshot);
}

function applyGlobals(input: unknown) {
  const next = resolveToolbarGlobals(toolbar, input);
  if (!toolbar.some((control) => next[control.id] !== snapshot[control.id])) return;
  snapshot = next;
  globals.value = next;
  try {
    localStorage.setItem(storageKey, JSON.stringify(next));
  } catch {
    // Persistence is optional; unavailable storage does not prevent updates.
  }
  for (const frame of frames) syncFrame(frame);
}

/** A snapshot read does not subscribe preview URL computations to toolbar edits. */
export function withGlobalsQuery(url: string): string {
  if (toolbar.length === 0) return url;
  const parsed = new URL(url, window.location.origin);
  parsed.searchParams.set(MUSEA_GLOBALS_QUERY, JSON.stringify(snapshot));
  return url.startsWith("/") ? `${parsed.pathname}${parsed.search}${parsed.hash}` : parsed.href;
}

export function useGlobals() {
  function setGlobal(id: string, value: MuseaGlobalValue) {
    applyGlobals({ ...snapshot, [id]: value });
    if (activeRouter) {
      void activeRouter.replace({
        hash: activeRouter.currentRoute.value.hash,
        query: {
          ...activeRouter.currentRoute.value.query,
          [MUSEA_GLOBALS_QUERY]: JSON.stringify(snapshot),
        },
      });
      return;
    }
    const url = new URL(window.location.href);
    url.searchParams.set(MUSEA_GLOBALS_QUERY, JSON.stringify(snapshot));
    // Preserve Vue Router's history state, other query parameters and the hash.
    history.replaceState(history.state, "", url);
  }
  return { toolbar, globals, setGlobal };
}

/** Carry globals through router links while honoring explicit reproducible URLs. */
export function initializeGlobals(router: Router) {
  if (toolbar.length === 0) return;
  activeRouter = router;
  router.beforeEach((to) => {
    const query = to.query[MUSEA_GLOBALS_QUERY];
    if (typeof query === "string") {
      applyGlobals(parseToolbarGlobals(query));
      return;
    }
    return {
      ...to,
      query: { ...to.query, [MUSEA_GLOBALS_QUERY]: JSON.stringify(snapshot) },
    };
  });
}

/** Register each real iframe directly; never scan the gallery DOM on changes. */
export const globalsDirective: Directive<HTMLIFrameElement> = {
  mounted(frame) {
    if (toolbar.length === 0) return;
    frames.add(frame);
    frame.addEventListener("load", onLoad);
  },
  unmounted(frame) {
    frames.delete(frame);
    frame.removeEventListener("load", onLoad);
  },
};

function onLoad(event: Event) {
  syncFrame(event.currentTarget as HTMLIFrameElement);
}
