import assert from "node:assert/strict";

import { mount } from "@vue/test-utils";
import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, shallowRef } from "vue";

import StickyStackItem from "./sticky-stack-item.vue";
import { computeStickyOffsets, sortByDocumentOrder } from "./sticky-stack-layout.ts";
import StickyStack from "./sticky-stack.vue";

test("stacks offsets and skips disabled heights", () => {
  assert.deepEqual(computeStickyOffsets([40, 30, 20], 10), { tops: [10, 50, 80], total: 100 });
  assert.deepEqual(computeStickyOffsets([40, Number.NaN, 20], 0, [true, true, false]), {
    tops: [0, 40, 40],
    total: 40,
  });
  assert.deepEqual(computeStickyOffsets([], Number.NaN), { tops: [], total: 0 });
});

test("orders entries by document position and keeps detached ones in registration order", () => {
  const parent = document.createElement("div");
  const first = document.createElement("p");
  const second = document.createElement("p");
  parent.append(first, second);
  const ordered = sortByDocumentOrder([
    { name: "second", element: second },
    { name: "detached", element: null },
    { name: "first", element: first },
  ]);
  assert.deepEqual(
    ordered.map((entry) => entry.name),
    ["first", "second", "detached"],
  );
});

function stub(element: Element, height: number, top: number): void {
  element.getBoundingClientRect = () => ({ height, top }) as DOMRect;
}

test("items stick below the base offset plus measured heights of earlier items", async () => {
  const showMiddle = shallowRef(true);
  const stuckEvents: boolean[] = [];
  const App = defineComponent(
    () => () =>
      h(
        StickyStack,
        { offset: 8 },
        {
          default: ({ total }: { total: number }) => [
            h("output", String(total)),
            h(
              StickyStackItem,
              { estimatedHeight: 40, onStuckChange: (value: boolean) => stuckEvents.push(value) },
              {
                default: ({ top, stuck }: { top: number; stuck: boolean }) =>
                  `app:${top}:${String(stuck)}`,
              },
            ),
            showMiddle.value
              ? h(
                  StickyStackItem,
                  { estimatedHeight: 30 },
                  { default: ({ top }: { top: number }) => `tabs:${top}` },
                )
              : null,
            h(
              StickyStackItem,
              { estimatedHeight: 20 },
              { default: ({ top }: { top: number }) => `filters:${top}` },
            ),
          ],
        },
      ),
  );
  const wrapper = mount(App, { attachTo: document.body });
  await nextTick();
  const items = () => wrapper.findAll('[data-vize-ui="sticky-stack-item"]');
  assert.deepEqual(
    items().map((item) => item.text()),
    ["app:8:false", "tabs:48", "filters:78"],
  );
  assert.equal(wrapper.get("output").text(), "98");
  assert.match(items()[1]?.attributes("style") ?? "", /position: sticky; top: 48px/);
  assert.match(
    wrapper.get('[data-vize-ui="sticky-stack"]').attributes("style") ?? "",
    /--vize-ui-sticky-stack-height: 98px/,
  );

  const [app, tabs, filters] = items().map((item) => item.element);
  assert.ok(app && tabs && filters);
  stub(app, 50, 8);
  stub(tabs, 10, 300);
  stub(filters, 20, 400);
  window.dispatchEvent(new Event("scroll"));
  await nextTick();
  assert.deepEqual(
    items().map((item) => item.text()),
    ["app:8:true", "tabs:58", "filters:68"],
  );
  assert.equal(items()[0]?.attributes("data-stuck"), "");
  assert.deepEqual(stuckEvents, [true]);

  showMiddle.value = false;
  await nextTick();
  assert.deepEqual(
    items().map((item) => item.text()),
    ["app:8:true", "filters:58"],
  );
  wrapper.unmount();
});

test("disabled items scroll normally and add no offset", () => {
  const wrapper = mount(
    defineComponent(
      () => () =>
        h(StickyStack, null, () => [
          h(StickyStackItem, { estimatedHeight: 40, disabled: true }, () => "banner"),
          h(
            StickyStackItem,
            { estimatedHeight: 20 },
            { default: ({ top }: { top: number }) => `bar:${top}` },
          ),
        ]),
    ),
  );
  const [banner, bar] = wrapper.findAll('[data-vize-ui="sticky-stack-item"]');
  assert.equal(banner?.attributes("style"), undefined);
  assert.equal(banner?.attributes("data-disabled"), "");
  assert.equal(bar?.text(), "bar:0");
  wrapper.unmount();
});

test("items outside a stack throw the context diagnostic", () => {
  assert.throws(() => mount(StickyStackItem), /VIZE_UI_CONTEXT_MISSING: StickyStack/);
});
