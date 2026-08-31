import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import TableBody from "./table-body.vue";
import TableCaption from "./table-caption.vue";
import TableCell from "./table-cell.vue";
import TableHead from "./table-head.vue";
import TableHeader from "./table-header.vue";
import TableRow from "./table-row.vue";
import Table from "./table.vue";
import type {
  TableBodyExpose,
  TableCaptionExpose,
  TableCellExpose,
  TableExpose,
  TableHeadExpose,
  TableHeaderExpose,
  TableRowExpose,
  TableSlotState,
} from "./table.ts";

function renderBasicTable() {
  return h(
    Table,
    { density: "normal", layout: "auto" },
    {
      default: () => [
        h(TableCaption, null, { default: () => "Quarterly revenue" }),
        h(TableHead, null, {
          default: () =>
            h(TableRow, null, {
              default: () => [
                h(TableHeader, { id: "quarter" }, { default: () => "Quarter" }),
                h(TableHeader, { align: "end", id: "revenue" }, { default: () => "Revenue" }),
              ],
            }),
        }),
        h(TableBody, null, {
          default: () =>
            h(TableRow, null, {
              default: () => [
                h(TableHeader, { scope: "row" }, { default: () => "Q1" }),
                h(
                  TableCell,
                  { align: "end", headers: "quarter revenue" },
                  { default: () => "$42" },
                ),
              ],
            }),
        }),
      ],
    },
  );
}

