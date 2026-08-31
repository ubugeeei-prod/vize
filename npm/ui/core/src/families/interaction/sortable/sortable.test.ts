import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope, ref } from "vue";

import {
  attachSourceProps,
  keydown,
  liveRegionText,
  rect,
  removeLiveRegions,
} from "../drag-and-drop/drag-and-drop-test-utils.ts";
import { pointer } from "../move/move-test-utils.ts";
import { createSortable, useSortable } from "./sortable.ts";
import type { SortableEvent, SortableItemRegistration, SortableOptions } from "./sortable.ts";

interface ListHarness {
  readonly controller: ReturnType<typeof createSortable>;
  readonly events: SortableEvent[];
  readonly hosts: readonly HTMLElement[];
  readonly registrations: readonly SortableItemRegistration[];
  readonly unmount: () => void;
}

/** Mount a three-item vertical list: 100px-tall rows stacked from y = 0. */
function mountList(options: SortableOptions = {}, count = 3): ListHarness {
  const events: SortableEvent[] = [];
  const record = (event: SortableEvent) => events.push(event);
  const controller = createSortable({
    onSortStart: record,
    onSortPreview: record,
    onSortCommit: record,
    onSortCancel: record,
    ...options,
  });
  const hosts: HTMLElement[] = [];
  const registrations: SortableItemRegistration[] = [];
  const list = document.createElement("ul");
  document.body.append(list);
  for (let index = 0; index < count; index += 1) {
    const host = document.createElement("li");
    host.tabIndex = 0;
    list.append(host);
    const registration = controller.registerItem({
      key: `item-${index}`,
      element: () => host,
      label: `Item ${index}`,
      getRect: () => rect(index * 100, 0, index * 100 + 100, 100),
    });
    attachSourceProps(host, registration.itemProps);
    hosts.push(host);
    registrations.push(registration);
  }
  return {
    controller,
    events,
    hosts,
    registrations,
    unmount: () => {
      try {
        controller.dispose();
      } finally {
        list.remove();
        removeLiveRegions();
      }
    },
  };
}

test("pointer sorting previews edges and commits origin and final indexes", () => {
  const harness = mountList();
  try {
    const host = harness.hosts[0];
    assert.ok(host);
    host.dispatchEvent(pointer("pointerdown", 50, 50));
    document.dispatchEvent(pointer("pointermove", 50, 60));
    assert.equal(harness.events[0]?.type, "sortstart");
    assert.equal(harness.events[0]?.fromIndex, 0);
    assert.equal(harness.controller.isSorting.value, true);
    assert.equal(harness.controller.activeKey.value, "item-0");
    assert.equal(harness.registrations[0]?.isDragging.value, true);

    document.dispatchEvent(pointer("pointermove", 50, 290));
    const preview = harness.events.at(-1);
    assert.equal(preview?.type, "sortpreview");
    assert.equal(preview?.overKey, "item-2");
    assert.equal(preview?.position, "after");
    assert.equal(preview?.toIndex, 2);
    assert.equal(harness.controller.indicator.value?.position, "after");
    assert.deepEqual(harness.controller.indicator.value?.line, rect(300, 0, 300, 100));

    document.dispatchEvent(pointer("pointermove", 50, 292));
    assert.equal(
      harness.events.filter((event) => event.type === "sortpreview").length,
      1,
      "an unchanged projection must not emit again",
    );

    document.dispatchEvent(pointer("pointerup", 50, 290));
    const commit = harness.events.at(-1);
    assert.equal(commit?.type, "sortcommit");
    assert.equal(commit?.fromIndex, 0);
    assert.equal(commit?.toIndex, 2);
    assert.equal(commit?.overKey, "item-2");
    assert.equal(harness.controller.isSorting.value, false);
    assert.equal(harness.controller.indicator.value, null);
  } finally {
    harness.unmount();
  }
});

test("pointer sorting projects before halves and releases outside as a cancel", () => {
  const harness = mountList();
  try {
    const host = harness.hosts[2];
    assert.ok(host);
    host.dispatchEvent(pointer("pointerdown", 50, 250));
    document.dispatchEvent(pointer("pointermove", 50, 130));
    const preview = harness.events.at(-1);
    assert.equal(preview?.type, "sortpreview");
    assert.equal(preview?.overKey, "item-1");
    assert.equal(preview?.position, "before");
    assert.equal(preview?.toIndex, 1);

    document.dispatchEvent(pointer("pointermove", 500, 500));
    document.dispatchEvent(pointer("pointerup", 500, 500));
    const cancel = harness.events.at(-1);
    assert.equal(cancel?.type, "sortcancel");
    assert.equal(cancel?.toIndex, 2, "the item must return to its origin index");
  } finally {
    harness.unmount();
  }
});

