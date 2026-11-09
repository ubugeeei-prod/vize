import assert from "node:assert/strict";
import { test } from "vitest";
import { h, ref } from "vue";

import { dispatchKey, findNodes, mountFresco } from "../testing/mount.js";
import { Table } from "./Table.js";
import { fitTableCell, sortTableRows, tableViewportRange } from "./table-utils.js";

test("table helpers preserve graphemes, stable ties, and bounded viewport ranges", () => {
  assert.equal(fitTableCell("A👍🏽B", 2), "A👍🏽");
  assert.equal(fitTableCell("A", 3, "right"), "  A");
  assert.equal(fitTableCell("A", 4, "center"), " A  ");

  const rows = [
    { id: "first", score: 2 },
    { id: "second", score: 1 },
    { id: "third", score: 2 },
  ];
  const sorted = sortTableRows(rows, [{ key: "score", header: "Score" }], "score", "descending");
  assert.deepEqual(
    sorted.map(({ row }) => row.id),
    ["first", "third", "second"],
  );
  assert.deepEqual(tableViewportRange(100_000, 50_000, 8, 2), {
    visibleStart: 50_000,
    visibleEnd: 50_008,
    retainedStart: 49_998,
    retainedEnd: 50_010,
  });
});

test("Table retains only the viewport and overscan for large data", () => {
  const rows = Array.from({ length: 100_000 }, (_, id) => ({ id, label: `Row ${id}` }));
  const mounted = mountFresco(() =>
    h(Table, {
      columns: [{ key: "label", header: "Label", width: 12 }],
      data: rows,
      offset: 50_000,
      viewportSize: 4,
      overscan: 2,
    }),
  );

  const retained = findNodes(mounted.root, (node) => node.props["data-table-index"] !== undefined);
  assert.equal(retained.length, 8);
  assert.deepEqual(
    retained.map((node) => node.props["data-table-index"]),
    [49_998, 49_999, 50_000, 50_001, 50_002, 50_003, 50_004, 50_005],
  );
  assert.equal(retained.filter((node) => node.props["aria-hidden"] === false).length, 4);
  mounted.unmount();
});

test("Table owns controlled sort, column navigation, resize, scrolling, and cell slots", async () => {
  const sortKey = ref<string>();
  const sortDirection = ref<"ascending" | "descending">("ascending");
  const focusedColumn = ref(0);
  const columnWidths = ref<Readonly<Record<string, number>>>({ name: 8, score: 4 });
  const offset = ref(0);
  const mounted = mountFresco(() =>
    h(
      Table,
      {
        columns: [
          { key: "name", header: "Name", sortable: true, width: 8 },
          { key: "score", header: "Score", sortable: true, width: 4, minWidth: 3 },
        ],
        data: [
          { name: "Ada", score: 2 },
          { name: "Grace", score: 3 },
          { name: "Linus", score: 1 },
        ],
        focused: true,
        focusedColumn: focusedColumn.value,
        sortKey: sortKey.value,
        sortDirection: sortDirection.value,
        columnWidths: columnWidths.value,
        resizeStep: 2,
        offset: offset.value,
        viewportSize: 2,
        overscan: 0,
        "onUpdate:focusedColumn": (next: number) => {
          focusedColumn.value = next;
        },
        "onUpdate:sortKey": (next: string) => {
          sortKey.value = next;
        },
        "onUpdate:sortDirection": (next: "ascending" | "descending") => {
          sortDirection.value = next;
        },
        "onUpdate:columnWidths": (next: Readonly<Record<string, number>>) => {
          columnWidths.value = next;
        },
        "onUpdate:offset": (next: number) => {
          offset.value = next;
        },
      },
      {
        cell: ({ content }: { content: string }) => h("text", { text: `[${content}]` }),
      },
    ),
  );

  await dispatchKey({ key: "right" });
  assert.equal(focusedColumn.value, 1);
  await dispatchKey({ key: "enter" });
  assert.equal(sortKey.value, "score");
  assert.equal(sortDirection.value, "ascending");
  await dispatchKey({ key: "enter" });
  assert.equal(sortDirection.value, "descending");
  await dispatchKey({ key: "right", shift: true });
  assert.deepEqual(columnWidths.value, { name: 8, score: 6 });
  await dispatchKey({ key: "down" });
  assert.equal(offset.value, 1);

  const slottedText = findNodes(
    mounted.root,
    (node) => node.type === "text" && String(node.props.text).startsWith("["),
  );
  assert.equal(slottedText.length, 4, "two visible rows retain two slotted cells each");
  mounted.unmount();
});
