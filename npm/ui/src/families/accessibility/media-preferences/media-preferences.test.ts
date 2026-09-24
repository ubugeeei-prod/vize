import assert from "node:assert/strict";

import { afterEach, beforeEach, test } from "vite-plus/test";
import { defineComponent, effectScope, h, nextTick } from "vue";
import type { ComputedRef } from "vue";

import type { MediaPreferences, MediaPreferencesProviderExpose } from "./media-preferences.ts";
import MediaPreferencesProvider from "./media-preferences-provider.vue";
import {
  createMediaPreferencesTracker,
  mergeMediaPreferences,
  defaultMediaPreferences,
  readMediaPreferences,
  useForcedColors,
  useMediaPreferences,
  useMediaPreferencesTracker,
  usePrefersColorScheme,
  usePrefersContrast,
  usePrefersReducedMotion,
  usePrefersReducedTransparency,
} from "./media-preferences-runtime.ts";
import { mountInteraction } from "../../../testing/mount.ts";

/** Controllable `matchMedia` double keyed by exact query text. */
class FakeMediaEnvironment {
  readonly active = new Set<string>();
  readonly listeners = new Map<string, Set<() => void>>();

  matchMedia = (query: string) => ({
    addEventListener: (_type: string, listener: () => void) => {
      const set = this.listeners.get(query) ?? new Set();
      set.add(listener);
      this.listeners.set(query, set);
    },
    matches: this.active.has(query),
    media: query,
    removeEventListener: (_type: string, listener: () => void) => {
      this.listeners.get(query)?.delete(listener);
    },
  });

  set(query: string, on: boolean): void {
    if (on) this.active.add(query);
    else this.active.delete(query);
    for (const listener of this.listeners.get(query) ?? []) listener();
  }

  listenerCount(): number {
    let count = 0;
    for (const set of this.listeners.values()) count += set.size;
    return count;
  }
}

let media = new FakeMediaEnvironment();
const originalMatchMedia = window.matchMedia;

beforeEach(() => {
  media = new FakeMediaEnvironment();
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    value: media.matchMedia,
    writable: true,
  });
});

afterEach(() => {
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    value: originalMatchMedia,
    writable: true,
  });
});

function mountProvider(props: Record<string, unknown> = {}) {
  return mountInteraction(MediaPreferencesProvider, {
    props,
    slots: {
      default: (state: MediaPreferences) =>
        h("output", `${state.reducedMotion ? "calm" : "lively"} ${state.colorScheme}`),
    },
  });
}

test("provider publishes detected preferences as data attributes after mount", async () => {
  media.set("(prefers-reduced-motion: reduce)", true);
  media.set("(prefers-color-scheme: dark)", true);
  media.set("(prefers-contrast: more)", true);
  const handle = mountProvider();
  await nextTick();
  const root = handle.root();

  assert.equal(root.getAttribute("data-vize-ui"), "media-preferences-provider");
  assert.equal(root.getAttribute("data-reduced-motion"), "true");
  assert.equal(root.getAttribute("data-color-scheme"), "dark");
  assert.equal(root.getAttribute("data-prefers-contrast"), "more");
  assert.equal(root.hasAttribute("data-forced-colors"), false);
  assert.equal(root.hasAttribute("data-reduced-transparency"), false);
  assert.equal(root.textContent, "calm dark");

  handle.unmount();
});

test("provider follows media query change events and cleans up listeners", async () => {
  const handle = mountProvider();
  await nextTick();
  const root = handle.root();
  assert.ok(media.listenerCount() > 0);

  media.set("(forced-colors: active)", true);
  media.set("(prefers-reduced-transparency: reduce)", true);
  media.set("(prefers-contrast: less)", true);
  media.set("(prefers-color-scheme: light)", true);
  await nextTick();
  assert.equal(root.getAttribute("data-forced-colors"), "true");
  assert.equal(root.getAttribute("data-reduced-transparency"), "true");
  assert.equal(root.getAttribute("data-prefers-contrast"), "less");
  assert.equal(root.getAttribute("data-color-scheme"), "light");

  handle.unmount();
  assert.equal(media.listenerCount(), 0, "unmount removes every change listener");
});