test("keyboard sorting steps, clamps, announces, and commits", () => {
  const harness = mountList();
  try {
    const host = harness.hosts[0];
    assert.ok(host);
    host.dispatchEvent(keydown("Enter"));
    assert.equal(harness.events[0]?.type, "sortstart");
    assert.equal(harness.controller.isSorting.value, true);
    assert.match(liveRegionText() ?? "", /Picked up Item 0, position 1 of 3\./);
    assert.match(liveRegionText() ?? "", /arrow keys/);

    host.dispatchEvent(keydown("ArrowDown"));
    const preview = harness.events.at(-1);
    assert.equal(preview?.type, "sortpreview");
    assert.equal(preview?.toIndex, 1);
    assert.match(liveRegionText() ?? "", /Item 0 moved to position 2 of 3\./);
    assert.equal(harness.controller.indicator.value?.toIndex, 1);

    host.dispatchEvent(keydown("ArrowUp"));
    host.dispatchEvent(keydown("ArrowUp"));
    assert.equal(harness.events.at(-1)?.toIndex, 0, "steps must clamp at the first index");

    host.dispatchEvent(keydown("End"));
    assert.equal(harness.events.at(-1)?.toIndex, 2);
    host.dispatchEvent(keydown("Home"));
    assert.equal(harness.events.at(-1)?.toIndex, 0);

    host.dispatchEvent(keydown("ArrowDown"));
    host.dispatchEvent(keydown(" "));
    const commit = harness.events.at(-1);
    assert.equal(commit?.type, "sortcommit");
    assert.equal(commit?.fromIndex, 0);
    assert.equal(commit?.toIndex, 1);
    assert.match(liveRegionText() ?? "", /Item 0 dropped, final position 2 of 3\./);
    assert.equal(harness.controller.isSorting.value, false);
  } finally {
    harness.unmount();
  }
});

test("keyboard sorting cancels on Escape and announces the return position", () => {
  const harness = mountList();
  try {
    const host = harness.hosts[1];
    assert.ok(host);
    host.dispatchEvent(keydown(" "));
    host.dispatchEvent(keydown("ArrowDown"));
    host.dispatchEvent(keydown("Escape"));
    const cancel = harness.events.at(-1);
    assert.equal(cancel?.type, "sortcancel");
    assert.equal(cancel?.toIndex, 1);
    assert.match(liveRegionText() ?? "", /Sorting canceled\. Item 1 returned to position 2 of 3\./);
    assert.equal(harness.controller.isSorting.value, false);
  } finally {
    harness.unmount();
  }
});

test("grid arrows step by the resolved column count", () => {
  const harness = mountList({ orientation: "grid", columns: 2 }, 4);
  try {
    const host = harness.hosts[0];
    assert.ok(host);
    host.dispatchEvent(keydown("Enter"));
    host.dispatchEvent(keydown("ArrowDown"));
    assert.equal(harness.events.at(-1)?.toIndex, 2);
    host.dispatchEvent(keydown("ArrowRight"));
    assert.equal(harness.events.at(-1)?.toIndex, 3);
    host.dispatchEvent(keydown("ArrowUp"));
    assert.equal(harness.events.at(-1)?.toIndex, 1);
    host.dispatchEvent(keydown("Escape"));
  } finally {
    harness.unmount();
  }
});

test("RTL horizontal sorting flips the logical arrow directions", () => {
  const harness = mountList({ orientation: "horizontal", direction: "rtl" });
  try {
    const host = harness.hosts[0];
    assert.ok(host);
    host.dispatchEvent(keydown("Enter"));
    host.dispatchEvent(keydown("ArrowLeft"));
    assert.equal(harness.events.at(-1)?.toIndex, 1, "ArrowLeft must advance in RTL");
    host.dispatchEvent(keydown("ArrowRight"));
    assert.equal(harness.events.at(-1)?.toIndex, 0, "ArrowRight must retreat in RTL");
    host.dispatchEvent(keydown("Escape"));
  } finally {
    harness.unmount();
  }
});

