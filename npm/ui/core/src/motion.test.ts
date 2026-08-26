import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
// Paths are resolved from the package cwd: the runner virtualizes import.meta.url.
import path from "node:path";

import { test } from "vite-plus/test";
import { defineComponent, effectScope, h, nextTick } from "vue";

import { mountInteraction } from "./testing/mount.ts";
import {
  motionDelays,
  motionDurations,
  motionEasings,
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

const stylesheet = await readFile(path.resolve("dist/style.css"), "utf8");

/** Value of one custom property in the packaged stylesheet. */
function shippedToken(name: string): string {
  const pattern = new RegExp(`${name.replaceAll(/[-]/g, "\\-")}:([^;}]+)[;}]`);
  const match = pattern.exec(stylesheet);
  assert.ok(match?.[1], `dist/style.css must define ${name}`);
  return match[1];
}

/** CSS time literal in milliseconds, tolerant of minifier unit rewrites. */
function timeToMs(value: string): number {
  const trimmed = value.trim();
  if (trimmed.endsWith("ms")) return Number.parseFloat(trimmed);
  assert.ok(trimmed.endsWith("s"), `${value} must be a CSS time`);
  return Number.parseFloat(trimmed) * 1_000;
}

/** Normalized comparison form, tolerant of minifier whitespace and zero trims. */
function normalizeCss(value: string): string {
  return value.replaceAll(/\s+/g, "").replaceAll("0.", ".");
}

test("ships layered zero-specificity motion tokens matching the mirrors", () => {
  assert.match(stylesheet, /@layer vize\.ui\{/);
  assert.match(stylesheet, /:where\(:root,:host\)\{--vize-ui-motion-duration-instant:/);

  for (const [token, value] of Object.entries(motionDurations)) {
    assert.equal(timeToMs(shippedToken(`--vize-ui-motion-duration-${token}`)), timeToMs(value));
  }
  for (const [token, value] of Object.entries(motionDelays)) {
    assert.equal(timeToMs(shippedToken(`--vize-ui-motion-delay-${token}`)), timeToMs(value));
  }
  for (const [token, value] of Object.entries(motionEasings)) {
    assert.equal(normalizeCss(shippedToken(`--vize-ui-motion-ease-${token}`)), normalizeCss(value));
  }

  // Recipe hooks resolve through the base scales so one override retunes a phase.
  assert.equal(
    shippedToken("--vize-ui-motion-enter-easing"),
    "var(--vize-ui-motion-ease-decelerate)",
  );
  assert.equal(
    shippedToken("--vize-ui-motion-exit-duration"),
    "var(--vize-ui-motion-duration-fast)",
  );
});

test("pairs enter and exit recipes with presence and transition hooks", () => {
  for (const recipe of ["fade", "scale", "slide"] as const) {
    assert.match(
      stylesheet,
      new RegExp(
        `:where\\(\\[data-vize-motion~=${recipe}\\]\\):where\\(\\[data-vize-presence=entering\\],` +
          `\\[data-vize-transition=entering\\]\\)\\{animation-name:vize-ui-motion-${recipe}-in\\}`,
      ),
    );
    assert.match(
      stylesheet,
      new RegExp(
        `:where\\(\\[data-vize-motion~=${recipe}\\]\\):where\\(\\[data-vize-presence=exiting\\],` +
          `\\[data-vize-transition=exiting\\]\\)\\{animation-name:vize-ui-motion-${recipe}-out\\}`,
      ),
    );
    assert.match(stylesheet, new RegExp(`@keyframes vize-ui-motion-${recipe}-in\\{`));
    assert.match(stylesheet, new RegExp(`@keyframes vize-ui-motion-${recipe}-out\\{`));
  }

  // Shared enter/exit timing reads the recipe hooks, never literal durations.
  assert.match(
    stylesheet,
    /\[data-vize-transition=entering\]\)\{animation-duration:var\(--vize-ui-motion-enter-duration\);animation-timing-function:var\(--vize-ui-motion-enter-easing\);animation-fill-mode:both\}/,
  );
  assert.match(
    stylesheet,
    /\[data-vize-transition=exiting\]\)\{animation-duration:var\(--vize-ui-motion-exit-duration\);animation-timing-function:var\(--vize-ui-motion-exit-easing\);animation-fill-mode:both\}/,
  );
});

test("ships move and emphasis recipes with token-driven timing", () => {
  assert.match(
    stylesheet,
    /:where\(\[data-vize-motion~=move\]\)\{transition-property:translate,transform,inset-block-start,inset-block-end,inset-inline-start,inset-inline-end;transition-duration:var\(--vize-ui-motion-move-duration\);transition-timing-function:var\(--vize-ui-motion-move-easing\)\}/,
  );
  assert.match(
    stylesheet,
    /:where\(\[data-vize-motion~=pulse\]\)\{animation:vize-ui-motion-pulse var\(--vize-ui-motion-emphasis-duration\) var\(--vize-ui-motion-emphasis-easing\)\}/,
  );
  assert.match(stylesheet, /@keyframes vize-ui-motion-shake\{/);
  // Document-level view transitions inherit the shared tokens.
  assert.match(
    stylesheet,
    /::view-transition-old\(root\)\{animation-duration:var\(--vize-ui-motion-duration-base\)/,
  );
});

test("ships the starting-style and scroll-driven recipes verbatim", () => {
  assert.match(
    stylesheet,
    /:where\(\[data-vize-motion~=enter\]\)\{transition-property:opacity,translate,scale;transition-duration:var\(--vize-ui-motion-enter-duration\)/,
  );
  assert.match(
    stylesheet,
    /@starting-style\{:where\(\[data-vize-motion~=enter\]\)\{opacity:0;translate:0 var\(--vize-ui-motion-slide-distance\)\}\}/,
  );
  // Both at-features are newer than the floor and must pass through unlowered.
  // The authored `entry 0% entry 100%` range minifies to its `entry` shorthand.
  assert.match(
    stylesheet,
    /:where\(\[data-vize-motion~=reveal\]\)\{animation:vize-ui-motion-fade-in var\(--vize-ui-motion-ease-standard\) both;animation-timeline:view\(\);animation-range:entry\}/,
  );
});

test("zeroes packaged motion under reduced motion", () => {
  const start = stylesheet.indexOf("@media (prefers-reduced-motion:reduce)");
  assert.ok(start >= 0, "the reduced-motion policy block must ship");
  const block = stylesheet.slice(start, stylesheet.indexOf("@media (forced-colors:active)"));

  for (const token of Object.keys(motionDurations)) {
    assert.match(block, new RegExp(`--vize-ui-motion-duration-${token}:0s`));
  }
  for (const token of Object.keys(motionDelays)) {
    assert.match(block, new RegExp(`--vize-ui-motion-delay-${token}:0s`));
  }
  assert.match(
    block,
    /:where\(\[data-vize-motion\]\)\{transition-duration:0s;animation-duration:0s\}/,
  );
  // Timeline-driven animations ignore zeroed durations, so reveal stands down.
  assert.match(block, /:where\(\[data-vize-motion~=reveal\]\)\{animation:none\}/);
});

test("stands down under forced colors", () => {
  const start = stylesheet.indexOf("@media (forced-colors:active)");
  assert.ok(start >= 0, "the forced-colors policy block must ship");
  assert.match(
    stylesheet.slice(start),
    /:where\(\[data-vize-motion\]\)\{transition-property:none;animation-name:none\}/,
  );
});

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
