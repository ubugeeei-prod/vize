import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type { HotspotDefinition, HotspotRootExpose, HotspotSlotState } from "./hotspot.ts";
import HotspotArea from "./hotspot-area.vue";
import HotspotContent from "./hotspot-content.vue";
import HotspotImage from "./hotspot-image.vue";
import HotspotMarker from "./hotspot-marker.vue";
import HotspotRoot from "./hotspot-root.vue";
import { mountInteraction } from "../../../testing/mount.ts";

interface Room {
  readonly floor: number;
}

const rooms: readonly HotspotDefinition<Room>[] = [
  { id: "kitchen", x: 20, y: 30, label: "Kitchen", data: { floor: 1 } },
  { id: "study", x: 70, y: 32, label: "Study", data: { floor: 1 } },
  { id: "attic", x: 45, y: 5, label: "Attic", data: { floor: 3 } },
  { id: "cellar", x: 50, y: 90, label: "Cellar", data: { floor: 0 }, disabled: true },
];

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function settle(): Promise<void> {
  await nextTick();
  await wait(0);
  await nextTick();
}

function mountHotspots(props: Record<string, unknown> = {}) {
  return mountInteraction(HotspotRoot, {
    props: { id: "plan", hotspots: rooms, ...props },
    record: ["update:active", "activeChange"],
    slots: {
      default: (state: HotspotSlotState<Room>) => [
        h(HotspotImage, { src: "/plan.png", alt: "Floor plan", width: 800, height: 600 }),
        ...state.hotspots.map((room) =>
          h(
            HotspotMarker,
            {
              key: room.id,
              id: room.id,
              x: room.x,
              y: room.y,
              label: room.label,
              disabled: room.disabled,
            },
            {
              trigger: ({ state: markerState }: { state: string }) =>
                h("span", { "data-marker-state": markerState }),
              default: () =>
                h(
                  HotspotContent,
                  { portalDisabled: true, ariaLabel: `${room.label} details` },
                  () => h("p", `${room.label} on floor ${room.data?.floor ?? 0}`),
                ),
            },
          ),
        ),
        h(HotspotArea, {
          id: "garden",
          label: "Garden",
          shape: {
            type: "polygon",
            points: [
              { x: 0, y: 80 },
              { x: 40, y: 80 },
              { x: 40, y: 100 },
              { x: 0, y: 100 },
            ],
          },
        }),
        h(HotspotArea, {
          id: "porch",
          label: "Porch",
          href: "/rooms/porch",
          shape: { type: "rect", x: 80, y: 80, width: 20, height: 20 },
        }),
        h("output", { "data-open-ids": state.openIds.join(",") }, state.active ?? ""),
      ],
    },
  });
}

function markerButton(root: HTMLElement, id: string): HTMLButtonElement {
  const button = root.querySelector<HTMLButtonElement>(
    `[data-vize-ui="hotspot-marker"][data-id="${id}"] [data-vize-ui="popover-trigger"]`,
  );
  assert.ok(button, `${id} marker button must render`);
  return button;
}

function marker(root: HTMLElement, id: string): HTMLElement {
  const element = root.querySelector<HTMLElement>(
    `[data-vize-ui="hotspot-marker"][data-id="${id}"]`,
  );
  assert.ok(element);
  return element;
}

function content(id: string): HTMLElement | null {
  return document.querySelector<HTMLElement>(
    `[data-hotspot-id="${id}"] [data-vize-ui="popover-content"]`,
  );
}

test("renders positioned marker buttons, a safe image, regions, and typed slot data", () => {
  const handle = mountHotspots();
  const root = handle.root();

  assert.equal(root.id, "plan");
  assert.equal(root.getAttribute("data-vize-ui"), "hotspot-root");
  assert.equal(root.getAttribute("data-state"), "closed");
  const image = root.querySelector<HTMLImageElement>('[data-vize-ui="hotspot-image"]');
  assert.equal(image?.getAttribute("src"), "/plan.png");
  assert.equal(image?.alt, "Floor plan");
  assert.equal(image?.getAttribute("loading"), "lazy");
  assert.equal(root.querySelectorAll('[data-vize-ui="hotspot-marker"]').length, 4);
  const kitchen = marker(root, "kitchen");
  assert.equal(kitchen.style.getPropertyValue("--vize-ui-hotspot-x"), "20%");
  assert.equal(kitchen.style.getPropertyValue("--vize-ui-hotspot-y"), "30%");
  assert.equal(kitchen.getAttribute("data-state"), "closed");
  const button = markerButton(root, "kitchen");
  assert.equal(button.type, "button");
  assert.equal(button.getAttribute("aria-label"), "Kitchen");
  assert.equal(button.getAttribute("aria-expanded"), "false");
  assert.equal(marker(root, "cellar").getAttribute("data-state"), "disabled");
  assert.equal(markerButton(root, "cellar").disabled, true);
  handle.unmount();
});

