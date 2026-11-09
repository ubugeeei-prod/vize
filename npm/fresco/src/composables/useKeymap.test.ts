import assert from "node:assert/strict";
import { test } from "vitest";
import { defineComponent, h, ref } from "vue";

import { dispatchKey, mountFresco } from "../testing/mount.js";
import { createKeymapResolver, defineKeymap, useKeymap } from "./useKeymap.js";

const navigationKeymap = defineKeymap([
  {
    command: "save",
    sequence: [
      { key: "x", ctrl: true },
      { key: "s", ctrl: true },
    ],
  },
  { command: "top", sequence: [{ key: "g" }, { key: "g" }] },
  { command: "close", sequence: [{ key: "escape" }] },
] as const);

test("resolver matches exact modifiers and retries after a broken sequence", () => {
  const resolver = createKeymapResolver(() => navigationKeymap);
  assert.deepEqual(resolver.feed({ key: "x", ctrl: true }), { pending: true });
  assert.equal(resolver.feed({ key: "s", ctrl: true }).command, "save");
  assert.deepEqual(resolver.pendingSequence, []);

  assert.deepEqual(resolver.feed({ key: "g" }), { pending: true });
  assert.deepEqual(resolver.feed({ key: "g", shift: true }), { pending: false });
  assert.equal(resolver.feed({ key: "esc" }).command, "close");
});

test("defineKeymap rejects duplicate normalized sequences", () => {
  assert.throws(
    () =>
      defineKeymap([
        { command: "first", sequence: [{ key: "Esc" }] },
        { command: "second", sequence: [{ key: "escape" }] },
      ]),
    /duplicate keymap sequence: escape/,
  );
});

test("useKeymap owns Emacs chords and Vim sequences while active", async () => {
  const active = ref(true);
  const commands: string[] = [];
  let pending: ReturnType<typeof useKeymap>["pendingSequence"] | undefined;
  const Probe = defineComponent({
    setup() {
      const keymap = useKeymap({
        keymap: navigationKeymap,
        active,
        onCommand: (command) => commands.push(command),
      });
      pending = keymap.pendingSequence;
      return () => h("text", { text: commands.join(",") });
    },
  });
  const mounted = mountFresco(() => h(Probe));

  await dispatchKey({ key: "x", ctrl: true });
  assert.equal(pending?.value.length, 1);
  await dispatchKey({ key: "s", ctrl: true });
  await dispatchKey({ char: "g" });
  await dispatchKey({ char: "g" });
  assert.deepEqual(commands, ["save", "top"]);

  active.value = false;
  await dispatchKey({ key: "escape" });
  assert.deepEqual(commands, ["save", "top"]);
  mounted.unmount();
});
