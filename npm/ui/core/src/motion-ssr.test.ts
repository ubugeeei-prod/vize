import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  motionTokenVar,
  startViewTransition,
  supportsScrollDrivenAnimations,
  supportsStartingStyle,
  supportsViewTransitions,
  useReducedMotion,
} from "./motion.ts";

/** A consumer that touches every adapter during setup, exactly like real SSR. */
const SsrProbe = defineComponent({
  name: "MotionSsrProbe",
  setup() {
    const reduced = useReducedMotion();
    const handle = startViewTransition(() => undefined);
    return () =>
      h(
        "output",
        {
          "data-vize-motion": "enter",
          "data-reduced": String(reduced.value),
          "data-native": String(handle.native),
          "data-ease": motionTokenVar("ease-standard"),
        },
        "Motion",
      );
  },
});

/** Render with every platform global a server lacks removed. */
async function renderWithoutPlatformGlobals(): Promise<string> {
  const originalMatchMedia = globalThis.matchMedia;
  const originalCss = globalThis.CSS;
  // @ts-expect-error simulating a server runtime without matchMedia.
  delete globalThis.matchMedia;
  // @ts-expect-error simulating a server runtime without the CSS namespace.
  delete globalThis.CSS;
  try {
    return await renderToString(createSSRApp(SsrProbe));
  } finally {
    globalThis.matchMedia = originalMatchMedia;
    globalThis.CSS = originalCss;
  }
}

test("renders byte-identical SSR markup without platform globals", async () => {
  const outputs = [await renderWithoutPlatformGlobals(), await renderWithoutPlatformGlobals()];
  assert.equal(outputs[0], outputs[1]);
  assert.equal(
    outputs[0],
    '<output data-vize-motion="enter" data-reduced="false" data-native="false"' +
      ' data-ease="var(--vize-ui-motion-ease-standard)">Motion</output>',
  );
});

test("probes report no support without platform globals", async () => {
  const originalMatchMedia = globalThis.matchMedia;
  const originalCss = globalThis.CSS;
  // @ts-expect-error simulating a server runtime without matchMedia.
  delete globalThis.matchMedia;
  // @ts-expect-error simulating a server runtime without the CSS namespace.
  delete globalThis.CSS;
  try {
    assert.equal(supportsScrollDrivenAnimations(), false);
    assert.equal(supportsStartingStyle(), false);
    assert.equal(supportsViewTransitions(), false);
    let ran = false;
    await startViewTransition(() => {
      ran = true;
    }).finished;
    assert.ok(ran, "the server fallback must still run the update");
  } finally {
    globalThis.matchMedia = originalMatchMedia;
    globalThis.CSS = originalCss;
  }
});

test("hydrates a motion consumer without replacement or diagnostics", async () => {
  const serverHtml = await renderWithoutPlatformGlobals();
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverTarget = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);

  try {
    app.mount(host);
    assert.equal(host.firstElementChild, serverTarget);
    assert.equal(host.firstElementChild?.getAttribute("data-vize-motion"), "enter");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
