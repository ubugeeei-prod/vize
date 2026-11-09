import assert from "node:assert/strict";
import { h } from "vue";
import { test } from "vitest";

import type { FrescoRoutedInputEvent } from "./renderer.js";
import { defaultInputAction, routeFocusedInput } from "./focusRouting.js";
import { firstChild, mountFresco } from "./testing/mount.js";

test("routes focused input through the registered host path", () => {
  const calls: string[] = [];
  const mounted = mountFresco(() =>
    h(
      "box",
      {
        onKeyCapture: (event: FrescoRoutedInputEvent) => calls.push(`root:${event.phase}`),
      },
      [
        h("input", {
          focusId: "editor",
          onKey: (event: FrescoRoutedInputEvent) => {
            calls.push(`editor:${event.phase}`);
            event.preventDefault();
          },
        }),
      ],
    ),
  );

  const result = routeFocusedInput(mounted.renderer, mounted.root, "editor", {
    type: "key",
    key: "tab",
    ctrl: false,
    alt: false,
    shift: false,
    meta: false,
    super: false,
    hyper: false,
    capsLock: false,
    numLock: false,
  });

  assert.deepEqual(calls, ["root:capture", "editor:target"]);
  assert.equal(result?.target, firstChild(mounted).children[0]);
  assert.equal(result?.defaultPrevented, true);
  mounted.unmount();
});

test("does not dispatch without a live focused host", () => {
  const mounted = mountFresco(() => h("box", { focusId: "present" }));

  assert.equal(
    routeFocusedInput(mounted.renderer, mounted.root, undefined, {
      type: "focus",
      focused: true,
    }),
    undefined,
  );
  assert.equal(
    routeFocusedInput(mounted.renderer, mounted.root, "missing", {
      type: "focus",
      focused: true,
    }),
    undefined,
  );
  mounted.unmount();
});

test("focused handlers can prevent application fallback actions", () => {
  const tab = { eventType: "key", key: "tab" };
  const ctrlC = { eventType: "key", char: "c", modifiers: { ctrl: true } };

  assert.equal(defaultInputAction(tab, undefined, true), "focus-next");
  assert.equal(
    defaultInputAction({ eventType: "key", key: "backtab" }, undefined, true),
    "focus-previous",
  );
  assert.equal(defaultInputAction(ctrlC, undefined, true), "exit");
  assert.equal(defaultInputAction(ctrlC, undefined, false), undefined);
  assert.equal(defaultInputAction(tab, { defaultPrevented: true }, true), undefined);
});
