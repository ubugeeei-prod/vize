import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";
import { defineComponent, effectScope, h, nextTick } from "vue";

import type { LandmarkInfo, LandmarkProviderExpose } from "./landmark.ts";
import {
  createLandmarkNavigation,
  landmarkRoleOf,
  useLandmarkNavigation,
} from "./landmark-runtime.ts";
import LandmarkProvider from "./landmark-provider.vue";
import Landmark from "./landmark.vue";
import { mountInteraction } from "../../../testing/mount.ts";

afterEach(() => {
  document.body.innerHTML = "";
});

function press(key: string, shiftKey = false): KeyboardEvent {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key, shiftKey });
  (document.activeElement ?? document.body).dispatchEvent(event);
  return event;
}

function mountPage(providerProps: Record<string, unknown> = {}, extra: () => unknown = () => null) {
  let provider: LandmarkProviderExpose | null = null;
  const navigated: string[] = [];
  const Page = defineComponent({
    name: "LandmarkPage",
    setup: () => () =>
      h("div", { "data-page": "" }, [
        h(
          LandmarkProvider,
          {
            ...providerProps,
            ref: (value) => {
              provider = value as LandmarkProviderExpose | null;
            },
            onNavigate: (landmark: LandmarkInfo) => navigated.push(landmark.role),
          },
          () => [
            h(Landmark, { role: "banner", id: "top" }, () => h("a", { href: "#" }, "Home")),
            h(Landmark, { role: "navigation", ariaLabel: "Primary" }, () =>
              h("a", { href: "#docs" }, "Docs"),
            ),
            h(Landmark, { role: "main", id: "content" }, () => h("button", "Action")),
            extra(),
            h(Landmark, { role: "contentinfo" }, () => "Footer"),
          ],
        ),
      ]),
  });
  const handle = mountInteraction(Page);
  return {
    handle,
    navigated,
    provider: (): LandmarkProviderExpose => {
      if (provider === null) assert.fail("LandmarkProvider must expose its API");
      return provider;
    },
  };
}

test("renders native landmark elements with names, ids, and data hooks", () => {
  const { handle } = mountPage();
  const root = handle.root();
  const banner = root.querySelector("header");
  const nav = root.querySelector("nav");
  const main = root.querySelector("main");
  const footer = root.querySelector("footer");

  assert.ok(banner && nav && main && footer);
  assert.equal(banner.id, "top");
  assert.equal(banner.getAttribute("data-vize-ui"), "landmark");
  assert.equal(banner.getAttribute("data-landmark"), "banner");
  assert.equal(nav.getAttribute("aria-label"), "Primary");
  assert.match(nav.id, /-landmark$/);
  assert.equal(main.hasAttribute("tabindex"), false, "landmarks stay out of the tab order");
  assert.equal(footer.getAttribute("data-landmark"), "contentinfo");

  handle.unmount();
});

test("region, search, and form landmarks render section, search, and form elements", () => {
  const handle = mountInteraction(
    defineComponent({
      setup: () => () =>
        h("div", [
          h(Landmark, { role: "region", ariaLabel: "Results" }),
          h(Landmark, { role: "search", ariaLabel: "Site" }),
          h(Landmark, { role: "form", ariaLabelledby: "form-title" }),
          h(Landmark, { role: "complementary", ariaLabel: "Related" }),
        ]),
    }),
  );
  const tags = [...handle.root().querySelectorAll('[data-vize-ui="landmark"]')].map(
    (element) => element.localName,
  );

  assert.deepEqual(tags, ["section", "search", "form", "aside"]);
  handle.unmount();
});

test("unnamed repeatable landmarks warn in development", () => {
  const warnings: string[] = [];
  const originalWarn = console.warn;
  console.warn = (...values: unknown[]) => warnings.push(values.map(String).join(" "));
  try {
    const handle = mountInteraction(Landmark, { props: { role: "navigation" } });
    handle.unmount();
    const main = mountInteraction(Landmark, { props: { role: "main" } });
    main.unmount();
  } finally {
    console.warn = originalWarn;
  }
  assert.equal(warnings.length, 1);
  assert.match(warnings[0] ?? "", /VIZE_UI_LANDMARK_NAME: a "navigation" landmark/);
});

test("F6 and Shift+F6 cycle focus through landmarks in document order with wrapping", async () => {
  const { handle, navigated } = mountPage();
  await nextTick();
  const root = handle.root();
  const [banner, nav, main, footer] = ["header", "nav", "main", "footer"].map((tag) =>
    root.querySelector<HTMLElement>(tag),
  );

  const first = press("F6");
  assert.equal(first.defaultPrevented, true);
  assert.ok(document.activeElement === banner);
  assert.equal(banner?.getAttribute("tabindex"), "-1");
  await nextTick();
  assert.equal(banner?.getAttribute("data-focused"), "true");
  press("F6");
  assert.ok(document.activeElement === nav);
  assert.equal(banner?.hasAttribute("tabindex"), false, "temporary tabindex is removed on blur");
  press("F6");
  press("F6");
  assert.ok(document.activeElement === footer);
  press("F6");
  assert.ok(document.activeElement === banner, "cycling wraps to the first landmark");
  press("F6", true);
  assert.ok(document.activeElement === footer, "Shift+F6 wraps backwards");

  root.querySelector<HTMLButtonElement>("main button")?.focus();
  press("F6", true);
  assert.ok(document.activeElement === nav, "cycling starts from the landmark containing focus");
  assert.ok(main);
  assert.deepEqual(navigated.slice(0, 2), ["banner", "navigation"]);

  handle.unmount();
});

