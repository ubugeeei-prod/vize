import assert from "node:assert/strict";
import { test } from "vitest";
import { defineComponent, h, nextTick, ref } from "vue";

import { firstChild, mountFresco } from "./testing/mount.js";
import {
  defaultFrescoTheme,
  provideFrescoTheme,
  resolveFrescoTheme,
  useFrescoTheme,
} from "./theme.js";

test("resolves partial tokens without mutating static defaults", () => {
  const resolved = resolveFrescoTheme(defaultFrescoTheme, {
    colors: { accent: "yellow" },
    motion: { reduced: true },
  });

  assert.equal(resolved.colors.accent, "yellow");
  assert.equal(resolved.colors.primary, "cyan");
  assert.equal(resolved.motion.reduced, true);
  assert.equal(defaultFrescoTheme.colors.accent, "magenta");
  assert.equal(defaultFrescoTheme.motion.reduced, false);
});

test("inherits reactive scoped overrides through component subtrees", async () => {
  const accent = ref("yellow");
  const ThemedText = defineComponent({
    setup() {
      const theme = useFrescoTheme();
      return () =>
        h("text", {
          text: theme.value.colors.accent,
          reducedMotion: theme.value.motion.reduced,
        });
    },
  });
  const NestedScope = defineComponent({
    setup() {
      provideFrescoTheme({ motion: { reduced: true } });
      return () => h(ThemedText);
    },
  });
  const RootScope = defineComponent({
    setup() {
      provideFrescoTheme(() => ({ colors: { accent: accent.value } }));
      return () => h("box", [h(ThemedText), h(NestedScope)]);
    },
  });
  const mounted = mountFresco(() => h(RootScope));
  const root = firstChild(mounted);

  assert.deepEqual(
    root.children.map((node) => [node.props.text, node.props.reducedMotion]),
    [
      ["yellow", false],
      ["yellow", true],
    ],
  );

  accent.value = "green";
  await nextTick();
  assert.deepEqual(
    root.children.map((node) => [node.props.text, node.props.reducedMotion]),
    [
      ["green", false],
      ["green", true],
    ],
  );
  mounted.unmount();
});