test("nesting previews inside targets from keyboard and pointer", () => {
  const harness = mountList({ nesting: true });
  try {
    const keyboardHost = harness.hosts[1];
    assert.ok(keyboardHost);
    keyboardHost.dispatchEvent(keydown("Enter"));
    keyboardHost.dispatchEvent(keydown("ArrowRight"));
    const nested = harness.events.at(-1);
    assert.equal(nested?.type, "sortpreview");
    assert.equal(nested?.overKey, "item-0");
    assert.equal(nested?.position, "inside");
    assert.match(liveRegionText() ?? "", /Item 1 placed inside Item 0\./);
    assert.equal(harness.controller.indicator.value?.position, "inside");
    keyboardHost.dispatchEvent(keydown("ArrowLeft"));
    assert.equal(harness.events.at(-1)?.position, null, "ArrowLeft must leave the nest");
    keyboardHost.dispatchEvent(keydown("ArrowRight"));
    keyboardHost.dispatchEvent(keydown("Enter"));
    const commit = harness.events.at(-1);
    assert.equal(commit?.type, "sortcommit");
    assert.equal(commit?.position, "inside");
    assert.equal(commit?.overKey, "item-0");

    const pointerHost = harness.hosts[2];
    assert.ok(pointerHost);
    pointerHost.dispatchEvent(pointer("pointerdown", 50, 250));
    document.dispatchEvent(pointer("pointermove", 50, 150));
    const preview = harness.events.at(-1);
    assert.equal(preview?.type, "sortpreview");
    assert.equal(preview?.overKey, "item-1");
    assert.equal(preview?.position, "inside");
    document.dispatchEvent(pointer("pointerup", 50, 150));
    assert.equal(harness.events.at(-1)?.position, "inside");
  } finally {
    harness.unmount();
  }
});

test("disabled controllers and items refuse grabs and cancel active sorts", () => {
  const disabled = ref(false);
  const harness = mountList({ isDisabled: disabled });
  try {
    const sleeping = harness.controller.registerItem({
      key: "sleeping",
      element: () => null,
      isDisabled: true,
    });
    const host = harness.hosts[0];
    assert.ok(host);
    disabled.value = true;
    host.dispatchEvent(keydown("Enter"));
    assert.equal(harness.controller.isSorting.value, false);
    disabled.value = false;
    host.dispatchEvent(keydown("Enter"));
    assert.equal(harness.controller.isSorting.value, true);
    disabled.value = true;
    host.dispatchEvent(keydown("ArrowDown"));
    assert.equal(harness.controller.isSorting.value, false);
    assert.equal(harness.events.at(-1)?.type, "sortcancel");
    sleeping.dispose();
  } finally {
    harness.unmount();
  }
});

test("validates options, rejects duplicates, and disposal is terminal", () => {
  assert.throws(() => createSortable({ orientation: "stack" as never }), /VIZE_UI_SORTABLE_OPTION/);
  assert.throws(() => createSortable({ columns: 0 }), /VIZE_UI_SORTABLE_OPTION/);
  const harness = mountList();
  try {
    assert.throws(
      () => harness.controller.registerItem({ key: "item-0", element: () => null }),
      /duplicate key/,
    );
    assert.equal(harness.controller.cancel(), false);
    const host = harness.hosts[0];
    assert.ok(host);
    host.dispatchEvent(keydown("Enter"));
    assert.equal(harness.controller.cancel(), true);
    assert.equal(harness.events.at(-1)?.type, "sortcancel");
  } finally {
    harness.unmount();
  }
  assert.throws(() => harness.controller.cancel(), /VIZE_UI_SORTABLE_DISPOSED/);
  assert.throws(
    () => harness.controller.registerItem({ key: "late", element: () => null }),
    /DISPOSED/,
  );
});

test("item disposal cancels its own keyboard sort and frees both registrations", () => {
  const harness = mountList();
  try {
    const host = harness.hosts[0];
    const registration = harness.registrations[0];
    assert.ok(host && registration);
    host.dispatchEvent(keydown("Enter"));
    registration.dispose();
    assert.equal(harness.events.at(-1)?.type, "sortcancel");
    assert.equal(harness.controller.isSorting.value, false);
    const again = harness.controller.registerItem({ key: "item-0", element: () => host });
    again.dispose();
  } finally {
    harness.unmount();
  }
});

test("useSortable requires a scope and disposes with it", () => {
  assert.throws(() => useSortable(), /VIZE_UI_SORTABLE_SETUP/);
  const scope = effectScope();
  const controller = scope.run(() => useSortable());
  assert.ok(controller);
  scope.stop();
  assert.throws(() => controller.cancel(), /VIZE_UI_SORTABLE_DISPOSED/);
});
