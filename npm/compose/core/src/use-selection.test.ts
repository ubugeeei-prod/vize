import assert from "node:assert/strict";
import { test } from "node:test";
import { shallowRef } from "vue";

import { useSelection } from "./use-selection.ts";

interface Row {
  readonly id: number;
  readonly disabled?: boolean;
}

const rows: Row[] = [{ id: 1 }, { id: 2 }, { id: 3, disabled: true }, { id: 4 }];

void test("single mode replaces the selection", () => {
  const selection = useSelection({ items: rows, getKey: (row) => row.id });
  const selected = (): Row | undefined => selection.selected.value;
  assert.equal(selected(), undefined);
  assert.equal(selection.select(rows[0] ?? { id: 0 }), true);
  selection.toggle({ id: 2 });
  assert.deepEqual(selection.selectedKeys.value, [2]);
  assert.equal(selected()?.id, 2);
  selection.toggle({ id: 2 });
  assert.equal(selection.count.value, 0);
});

void test("multiple mode with max, required, isSelectable, and select-all", () => {
  const selection = useSelection({
    items: rows,
    multiple: true,
    getKey: (row) => row.id,
    isSelectable: (row) => row.disabled !== true,
    max: 2,
    required: true,
  });
  assert.equal(selection.select({ id: 3, disabled: true }), false);
  selection.select({ id: 4 });
  selection.select({ id: 1 });
  assert.equal(selection.select({ id: 2 }), false);
  assert.deepEqual(
    selection.selected.value.map((row) => row.id),
    [1, 4],
  );
  assert.equal(selection.isIndeterminate.value, true);
  selection.deselect({ id: 4 });
  assert.equal(selection.deselect({ id: 1 }), false, "required keeps the last item");
  selection.clear();
  assert.equal(selection.count.value, 0);
});

void test("select-all and tri-state flags", () => {
  const selection = useSelection({
    items: rows,
    multiple: true,
    getKey: (row) => row.id,
    isSelectable: (row) => row.disabled !== true,
  });
  selection.selectAll();
  assert.deepEqual(selection.selectedKeys.value, [1, 2, 4]);
  assert.equal(selection.isAllSelected.value, true);
  assert.equal(selection.isIndeterminate.value, false);
  selection.selectOnly({ id: 2 });
  assert.deepEqual(selection.selectedKeys.value, [2]);
});

void test("extendTo selects the range from the anchor in display order", () => {
  const selection = useSelection({
    items: rows,
    multiple: true,
    getKey: (row) => row.id,
    isSelectable: (row) => row.disabled !== true,
  });
  selection.select({ id: 4 });
  selection.extendTo({ id: 1 });
  assert.deepEqual(
    [...selection.selectedKeys.value].sort((left, right) => left - right),
    [1, 2, 4],
  );
  assert.equal(selection.anchor.value, 4);
  assert.equal(selection.extendTo({ id: 99 }), false);
});

void test("keys survive item replacement and default to the items themselves", () => {
  const items = shallowRef<Row[]>([{ id: 1 }, { id: 2 }]);
  const selection = useSelection({ items, multiple: true, getKey: (row) => row.id, initial: [2] });
  items.value = [{ id: 2 }, { id: 3 }];
  assert.deepEqual(
    selection.selected.value.map((row) => row.id),
    [2],
  );
  const plain = useSelection({ items: ["a", "b"] });
  plain.extendTo("b");
  assert.equal(plain.selected.value, "b");
  assert.equal(plain.isSelected("b"), true);
});