test("hidden and inert landmarks are skipped", async () => {
  const { handle } = mountPage({}, () =>
    h("div", { hidden: true }, h(Landmark, { role: "complementary", ariaLabel: "Hidden" })),
  );
  await nextTick();
  const nav = handle.root().querySelector("nav");
  nav?.setAttribute("inert", "");

  press("F6");
  press("F6");
  assert.equal(document.activeElement?.localName, "main");
  press("F6");
  assert.equal(document.activeElement?.localName, "footer");

  handle.unmount();
});

test("discovery includes native and role landmarks that Landmark did not render", async () => {
  const { handle, provider } = mountPage({ discover: true }, () => [
    h("aside", { "aria-label": "Ads" }, "Ad"),
    h("div", { role: "search", "aria-label": "Find" }, "Search"),
    h("section", null, "Unnamed section is not a landmark"),
    h("article", null, h("header", null, "Article header is not a banner")),
  ]);
  await nextTick();

  const roles = provider()
    .refresh()
    .map((landmark) => landmark.role);
  assert.deepEqual(roles, [
    "banner",
    "navigation",
    "main",
    "complementary",
    "search",
    "contentinfo",
  ]);
  assert.equal(provider().landmarks.find((landmark) => landmark.role === "search")?.label, "Find");

  handle.unmount();
});

test("expose focuses by id or role and disabled or remapped keys are respected", async () => {
  const { handle, provider } = mountPage({ disabled: true });
  await nextTick();

  const event = press("F6");
  assert.equal(event.defaultPrevented, false, "disabled providers ignore F6");
  assert.equal(provider().focusLandmark("content")?.role, "main");
  assert.equal(document.activeElement?.localName, "main");
  assert.equal(provider().focusLandmark("contentinfo")?.element.localName, "footer");
  assert.equal(provider().focusLandmark("missing"), null);
  assert.equal(provider().focusPrevious()?.role, "main");
  assert.equal(provider().focusNext()?.role, "contentinfo");
  handle.unmount();

  const remapped = mountPage({ nextKey: { key: "F7", altKey: true }, previousKey: null });
  await nextTick();
  press("F6");
  assert.equal(document.activeElement, document.body);
  const alt = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key: "F7",
    altKey: true,
  });
  document.body.dispatchEvent(alt);
  assert.equal(document.activeElement?.localName, "header");
  remapped.handle.unmount();
});

test("landmarks without a provider still render and focus through expose", async () => {
  let exposed: { focus: () => boolean } | null = null;
  const handle = mountInteraction(
    defineComponent({
      setup: () => () =>
        h(Landmark, {
          role: "main",
          ref: (value) => {
            exposed = value as { focus: () => boolean } | null;
          },
        }),
    }),
  );
  await nextTick();
  if (exposed === null) assert.fail("Landmark must expose focus");
  const api: { focus: () => boolean } = exposed;
  assert.equal(api.focus(), true);
  assert.equal(document.activeElement?.localName, "main");
  handle.unmount();
});

test("runtime helpers resolve roles and work in a plain effect scope", () => {
  document.body.innerHTML = `
    <header id="h">Top</header>
    <section id="s">Plain</section>
    <section id="n" aria-label="Named">Named</section>
    <div id="r" role="navigation" aria-label="Side"></div>
    <div id="x" role="button"></div>`;
  const byId = (id: string) => document.getElementById(id) as HTMLElement;
  assert.equal(landmarkRoleOf(byId("h")), "banner");
  assert.equal(landmarkRoleOf(byId("s")), null);
  assert.equal(landmarkRoleOf(byId("n")), "region");
  assert.equal(landmarkRoleOf(byId("r")), "navigation");
  assert.equal(landmarkRoleOf(byId("x")), null);

  const scope = effectScope();
  const controller = scope.run(() => useLandmarkNavigation({ discover: true }));
  assert.ok(controller);
  press("F6");
  assert.equal(document.activeElement?.id, "h");
  scope.stop();
  press("F6");
  assert.equal(document.activeElement?.id, "h", "listeners are removed with the scope");

  const detached = createLandmarkNavigation({ discover: true });
  assert.equal(detached.refresh().length, 3);
  detached.dispose();
  assert.throws(() => useLandmarkNavigation(), /VIZE_UI_LANDMARK_SETUP/);
});
