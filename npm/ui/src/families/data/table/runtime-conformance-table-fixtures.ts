import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "./table.ts";

function renderRuntimeTable() {
  return h(
    Table,
    { density: "compact", layout: "fixed" },
    {
      default: () => [
        h(TableCaption, { side: "bottom" }, { default: () => "Release health" }),
        h(TableHead, null, {
          default: () =>
            h(TableRow, null, {
              default: () => [
                h(TableHeader, { id: "signal", scope: "col" }, { default: () => "Signal" }),
                h(
                  TableHeader,
                  { align: "end", id: "value", scope: "col" },
                  { default: () => "Value" },
                ),
              ],
            }),
        }),
        h(TableBody, null, {
          default: () =>
            h(
              TableRow,
              { state: "selected" },
              {
                default: () => [
                  h(TableHeader, { scope: "row" }, { default: () => "CI" }),
                  h(
                    TableCell,
                    { align: "end", headers: "signal value" },
                    { default: () => "green" },
                  ),
                ],
              },
            ),
        }),
      ],
    },
  );
}

function assertServerMarkup(html: string): void {
  assert.match(html, /^<table/);
  assert.match(html, /data-vize-ui="table"/);
  assert.match(html, /data-layout="fixed"/);
  assert.match(html, /data-density="compact"/);
  assert.match(html, /<caption[^>]+data-vize-ui="table-caption"[^>]+data-caption-side="bottom"/);
  assert.match(html, /<thead[^>]+data-vize-ui="table-head"[^>]+data-section="head"/);
  assert.match(html, /<tbody[^>]+data-vize-ui="table-body"[^>]+data-section="body"/);
  assert.match(html, /<tr[^>]+data-vize-ui="table-row"[^>]+data-state="selected"/);
  assert.match(html, /<th[^>]+scope="row"[^>]+data-vize-ui="table-header"/);
  assert.match(html, /<td[^>]+headers="signal value"[^>]+data-vize-ui="table-cell"/);
  assert.doesNotMatch(html, /role="grid"/);
}

function assertHydratedDom(host: HTMLElement): void {
  const table = host.querySelector('[data-vize-ui="table"]');
  const caption = host.querySelector('[data-vize-ui="table-caption"]');
  const head = host.querySelector('[data-vize-ui="table-head"]');
  const body = host.querySelector('[data-vize-ui="table-body"]');
  const selectedRow = host.querySelector('[data-vize-ui="table-row"][data-selected="true"]');
  const header = host.querySelector('[data-vize-ui="table-header"][scope="row"]');
  const cell = host.querySelector('[data-vize-ui="table-cell"]');

  assert.ok(table instanceof HTMLTableElement);
  assert.ok(caption instanceof HTMLTableCaptionElement);
  assert.ok(head instanceof HTMLTableSectionElement);
  assert.ok(body instanceof HTMLTableSectionElement);
  assert.ok(selectedRow instanceof HTMLTableRowElement);
  assert.ok(header instanceof HTMLTableCellElement);
  assert.ok(cell instanceof HTMLTableCellElement);
  assert.equal(cell.getAttribute("headers"), "signal value");
  assert.equal(table.textContent?.replace(/\s+/g, " ").trim(), "Release healthSignalValueCIgreen");
}

function tableFixture(name: string, sourceFile: string): RuntimeFixture {
  return {
    name,
    sourceFile,
    render: renderRuntimeTable,
    assertServerMarkup,
    assertHydratedDom,
  };
}

export const tableRuntimeFixtures: readonly RuntimeFixture[] = [
  tableFixture("table-body", "families/data/table/table-body.vue"),
  tableFixture("table-caption", "families/data/table/table-caption.vue"),
  tableFixture("table-cell", "families/data/table/table-cell.vue"),
  tableFixture("table-head", "families/data/table/table-head.vue"),
  tableFixture("table-header", "families/data/table/table-header.vue"),
  tableFixture("table-row", "families/data/table/table-row.vue"),
  tableFixture("table", "families/data/table/table.vue"),
];