test("force overrides detection and initial is only a pre-detection hint", async () => {
  media.set("(prefers-reduced-motion: reduce)", true);
  const handle = mountProvider({
    force: { reducedMotion: false, colorScheme: "dark" },
    initial: { contrast: "more" },
  });
  await nextTick();
  const root = handle.root();

  assert.equal(root.hasAttribute("data-reduced-motion"), false, "force wins over the OS");
  assert.equal(root.getAttribute("data-color-scheme"), "dark");
  assert.equal(
    root.getAttribute("data-prefers-contrast"),
    "no-preference",
    "detection replaces hints",
  );

  await handle.wrapper.setProps({ force: undefined });
  assert.equal(root.getAttribute("data-reduced-motion"), "true");

  handle.unmount();
});

test("composables read the provider context", async () => {
  const seen: Record<string, ComputedRef<unknown>> = {};
  const Reader = defineComponent({
    name: "MediaPreferencesReader",
    setup() {
      seen.motion = usePrefersReducedMotion();
      seen.transparency = usePrefersReducedTransparency();
      seen.forced = useForcedColors();
      seen.contrast = usePrefersContrast();
      seen.scheme = usePrefersColorScheme();
      return () => h("span");
    },
  });
  const handle = mountInteraction(MediaPreferencesProvider, {
    props: { force: { forcedColors: true, reducedMotion: true } },
    slots: { default: () => h(Reader) },
  });
  await nextTick();

  assert.equal(seen.motion?.value, true);
  assert.equal(seen.forced?.value, true);
  assert.equal(seen.transparency?.value, false);
  assert.equal(seen.contrast?.value, "no-preference");
  assert.equal(seen.scheme?.value, "no-preference");
  assert.equal(media.listenerCount() > 0, true, "only the provider subscribes");

  handle.unmount();
});

test("standalone composables subscribe after mount and in bare effect scopes", async () => {
  media.set("(prefers-reduced-motion: reduce)", true);
  let beforeMount: boolean | null = null;
  let motion: ComputedRef<boolean> | null = null;
  const Standalone = defineComponent({
    name: "MediaPreferencesStandalone",
    setup() {
      motion = usePrefersReducedMotion();
      beforeMount = motion.value;
      return () => h("span", String(motion?.value));
    },
  });
  const handle = mountInteraction(Standalone);
  await nextTick();
  assert.equal(beforeMount, false, "setup renders the SSR-safe default");
  assert.equal(handle.root().textContent, "true");
  handle.unmount();

  const scope = effectScope();
  const preferences = scope.run(() => useMediaPreferences());
  assert.equal(preferences?.value.reducedMotion, true, "effect scopes start immediately");
  scope.stop();
  assert.equal(media.listenerCount(), 0);
  assert.throws(() => useMediaPreferencesTracker(), /VIZE_UI_MEDIA_PREFERENCES_SETUP/);
});

test("tracker helpers read, merge, stop, and dispose", () => {
  media.set("(prefers-contrast: custom)", true);
  assert.equal(readMediaPreferences(window).contrast, "custom");
  assert.deepEqual(
    mergeMediaPreferences(defaultMediaPreferences, { reducedMotion: true }, undefined, {
      colorScheme: "light",
    }),
    { ...defaultMediaPreferences, colorScheme: "light", reducedMotion: true },
  );

  const tracker = createMediaPreferencesTracker({ initial: { colorScheme: "dark" } });
  assert.equal(tracker.preferences.value.colorScheme, "dark");
  assert.equal(tracker.started.value, false);
  tracker.start(window);
  assert.equal(tracker.started.value, true);
  assert.equal(tracker.preferences.value.contrast, "custom");
  tracker.stop();
  assert.equal(media.listenerCount(), 0);
  tracker.start(null);
  assert.equal(tracker.started.value, false);
  tracker.dispose();
  assert.throws(() => tracker.start(window), /VIZE_UI_MEDIA_PREFERENCES_DISPOSED/);
});

test("provider exposes effective preferences", async () => {
  let exposed: MediaPreferencesProviderExpose | null = null;
  const Probe = defineComponent({
    name: "MediaPreferencesExposeProbe",
    setup: () => () =>
      h(MediaPreferencesProvider, {
        force: { reducedTransparency: true },
        ref: (value) => {
          exposed = value as MediaPreferencesProviderExpose | null;
        },
      }),
  });
  const handle = mountInteraction(Probe);
  await nextTick();
  if (exposed === null) assert.fail("MediaPreferencesProvider must expose its state");
  const api: MediaPreferencesProviderExpose = exposed;
  assert.equal(api.reducedTransparency, true);
  assert.equal(api.reducedMotion, false);
  assert.ok(api.element instanceof HTMLDivElement);
  handle.unmount();
});
