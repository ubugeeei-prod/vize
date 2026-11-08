import assert from "node:assert/strict";
import { test } from "vitest";
import { defineComponent, h, nextTick, ref } from "vue";

import { firstChild, mountFresco } from "./testing/mount.js";
import { ThemeProbe } from "./theme-probe.test-support.js";
import { defaultFrescoTheme, provideFrescoTheme, resolveFrescoTheme } from "./theme.js";

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
  const RootScope = defineComponent({
    name: "RootThemeScope",
    setup() {
      provideFrescoTheme(() => ({
        colors: { accent: accent.value },
        motion: { reduced: true },
      }));
      return () => h("box", [h(ThemeProbe)]);
    },
  });
  const mounted = mountFresco(() => h(RootScope));
  const probe = () => firstChild(mounted).children[0]?.props;

  assert.deepEqual(probe(), {
    accent: "yellow",
    primary: "cyan",
    reducedMotion: true,
  });

  accent.value = "green";
  await nextTick();
  assert.deepEqual(probe(), {
    accent: "green",
    primary: "cyan",
    reducedMotion: true,
  });
  mounted.unmount();
});
