import assert from "node:assert/strict";
import { beforeEach, test } from "node:test";
import { effectScope, ref } from "vue";

import { useVirtualList } from "./virtual-list.ts";
import {
  asElement,
  FakeDocument,
  latestObserver,
  resetObservers,
  resizeObserverHost,
  trigger,
} from "./testing/fake-dom.ts";

beforeEach(resetObservers);

const items = Array.from({ length: 1000 }, (_, index) => `item-${index}`);

void test("renders the initial slice before measuring (server and hydration)", () => {
  const list = useVirtualList(items, { itemSize: 20, initialItemCount: 3, host: () => null });
  assert.deepEqual(
    list.list.value.map((item) => item.index),
    [0, 1, 2],
  );
  assert.equal(list.totalSize.value, 20_000);
  assert.deepEqual(list.wrapperProps.value.style, {
    width: "100%",
    height: "20000px",
    marginTop: "0px",
    marginLeft: "0px",
    display: "block",
  });
});

void test("windows fixed-size items with overscan on scroll and resize", () => {
  const element = new FakeDocument().createElement();
  element.clientHeight = 100;
  const scope = effectScope();
  const list = scope.run(() =>
    useVirtualList(items, { itemSize: 20, overscan: 2, host: resizeObserverHost() }),
  );
  assert.ok(list);
  list.containerProps.ref(asElement(element));
  trigger(latestObserver(), [{}]);
  assert.deepEqual(
    list.list.value.map((item) => item.index),
    [0, 1, 2, 3, 4, 5, 6, 7],
  );

  element.scrollTop = 400;
  list.containerProps.onScroll();
  const indexes = list.list.value.map((item) => item.index);
  assert.deepEqual([indexes[0], indexes.at(-1)], [18, 27]);
  assert.equal(list.wrapperProps.value.style.marginTop, "360px");

  list.scrollTo(500);
  assert.equal(element.scrollTop, 10_000);
  assert.equal(list.list.value[0]?.index, 498);
  scope.stop();
});

void test("supports variable sizes, horizontal lists, and reactive sources", () => {
  const element = new FakeDocument().createElement();
  element.clientWidth = 50;
  const source = ref(["a", "bb", "ccc", "dddd"]);
  const list = useVirtualList(source, {
    itemSize: (_index, item) => item.length * 10,
    orientation: "horizontal",
    overscan: 0,
    host: () => null,
  });
  list.containerProps.ref(asElement(element));
  list.containerProps.onScroll();
  assert.deepEqual(
    list.list.value.map((item) => [item.data, item.offset, item.size]),
    [
      ["a", 0, 10],
      ["bb", 10, 20],
      ["ccc", 30, 30],
    ],
  );
  assert.equal(list.containerProps.style.overflowX, "auto");
  source.value = ["x"];
  assert.equal(list.totalSize.value, 10);
  assert.equal(list.wrapperProps.value.style.display, "flex");
});
