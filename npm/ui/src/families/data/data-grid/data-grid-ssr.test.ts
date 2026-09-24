import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { DataGrid } from "./data-grid.ts";
import { people, personColumns, tree } from "./data-grid-fixture.ts";
import type { Person, PersonColumn } from "./data-grid-fixture.ts";

const PersonGrid = DataGrid<Person, PersonColumn>;

function probe(props: Record<string, unknown>) {
  return defineComponent({
    setup: () => () =>
      h("div", [
        h(PersonGrid, {
          ariaLabel: "People",
          rows: people,
          columns: personColumns,
          getRowId: (row: Person) => row.id,
          ...props,
        }),
      ]),
  });
}

async function hydrate(
  component: ReturnType<typeof probe>,
): Promise<{ html: string; diagnostics: string[] }> {
  const html = await renderToString(createSSRApp(component));
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(component);
  try {
    app.mount(host);
    await nextTick();
    assert.ok(host.firstElementChild === serverRoot, "hydration reused the server root");
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
  return { html, diagnostics };
}

test("renders byte-identical sorted, selected grid markup across SSR requests", async () => {
  const Probe = probe({
    id: "people",
    selectionMode: "multiple",
    defaultState: { sorting: [{ columnId: "age", direction: "descending" }], selection: ["ada"] },
  });
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /role="grid"/);
  assert.match(left, /aria-rowcount="5"/);
  assert.match(left, /aria-sort="descending"/);
  assert.match(left, /aria-multiselectable="true"/);
  assert.match(left, /data-row-id="ada"[^>]*data-selected="true"|aria-selected="true"/);
  assert.ok(left.indexOf('data-row-id="grace"') < left.indexOf('data-row-id="ada"'));
});

test("hydrates plain, tree, and virtualized grids without diagnostics", async () => {
  for (const props of [
    { id: "plain" },
    {
      id: "tree",
      rows: tree,
      getSubRows: (row: Person) => row.reports,
      defaultState: { expanded: ["ceo"] },
    },
    { id: "virtual", virtualize: true, rowHeight: 20, initialViewportHeight: 40, overscan: 0 },
  ]) {
    const { html, diagnostics } = await hydrate(probe(props));
    assert.deepEqual(diagnostics, [], `${props.id} hydrated with warnings`);
    if (props.id === "tree") assert.match(html, /role="treegrid"[\s\S]*aria-level="2"/);
    if (props.id === "virtual") assert.ok((html.match(/role="row"/g)?.length ?? 0) < 5);
  }
});