test("renders native table composition without grid machinery or visual CSS", async () => {
  const handle = mountInteraction(renderBasicTable);
  const table = handle.root() as HTMLTableElement;
  const caption = table.querySelector('[data-vize-ui="table-caption"]');
  const head = table.querySelector('[data-vize-ui="table-head"]');
  const body = table.querySelector('[data-vize-ui="table-body"]');
  const rows = table.querySelectorAll('[data-vize-ui="table-row"]');
  const headers = table.querySelectorAll('[data-vize-ui="table-header"]');
  const cells = table.querySelectorAll('[data-vize-ui="table-cell"]');

  assert.equal(table.tagName, "TABLE");
  assert.equal(table.getAttribute("part"), "root");
  assert.equal(table.getAttribute("data-vize-ui"), "table");
  assert.equal(table.getAttribute("data-layout"), "auto");
  assert.equal(table.getAttribute("data-density"), "normal");
  assert.equal(table.getAttribute("role"), null);
  assert.equal(table.getAttribute("tabindex"), null);
  assert.equal(table.getAttribute("aria-hidden"), null);
  assert.equal(table.getAttribute("aria-live"), null);
  assert.equal(table.getAttribute("class"), null);
  assert.equal(table.style.getPropertyValue("--vize-ui-table-layout"), "auto");
  assert.equal(caption?.tagName, "CAPTION");
  assert.equal(caption?.getAttribute("part"), "caption");
  assert.equal(caption?.getAttribute("data-caption-side"), "top");
  assert.equal(head?.tagName, "THEAD");
  assert.equal(head?.getAttribute("part"), "head");
  assert.equal(head?.getAttribute("data-section"), "head");
  assert.equal(body?.tagName, "TBODY");
  assert.equal(body?.getAttribute("part"), "body");
  assert.equal(body?.getAttribute("data-section"), "body");
  assert.equal(rows.length, 2);
  assert.equal(headers.length, 3);
  assert.equal(cells.length, 1);
  assert.equal(headers[0]?.tagName, "TH");
  assert.equal(headers[0]?.getAttribute("scope"), "col");
  assert.equal(cells[0]?.tagName, "TD");
  assert.equal(table.querySelector('[role="grid"]'), null);
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("mirrors semantic hooks and native span/idref attributes", () => {
  const handle = mountInteraction(Table, {
    attrs: {
      "aria-describedby": "report-help",
      "data-owner": "consumer",
      id: "report-table",
    },
    props: {
      density: "compact",
      layout: "fixed",
    },
    slots: {
      default: () => [
        h(TableCaption, { side: "bottom" }, { default: () => "Revenue by product" }),
        h(TableBody, null, {
          default: () =>
            h(
              TableRow,
              { state: "selected" },
              {
                default: () => [
                  h(
                    TableHeader,
                    {
                      abbr: "Product",
                      align: "center",
                      colspan: 2,
                      id: "product",
                      rowspan: 1,
                      scope: "row",
                    },
                    { default: () => "Product" },
                  ),
                  h(
                    TableCell,
                    {
                      align: "end",
                      colspan: 3,
                      headers: "product total",
                      rowspan: 2,
                    },
                    { default: () => "$128" },
                  ),
                ],
              },
            ),
        }),
      ],
    },
  });
  const table = handle.root() as HTMLTableElement;
  const caption = table.querySelector('[data-vize-ui="table-caption"]') as HTMLTableCaptionElement;
  const row = table.querySelector('[data-vize-ui="table-row"]') as HTMLTableRowElement;
  const header = table.querySelector('[data-vize-ui="table-header"]') as HTMLTableCellElement;
  const cell = table.querySelector('[data-vize-ui="table-cell"]') as HTMLTableCellElement;

  assert.equal(table.id, "report-table");
  assert.equal(table.getAttribute("aria-describedby"), "report-help");
  assert.equal(table.getAttribute("data-owner"), "consumer");
  assert.equal(table.getAttribute("data-layout"), "fixed");
  assert.equal(table.getAttribute("data-density"), "compact");
  assert.equal(table.style.getPropertyValue("--vize-ui-table-layout"), "fixed");
  assert.equal(caption.getAttribute("data-caption-side"), "bottom");
  assert.equal(caption.style.getPropertyValue("--vize-ui-table-caption-side"), "bottom");
  assert.equal(row.getAttribute("data-state"), "selected");
  assert.equal(row.getAttribute("data-selected"), "true");
  assert.equal(header.getAttribute("scope"), "row");
  assert.equal(header.getAttribute("abbr"), "Product");
  assert.equal(header.getAttribute("colspan"), "2");
  assert.equal(header.getAttribute("rowspan"), "1");
  assert.equal(header.getAttribute("data-scope"), "row");
  assert.equal(header.getAttribute("data-align"), "center");
  assert.equal(header.style.getPropertyValue("--vize-ui-table-cell-align"), "center");
  assert.equal(cell.getAttribute("headers"), "product total");
  assert.equal(cell.getAttribute("colspan"), "3");
  assert.equal(cell.getAttribute("rowspan"), "2");
  assert.equal(cell.getAttribute("data-align"), "end");
  assert.equal(cell.style.getPropertyValue("--vize-ui-table-cell-align"), "end");
  assert.equal(cell.textContent, "$128");
  handle.unmount();
});

test("passes slot state and exposes live table state", async () => {
  const handle = mountInteraction(Table, {
    props: {
      density: "spacious",
      layout: "fixed",
    },
    slots: {
      default: (state: TableSlotState) =>
        `${state.layout}:${state.density}:${state.style["--vize-ui-table-layout"]}`,
    },
  });
  const exposed = handle.exposes<TableExpose>();
  const root = handle.root();

  assert.ok(exposed.element === root);
  assert.equal(exposed.layout, "fixed");
  assert.equal(exposed.density, "spacious");
  assert.equal(exposed.style["--vize-ui-table-layout"], "fixed");
  assert.equal(root.textContent, "fixed:spacious:fixed");

  await handle.wrapper.setProps({ density: "compact", layout: "auto" });

  assert.equal(exposed.layout, "auto");
  assert.equal(exposed.density, "compact");
  assert.equal(exposed.style["--vize-ui-table-layout"], "auto");
  assert.equal(handle.root().getAttribute("data-layout"), "auto");
  assert.equal(handle.root().getAttribute("data-density"), "compact");
  assert.equal(handle.root().textContent, "auto:compact:auto");
  handle.unmount();
});

test("passes compound slot state and exposes native element contracts", async () => {
  const caption = mountInteraction(TableCaption, {
    props: { side: "bottom" },
    slots: {
      default: (state) => `${state.side}:${state.style["--vize-ui-table-caption-side"]}`,
    },
  });
  const captionExpose = caption.exposes<TableCaptionExpose>();
  assert.ok(captionExpose.element === caption.root());
  assert.equal(captionExpose.side, "bottom");
  assert.equal(caption.root().textContent, "bottom:bottom");
  await caption.wrapper.setProps({ side: "top" });
  assert.equal(captionExpose.side, "top");
  assert.equal(caption.root().textContent, "top:top");
  caption.unmount();

  const head = mountInteraction(TableHead, {
    slots: { default: (state) => h("tr", state.section) },
  });
  const headExpose = head.exposes<TableHeadExpose>();
  assert.ok(headExpose.element === head.root());
  assert.equal(headExpose.section, "head");
  assert.equal(head.root().textContent, "head");
  head.unmount();

  const body = mountInteraction(TableBody, {
    slots: { default: (state) => h("tr", state.section) },
  });
  const bodyExpose = body.exposes<TableBodyExpose>();
  assert.ok(bodyExpose.element === body.root());
  assert.equal(bodyExpose.section, "body");
  assert.equal(body.root().textContent, "body");
  body.unmount();

  const row = mountInteraction(TableRow, {
    props: { state: "selected" },
    slots: { default: (state) => `${state.state}:${state.selected}` },
  });
  const rowExpose = row.exposes<TableRowExpose>();
  assert.ok(rowExpose.element === row.root());
  assert.equal(rowExpose.state, "selected");
  assert.equal(rowExpose.selected, true);
  assert.equal(row.root().textContent, "selected:true");
  await row.wrapper.setProps({ state: "default" });
  assert.equal(rowExpose.state, "default");
  assert.equal(rowExpose.selected, false);
  assert.equal(row.root().getAttribute("data-selected"), null);
  assert.equal(row.root().textContent, "default:false");
  row.unmount();

  const header = mountInteraction(TableHeader, {
    props: {
      abbr: "Total",
      align: "end",
      colspan: 2,
      rowspan: 1,
      scope: "colgroup",
    },
    slots: {
      default: (state) =>
        `${state.scope}:${state.abbr}:${state.align}:${state.colspan}:${state.rowspan}`,
    },
  });
  const headerExpose = header.exposes<TableHeaderExpose>();
  assert.ok(headerExpose.element === header.root());
  assert.equal(headerExpose.scope, "colgroup");
  assert.equal(headerExpose.abbr, "Total");
  assert.equal(headerExpose.align, "end");
  assert.equal(headerExpose.colspan, 2);
  assert.equal(headerExpose.rowspan, 1);
  assert.equal(header.root().textContent, "colgroup:Total:end:2:1");
  await header.wrapper.setProps({ align: "center", scope: "rowgroup" });
  assert.equal(headerExpose.scope, "rowgroup");
  assert.equal(headerExpose.align, "center");
  assert.equal(header.root().getAttribute("data-align"), "center");
  header.unmount();

  const cell = mountInteraction(TableCell, {
    props: {
      align: "center",
      colspan: 4,
      headers: "product total",
      rowspan: 2,
    },
    slots: {
      default: (state) => `${state.headers}:${state.align}:${state.colspan}:${state.rowspan}`,
    },
  });
  const cellExpose = cell.exposes<TableCellExpose>();
  assert.ok(cellExpose.element === cell.root());
  assert.equal(cellExpose.headers, "product total");
  assert.equal(cellExpose.align, "center");
  assert.equal(cellExpose.colspan, 4);
  assert.equal(cellExpose.rowspan, 2);
  assert.equal(cell.root().textContent, "product total:center:4:2");
  await cell.wrapper.setProps({ align: "start", headers: undefined });
  assert.equal(cellExpose.headers, undefined);
  assert.equal(cellExpose.align, "start");
  assert.equal(cell.root().getAttribute("headers"), null);
  cell.unmount();
});
