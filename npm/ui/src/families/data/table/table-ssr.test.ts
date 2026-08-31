import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "./table.ts";

const SsrProbe = defineComponent({
  name: "TableSsrProbe",
  setup: () => () =>
    h(
      Table,
      {
        density: "compact",
        layout: "fixed",
      },
      {
        default: () => [
          h(TableCaption, { side: "bottom" }, { default: () => "Revenue by quarter" }),
          h(TableHead, null, {
            default: () =>
              h(TableRow, null, {
                default: () => [
                  h(TableHeader, { id: "quarter", scope: "col" }, { default: () => "Quarter" }),
                  h(
                    TableHeader,
                    { align: "end", id: "amount", scope: "col" },
                    { default: () => "Amount" },
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
                    h(TableHeader, { scope: "row" }, { default: () => "Q1" }),
                    h(
                      TableCell,
                      { align: "end", headers: "quarter amount" },
                      { default: () => "$42" },
                    ),
                  ],
                },
              ),
          }),
        ],
      },
    ),
});

test("renders byte-identical native table markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<table/);
  assert.match(html, /data-vize-ui="table"/);
  assert.match(html, /data-layout="fixed"/);
  assert.match(html, /data-density="compact"/);
  assert.match(html, /--vize-ui-table-layout:fixed/);
  assert.match(html, /<caption[^>]+data-vize-ui="table-caption"[^>]+data-caption-side="bottom"/);
  assert.match(html, /<thead[^>]+data-vize-ui="table-head"[^>]+data-section="head"/);
  assert.match(html, /<tbody[^>]+data-vize-ui="table-body"[^>]+data-section="body"/);
  assert.match(html, /<tr[^>]+data-vize-ui="table-row"[^>]+data-state="selected"/);
  assert.match(html, /<th[^>]+scope="row"/);
  assert.match(html, /<td[^>]+headers="quarter amount"/);
  assert.doesNotMatch(html, /role="grid"/);
});

test("hydrates semantic table markup without replacing SSR nodes", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;

  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);
  let mounted = false;

  try {
    app.mount(host);
    mounted = true;
    const table = host.querySelector('[data-vize-ui="table"]');
    const cell = host.querySelector('[data-vize-ui="table-cell"]');
    assert.ok(host.firstElementChild === serverRoot);
    assert.ok(table instanceof HTMLTableElement);
    assert.ok(cell instanceof HTMLTableCellElement);
    assert.equal(cell.getAttribute("headers"), "quarter amount");
    assert.equal(
      table.textContent?.replace(/\s+/g, " ").trim(),
      "Revenue by quarterQuarterAmountQ1$42",
    );
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
