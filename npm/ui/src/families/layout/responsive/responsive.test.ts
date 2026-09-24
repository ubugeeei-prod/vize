import assert from "node:assert/strict";

import { mount } from "@vue/test-utils";
import { afterEach, test } from "vite-plus/test";
import { defineComponent, effectScope, h, nextTick } from "vue";

import { defaultBreakpoints, resolveActiveBreakpoint, useBreakpoint } from "./breakpoint.ts";
import ResponsiveShow from "./responsive-show.vue";
import ResponsiveSwitch from "./responsive-switch.vue";

function setViewport(width: number): void {
  window.happyDOM.setViewport({ width, height: 800 });
  window.dispatchEvent(new Event("resize"));
}

afterEach(() => setViewport(1024));

test("resolves the largest reached breakpoint deterministically", () => {
  assert.equal(resolveActiveBreakpoint(defaultBreakpoints, null), null);
  assert.equal(resolveActiveBreakpoint(defaultBreakpoints, 500), null);
  assert.equal(resolveActiveBreakpoint(defaultBreakpoints, 768), "md");
  assert.equal(resolveActiveBreakpoint(defaultBreakpoints, 5000), "2xl");
  assert.equal(resolveActiveBreakpoint({ compact: 0, wide: 900 }, 100), "compact");
});

test("useBreakpoint starts at ssrWidth, reads the real width after mount, and follows resize", async () => {
  setViewport(1300);
  let state: ReturnType<typeof useBreakpoint> | undefined;
  const Probe = defineComponent(() => {
    state = useBreakpoint({ ssrWidth: 600 });
    return () => h("span", state?.active.value ?? "none");
  });
  const wrapper = mount(Probe);
  await nextTick();
  assert.ok(state);
  assert.equal(state.width.value, 1300);
  assert.equal(wrapper.text(), "xl");
  assert.equal(state.isAbove("lg"), true);
  assert.equal(state.isBelow("2xl"), true);
  assert.equal(state.isBetween("md", "2xl"), true);

  setViewport(700);
  await nextTick();
  assert.equal(wrapper.text(), "sm");
  wrapper.unmount();
  setViewport(1500);
  assert.equal(state.width.value, 700);
});

test("useBreakpoint accepts custom maps, immediate reads, injected hosts, and rejects missing scopes", () => {
  const host = Object.assign(new EventTarget(), { innerWidth: 950 });
  const scope = effectScope();
  const state = scope.run(() =>
    useBreakpoint({ phone: 0, tablet: 600, desktop: 1000 }, { immediate: true, host }),
  );
  assert.equal(state?.active.value, "tablet");
  host.innerWidth = 1200;
  host.dispatchEvent(new Event("resize"));
  assert.equal(state?.active.value, "desktop");
  scope.stop();

  const empty = effectScope().run(() => useBreakpoint({ immediate: true, host: null }));
  assert.equal(empty?.width.value, null);
  assert.equal(empty?.isAbove("sm"), false);
  assert.throws(() => useBreakpoint(), /VIZE_UI_RESPONSIVE_SETUP/);
});

test("ResponsiveSwitch renders the largest reached breakpoint slot with fallbacks", async () => {
  setViewport(1100);
  const wrapper = mount(ResponsiveSwitch, {
    slots: {
      base: () => "phone",
      md: ({ active }: { active: string | null }) => `tablet:${active}`,
      default: () => "fallback",
    },
  });
  await nextTick();
  const root = wrapper.get('[data-vize-ui="responsive-switch"]');
  assert.equal(root.text(), "tablet:lg");
  assert.equal(root.attributes("data-breakpoint"), "lg");
  assert.equal(root.attributes("data-slot"), "md");

  setViewport(400);
  await nextTick();
  assert.equal(root.text(), "phone");
  assert.equal(root.attributes("data-breakpoint"), "base");
  wrapper.unmount();

  const fallback = mount(ResponsiveSwitch, {
    props: { as: "section" },
    slots: { default: () => "all" },
  });
  assert.equal(fallback.element.tagName, "SECTION");
  assert.equal(fallback.text(), "all");
  fallback.unmount();
});

test("ResponsiveShow unmounts or hides content outside its range", async () => {
  setViewport(900);
  const unmounted = mount(ResponsiveShow, {
    props: { above: "lg" },
    slots: { default: () => "desktop" },
  });
  const hidden = mount(ResponsiveShow, {
    props: { below: "lg", hideMode: "hidden" },
    slots: { default: ({ visible }: { visible: boolean }) => `mobile:${String(visible)}` },
  });
  await nextTick();
  const emptyWrapper = unmounted.get('[data-vize-ui="responsive-show"]');
  assert.equal(emptyWrapper.attributes("hidden"), "");
  assert.equal(emptyWrapper.text(), "");
  const kept = hidden.get('[data-vize-ui="responsive-show"]');
  assert.equal(kept.attributes("hidden"), undefined);
  assert.equal(kept.text(), "mobile:true");

  setViewport(1200);
  await nextTick();
  assert.equal(unmounted.text(), "desktop");
  assert.equal(kept.attributes("hidden"), "");
  assert.equal(kept.attributes("data-state"), "hidden");

  const unknown = mount(ResponsiveShow, {
    props: { above: "huge" },
    slots: { default: () => "x" },
  });
  assert.equal(unknown.text(), "");
  const always = mount(ResponsiveShow, { slots: { default: () => "always" } });
  assert.equal(always.text(), "always");
  for (const wrapper of [unmounted, hidden, unknown, always]) wrapper.unmount();
});
