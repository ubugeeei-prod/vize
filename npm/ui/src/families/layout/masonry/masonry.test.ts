import assert from "node:assert/strict";

import { mount } from "@vue/test-utils";
import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import Masonry from "./masonry.vue";

interface Card {
  readonly id: string;
  readonly size: number;
}

const cards: readonly Card[] = [
  { id: "a", size: 100 },
  { id: "b", size: 300 },
  { id: "c", size: 100 },
  { id: "d", size: 100 },
];

function stubHeights(root: Element, heights: Readonly<Record<string, number>>): void {
  for (const node of root.querySelectorAll<HTMLElement>("[data-masonry-index]")) {
    const id = node.textContent ?? "";
    node.getBoundingClientRect = () => ({ height: heights[id] ?? 0 }) as DOMRect;
  }
}

test("distributes estimated heights into balanced columns with slot state", () => {
  const wrapper = mount(Masonry<Card>, {
    props: {
      items: cards,
      columns: 2,
      gap: 8,
      estimateHeight: (card: Card) => card.size,
      getKey: (card: Card) => card.id,
    },
    slots: {
      item: ({ item, column }: { item: Card; column: number }) =>
        h("span", { "data-col": column }, item.id),
    },
  });
  const columns = wrapper.findAll('[data-part="column"]');
  assert.equal(columns.length, 2);
  assert.deepEqual(
    columns.map((column) => column.text()),
    ["acd", "b"],
  );
  assert.equal(wrapper.attributes("data-columns"), "2");
  assert.match(wrapper.attributes("style") ?? "", /gap: 8px/);
  assert.equal(wrapper.find('[data-col="1"]').text(), "b");
  wrapper.unmount();
});

test("rebalances after measuring real heights", async () => {
  const wrapper = mount(Masonry<Card>, {
    props: { items: cards, columns: 2, getKey: (card: Card) => card.id },
    slots: { item: ({ item }: { item: Card }) => item.id },
  });
  assert.deepEqual(
    wrapper.findAll('[data-part="column"]').map((column) => column.text()),
    ["ac", "bd"],
  );
  stubHeights(wrapper.element, { a: 400, b: 50, c: 50, d: 50 });
  (wrapper.vm as unknown as { measure: () => void }).measure();
  await nextTick();
  assert.deepEqual(
    wrapper.findAll('[data-part="column"]').map((column) => column.text()),
    ["a", "bcd"],
  );
  wrapper.unmount();
});

test("virtualizes items near the scroll viewport", async () => {
  const many = Array.from({ length: 40 }, (_, index) => ({ id: `n${index}`, size: 100 }));
  const wrapper = mount(Masonry<Card>, {
    props: {
      items: many,
      columns: 2,
      gap: 0,
      virtualize: true,
      overscan: 0,
      estimateHeight: () => 100,
    },
    slots: { item: ({ item }: { item: Card }) => item.id },
  });
  const root = wrapper.element as HTMLElement;
  Object.defineProperty(root, "clientHeight", { configurable: true, value: 250 });
  (wrapper.vm as unknown as { measure: () => void }).measure();
  await nextTick();
  assert.equal(
    wrapper.get('[data-part="sizer"]').attributes("style"),
    "position: relative; height: 2000px;",
  );
  assert.deepEqual(
    wrapper.findAll('[data-part="item"]').map((item) => item.text()),
    ["n0", "n1", "n2", "n3", "n4", "n5"],
  );

  root.scrollTop = 1000;
  await wrapper.trigger("scroll");
  const visible = wrapper.findAll('[data-part="item"]').map((item) => item.text());
  assert.ok(visible.includes("n20") && !visible.includes("n0"));
  const first = wrapper.get('[data-masonry-index="20"]');
  assert.match(first.attributes("style") ?? "", /top: 1000px/);
  wrapper.unmount();
});

test("falls back to one column for invalid counts", () => {
  const wrapper = mount(Masonry<Card>, {
    props: { items: cards, columns: 0 },
    slots: { item: ({ item }: { item: Card }) => item.id },
  });
  assert.equal(wrapper.findAll('[data-part="column"]').length, 1);
  wrapper.unmount();
});