test("clicking a marker opens its content and exclusive roots close the previous one", async () => {
  const handle = mountHotspots();
  const root = handle.root();

  await handle.click(markerButton(root, "kitchen"));
  await settle();
  assert.equal(marker(root, "kitchen").getAttribute("data-state"), "open");
  assert.equal(markerButton(root, "kitchen").getAttribute("aria-expanded"), "true");
  assert.equal(content("kitchen")?.textContent, "Kitchen on floor 1");
  assert.equal(root.getAttribute("data-active"), "kitchen");

  await handle.click(markerButton(root, "study"));
  await settle();
  assert.equal(marker(root, "kitchen").getAttribute("data-state"), "closed");
  assert.equal(marker(root, "study").getAttribute("data-state"), "open");
  assert.equal(root.querySelector("output")?.getAttribute("data-open-ids"), "study");

  await handle.click(markerButton(root, "study"));
  await settle();
  assert.equal(root.getAttribute("data-state"), "closed");
  assert.deepEqual(handle.wrapper.emitted("update:active"), [["kitchen"], ["study"], [null]]);
  assert.deepEqual(handle.wrapper.emitted("activeChange")?.[0], ["kitchen", null, "marker"]);
  assert.deepEqual(handle.wrapper.emitted("activeChange")?.[2], [null, "study", "dismiss"]);
  handle.unmount();
});

test("Escape closes the open content and returns focus to its marker", async () => {
  const handle = mountHotspots();
  const root = handle.root();
  const button = markerButton(root, "kitchen");
  button.focus();
  await handle.click(button);
  await settle();
  const panel = content("kitchen");
  assert.ok(panel);
  panel.dispatchEvent(
    new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }),
  );
  await settle();
  assert.equal(marker(root, "kitchen").getAttribute("data-state"), "closed");
  assert.ok(handle.activeElement() === button);
  handle.unmount();
});

test("non-exclusive roots keep several markers open and track the latest as active", async () => {
  const handle = mountHotspots({ exclusive: false });
  const root = handle.root();
  await handle.click(markerButton(root, "kitchen"));
  await settle();
  await handle.click(markerButton(root, "study"));
  await settle();
  assert.equal(root.querySelector("output")?.getAttribute("data-open-ids"), "kitchen,study");
  assert.equal(root.getAttribute("data-active"), "study");
  const exposed = handle.exposes<HotspotRootExpose>();
  assert.deepEqual(exposed.openIds, ["kitchen", "study"]);
  assert.equal(exposed.close("study"), true);
  await settle();
  assert.equal(
    root.getAttribute("data-active"),
    "kitchen",
    "closing the active marker restores the previous one",
  );
  assert.equal(exposed.open("attic"), true);
  assert.equal(exposed.close("kitchen"), true);
  await settle();
  assert.deepEqual(exposed.openIds, ["attic"]);
  assert.equal(exposed.close("kitchen"), false);
  assert.equal(exposed.close(), true);
  await settle();
  assert.deepEqual(exposed.openIds, []);
  handle.unmount();
});

test("arrow keys move focus to the spatially nearest enabled marker", async () => {
  const handle = mountHotspots();
  const root = handle.root();
  const kitchen = markerButton(root, "kitchen");
  kitchen.focus();

  const right = new KeyboardEvent("keydown", {
    key: "ArrowRight",
    bubbles: true,
    cancelable: true,
  });
  kitchen.dispatchEvent(right);
  assert.equal(right.defaultPrevented, true);
  assert.ok(handle.activeElement() === markerButton(root, "study"));
  markerButton(root, "study").dispatchEvent(
    new KeyboardEvent("keydown", { key: "ArrowUp", bubbles: true, cancelable: true }),
  );
  assert.ok(handle.activeElement() === markerButton(root, "attic"));
  const down = new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true, cancelable: true });
  markerButton(root, "attic").dispatchEvent(down);
  assert.ok(
    handle.activeElement() === markerButton(root, "kitchen") ||
      handle.activeElement() === markerButton(root, "study"),
  );
  const stuck = new KeyboardEvent("keydown", { key: "ArrowLeft", bubbles: true, cancelable: true });
  kitchen.focus();
  kitchen.dispatchEvent(stuck);
  assert.equal(stuck.defaultPrevented, false, "no marker lies further left");
  const modified = new KeyboardEvent("keydown", {
    key: "ArrowRight",
    bubbles: true,
    cancelable: true,
    ctrlKey: true,
  });
  kitchen.dispatchEvent(modified);
  assert.equal(modified.defaultPrevented, false);
  kitchen.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
  assert.ok(handle.activeElement() === kitchen, "the disabled cellar is skipped");
  handle.unmount();
});

test("controlled active wins until the parent accepts the request", async () => {
  const handle = mountHotspots({ active: "attic" });
  const root = handle.root();
  await settle();
  assert.equal(marker(root, "attic").getAttribute("data-state"), "open");
  await handle.click(markerButton(root, "kitchen"));
  await settle();
  assert.deepEqual(handle.wrapper.emitted("update:active"), [["kitchen"]]);
  assert.equal(marker(root, "attic").getAttribute("data-state"), "open");
  await handle.wrapper.setProps({ active: "kitchen" });
  await settle();
  assert.equal(marker(root, "kitchen").getAttribute("data-state"), "open");
  assert.equal(marker(root, "attic").getAttribute("data-state"), "closed");
  handle.unmount();
});

