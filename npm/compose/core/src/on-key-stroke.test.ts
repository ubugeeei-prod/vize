import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { onKeyDown, onKeyStroke, onKeyUp } from "./on-key-stroke.ts";
import { eventWith, keyEvent } from "./testing/fake-dom.ts";

void test("matches names, lists, predicates, and any key", () => {
  const target = new EventTarget();
  const seen: string[] = [];
  const scope = effectScope();
  scope.run(() => {
    onKeyStroke("esc", () => seen.push("esc"), { target });
    onKeyStroke(["a", "b"], (event) => seen.push(`list:${event.key}`), { target });
    onKeyStroke(
      (event) => event.key === "z",
      () => seen.push("predicate"),
      { target },
    );
    onKeyStroke(true, () => seen.push("any"), { target, eventName: "keyup" });
  });

  target.dispatchEvent(keyEvent("keydown", "Escape", "Escape"));
  target.dispatchEvent(keyEvent("keydown", "b", "KeyB"));
  target.dispatchEvent(keyEvent("keydown", "z", "KeyZ"));
  target.dispatchEvent(keyEvent("keyup", "q", "KeyQ"));
  assert.deepEqual(seen, ["esc", "list:b", "predicate", "any"]);

  scope.stop();
  target.dispatchEvent(keyEvent("keydown", "Escape", "Escape"));
  assert.equal(seen.length, 4);
});

void test("dedupes repeats and exposes keydown/keyup shorthands", () => {
  const target = new EventTarget();
  const seen: string[] = [];
  const stopDown = onKeyDown("enter", () => seen.push("down"), { target, dedupe: true });
  const stopUp = onKeyUp("enter", () => seen.push("up"), { target });
  target.dispatchEvent(eventWith("keydown", { key: "Enter", code: "Enter", repeat: false }));
  target.dispatchEvent(eventWith("keydown", { key: "Enter", code: "Enter", repeat: true }));
  target.dispatchEvent(keyEvent("keyup", "Enter", "Enter"));
  assert.deepEqual(seen, ["down", "up"]);
  stopDown();
  stopUp();
  target.dispatchEvent(keyEvent("keyup", "Enter", "Enter"));
  assert.deepEqual(seen, ["down", "up"]);
});
