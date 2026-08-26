import assert from "node:assert/strict";
import { test } from "vite-plus/test";
import { defineComponent, effectScope, h, nextTick } from "vue";

import { mountInteraction } from "./testing/mount.ts";
import {
  motionTokenProperty,
  motionTokenVar,
  prefersReducedMotion,
  setMotionTokens,
  startViewTransition,
  supportsScrollDrivenAnimations,
  supportsStartingStyle,
  supportsViewTransitions,
  useReducedMotion,
} from "./motion.ts";

interface FakeNativeTransition {
  finished: Promise<void>;
  ready: Promise<void>;
  updateCallbackDone: Promise<void>;
  skipTransition(): void;
}

type MutableViewTransitionDocument = Document & {
  startViewTransition?: (update: () => Promise<void>) => FakeNativeTransition;
};

test("applies and restores token overrides on a real element", () => {
  const element = document.createElement("section");
  document.body.append(element);
  element.style.setProperty("--vize-ui-motion-duration-base", "999ms");

  const restore = setMotionTokens(element, {
    "duration-base": "80ms",
    "ease-standard": "linear",
    "slide-distance": "16px",
  });
  assert.equal(element.style.getPropertyValue("--vize-ui-motion-duration-base"), "80ms");
  assert.equal(element.style.getPropertyValue("--vize-ui-motion-ease-standard"), "linear");
  assert.equal(element.style.getPropertyValue("--vize-ui-motion-slide-distance"), "16px");

  restore();
  assert.equal(element.style.getPropertyValue("--vize-ui-motion-duration-base"), "999ms");
  assert.equal(element.style.getPropertyValue("--vize-ui-motion-ease-standard"), "");
  element.remove();

  assert.equal(motionTokenProperty("ease-emphasized"), "--vize-ui-motion-ease-emphasized");
  assert.equal(motionTokenVar("duration-fast"), "var(--vize-ui-motion-duration-fast)");
});

test("rejects unknown tokens and empty override values", () => {
  const element = document.createElement("div");
  assert.throws(
    () => motionTokenProperty("duration-bogus" as Parameters<typeof motionTokenProperty>[0]),
    /VIZE_UI_MOTION_TOKEN/,
  );
  assert.throws(() => setMotionTokens(element, { "duration-base": "  " }), /VIZE_UI_MOTION_TOKEN/);
  assert.throws(
    () => setMotionTokens(null as unknown as HTMLElement, { "duration-base": "1ms" }),
    /VIZE_UI_MOTION_TOKEN/,
  );
});

test("falls back synchronously without native view transitions", async () => {
  const doc = document as MutableViewTransitionDocument;
  assert.equal(doc.startViewTransition, undefined, "happy-dom must not implement the API");
  assert.equal(supportsViewTransitions(), false);

  let ran = false;
  const handle = startViewTransition(() => {
    ran = true;
  });
  assert.equal(handle.native, false);
  await handle.finished;
  assert.ok(ran, "the fallback must still run the DOM update");
  assert.equal(handle.ready, handle.updateCallbackDone);
  handle.skipTransition();

  const failing = startViewTransition(() => {
    throw new Error("boom");
  });
  await assert.rejects(failing.updateCallbackDone, /boom/);
});

test("skips the native transition under reduced motion", async () => {
  const doc = document as MutableViewTransitionDocument;
  const originalMatchMedia = globalThis.matchMedia;
  let nativeCalls = 0;
  doc.startViewTransition = () => {
    nativeCalls += 1;
    return {
      finished: Promise.resolve(),
      ready: Promise.resolve(),
      updateCallbackDone: Promise.resolve(),
      skipTransition: () => undefined,
    };
  };
  globalThis.matchMedia = ((query: string) => ({
    matches: query.includes("prefers-reduced-motion"),
    media: query,
    addEventListener() {},
    removeEventListener() {},
  })) as typeof matchMedia;

  try {
    assert.equal(prefersReducedMotion(), true);
    const skipped = startViewTransition(() => undefined);
    assert.equal(skipped.native, false);
    await skipped.finished;
    assert.equal(nativeCalls, 0, "reduced motion must bypass the native transition");

    const forced = startViewTransition(() => undefined, { respectReducedMotion: false });
    assert.equal(forced.native, true);
    await forced.finished;
    assert.equal(nativeCalls, 1, "opting out must reach the native transition");
  } finally {
    delete doc.startViewTransition;
    globalThis.matchMedia = originalMatchMedia;
  }
});

test("drives the native view transition when supported", async () => {
  const doc = document as MutableViewTransitionDocument;
  let skipped = 0;
  let updateRan = false;
  doc.startViewTransition = (update) => {
    const updateCallbackDone = update();
    return {
      finished: updateCallbackDone,
      ready: Promise.reject(new Error("transition skipped")),
      updateCallbackDone,
      skipTransition: () => {
        skipped += 1;
      },
    };
  };

  try {
    assert.equal(supportsViewTransitions(), true);
    const handle = startViewTransition(() => {
      updateRan = true;
    });
    assert.equal(handle.native, true);
    await handle.finished;
    await handle.updateCallbackDone;
    assert.ok(updateRan, "the native path must run the update callback");
    await assert.rejects(handle.ready, /transition skipped/);
    handle.skipTransition();
    assert.equal(skipped, 1, "skipTransition must delegate to the native transition");
  } finally {
    delete doc.startViewTransition;
  }
});

test("rejects invalid view transition input", () => {
  assert.throws(
    () => startViewTransition("nope" as unknown as () => void),
    /VIZE_UI_MOTION_OPTION/,
  );
  assert.throws(
    () =>
      startViewTransition(() => undefined, {
        respectReducedMotion: "always" as unknown as boolean,
      }),
    /VIZE_UI_MOTION_OPTION/,
  );
});

test("tracks reduced motion reactively in a mounted consumer", async () => {
  const originalMatchMedia = globalThis.matchMedia;
  let listener: ((event: { matches: boolean }) => void) | undefined;
  let removals = 0;
  globalThis.matchMedia = ((query: string) => ({
    matches: false,
    media: query,
    addEventListener: (_type: string, callback: (event: { matches: boolean }) => void) => {
      listener = callback;
    },
    removeEventListener: () => {
      removals += 1;
    },
  })) as unknown as typeof matchMedia;

  const Probe = defineComponent({
    name: "MotionReducedProbe",
    setup() {
      const reduced = useReducedMotion();
      return () => h("output", { "data-reduced": String(reduced.value) }, "policy");
    },
  });

  try {
    const handle = mountInteraction(Probe);
    assert.equal(handle.root().getAttribute("data-reduced"), "false");
    assert.ok(listener, "useReducedMotion must subscribe to media changes");
    listener({ matches: true });
    await nextTick();
    assert.equal(handle.root().getAttribute("data-reduced"), "true");
    handle.unmount();
    assert.equal(removals, 1, "unmounting must release the media listener");
  } finally {
    globalThis.matchMedia = originalMatchMedia;
  }
});

test("rejects composable use outside an effect scope", () => {
  assert.throws(() => useReducedMotion(), /VIZE_UI_MOTION_SETUP/);
  const scope = effectScope();
  const reduced = scope.run(() => useReducedMotion());
  assert.equal(typeof reduced?.value, "boolean");
  scope.stop();
});

test("probes platform support without throwing", () => {
  assert.equal(typeof supportsStartingStyle(), "boolean");
  assert.equal(typeof supportsScrollDrivenAnimations(), "boolean");
  assert.equal(typeof supportsViewTransitions(), "boolean");
  assert.equal(typeof prefersReducedMotion(), "boolean");
});