test("regions toggle as buttons, render safe links, and support hit testing", async () => {
  const handle = mountHotspots();
  const root = handle.root();
  const garden = root.querySelector<SVGGElement>('[data-vize-ui="hotspot-area-button"]');
  assert.ok(garden);
  assert.equal(garden.getAttribute("role"), "button");
  assert.equal(garden.getAttribute("tabindex"), "0");
  assert.equal(garden.getAttribute("aria-label"), "Garden");
  assert.equal(garden.getAttribute("aria-pressed"), "false");
  assert.equal(garden.querySelector("polygon")?.getAttribute("points"), "0,80 40,80 40,100 0,100");
  const area = root.querySelector('[data-vize-ui="hotspot-area"][data-id="garden"]');
  assert.equal(area?.getAttribute("viewBox"), "0 0 100 100");
  assert.equal(area?.getAttribute("preserveAspectRatio"), "none");

  garden.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
  await settle();
  assert.equal(garden.getAttribute("aria-pressed"), "true");
  assert.equal(root.getAttribute("data-active"), "garden");
  assert.deepEqual(handle.wrapper.emitted("activeChange")?.[0], ["garden", null, "area"]);
  const enter = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });
  garden.dispatchEvent(enter);
  await settle();
  assert.equal(enter.defaultPrevented, true);
  assert.equal(garden.getAttribute("aria-pressed"), "false");

  const link = root.querySelector('[data-vize-ui="hotspot-area-link"]');
  assert.equal(link?.getAttribute("href"), "/rooms/porch");
  assert.equal(link?.getAttribute("aria-label"), "Porch");
  assert.ok(link?.querySelector("rect"));

  const exposed = handle.exposes<HotspotRootExpose>();
  assert.deepEqual(exposed.hitTest(10, 90), ["garden"]);
  assert.deepEqual(exposed.hitTest(90, 85), ["porch"]);
  assert.deepEqual(exposed.hitTest(50, 50), []);
  handle.unmount();
});

test("unsafe links and images are not rendered and disabled regions stay inert", async () => {
  const handle = mountInteraction(HotspotRoot, {
    slots: {
      default: () => [
        h(HotspotImage, { src: "javascript:alert(1)", alt: "Unsafe" }),
        h(HotspotArea, {
          id: "bad",
          label: "Bad",
          href: " javascript:alert(1)",
          shape: { type: "circle", cx: 50, cy: 50, r: 10 },
        }),
        h(
          HotspotArea,
          {
            id: "off",
            label: "Off",
            disabled: true,
            shape: { type: "circle", cx: 20, cy: 20, r: 5 },
            onActivate: () => undefined,
          },
          () => h("title", "Closed area"),
        ),
      ],
    },
  });
  const root = handle.root();
  const image = root.querySelector('[data-vize-ui="hotspot-image"]');
  assert.equal(image?.hasAttribute("src"), false);
  assert.equal(image?.getAttribute("data-invalid"), "true");
  assert.equal(
    root.querySelector('[data-vize-ui="hotspot-area-link"]')?.hasAttribute("href"),
    false,
  );
  const off = root.querySelector('[data-id="off"] [data-vize-ui="hotspot-area-button"]');
  assert.ok(off);
  assert.equal(off.getAttribute("tabindex"), "-1");
  assert.equal(off.getAttribute("aria-disabled"), "true");
  assert.ok(off.querySelector("circle"));
  assert.equal(off.querySelector("title")?.textContent, "Closed area");
  off.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
  await settle();
  assert.equal(off.getAttribute("aria-pressed"), "false");
  handle.unmount();
});

test("region activate listeners can cancel the toggle", async () => {
  const handle = mountInteraction(HotspotRoot, {
    slots: {
      default: () =>
        h(HotspotArea, {
          id: "blocked",
          label: "Blocked",
          shape: { type: "rect", x: 0, y: 0, width: 10, height: 10 },
          onActivate: (event: Event) => event.preventDefault(),
        }),
    },
  });
  const button = handle.root().querySelector('[data-vize-ui="hotspot-area-button"]');
  button?.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
  await settle();
  assert.equal(button?.getAttribute("aria-pressed"), "false");
  handle.unmount();
});

test("compound parts require matching providers", () => {
  for (const [part, props] of [
    [HotspotMarker, { id: "a", x: 1, y: 1, label: "A" }],
    [
      HotspotArea,
      { id: "a", label: "A", shape: { type: "rect", x: 0, y: 0, width: 1, height: 1 } },
    ],
  ] as const) {
    assert.throws(
      () => mountInteraction(part, { props }),
      /VIZE_UI_CONTEXT_MISSING: Hotspot requires a matching provider/,
    );
  }
  assert.throws(
    () => mountInteraction(HotspotContent),
    /VIZE_UI_CONTEXT_MISSING: HotspotMarker requires a matching provider/,
  );
});
