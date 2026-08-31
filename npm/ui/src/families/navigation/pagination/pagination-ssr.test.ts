import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import PaginationRoot from "./pagination.vue";
import PaginationEllipsis from "./pagination-ellipsis.vue";
import PaginationItem from "./pagination-item.vue";
import PaginationList from "./pagination-list.vue";
import PaginationNext from "./pagination-next.vue";
import PaginationPage from "./pagination-page.vue";
import PaginationPrevious from "./pagination-previous.vue";
import type { PaginationSlotState } from "./pagination.ts";

function renderPaginationTree(state: PaginationSlotState) {
  return h(PaginationList, null, {
    default: () => [
      h(PaginationItem, null, () => h(PaginationPrevious, null, () => "Previous")),
      ...state.range.map((item) =>
        item.type === "page"
          ? h(PaginationItem, { key: item.key, page: item.page }, () =>
              h(PaginationPage, { page: item.page }, () => String(item.page)),
            )
          : h(PaginationItem, { key: item.key }, () =>
              h(PaginationEllipsis, { position: item.position }),
            ),
      ),
      h(PaginationItem, null, () => h(PaginationNext, null, () => "Next")),
    ],
  });
}

const SsrProbe = defineComponent({
  name: "PaginationSsrProbe",
  setup: () => () =>
    h(
      PaginationRoot,
      { defaultValue: 2, pageCount: 4 },
      {
        default: (state: PaginationSlotState) => renderPaginationTree(state),
      },
    ),
});

test("renders byte-identical pagination markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<nav/);
  assert.match(html, /id="vize-v-\d+-pagination"/);
  assert.match(html, /aria-label="Pagination"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="pagination"/);
  assert.match(html, /data-page="2"/);
  assert.match(html, /data-page-count="4"/);
  assert.match(html, /<ol/);
  assert.match(html, /id="vize-v-\d+-pagination-list"/);
  assert.match(html, /data-vize-ui="pagination-previous"/);
  assert.match(html, /data-target-page="1"/);
  assert.match(html, /id="vize-v-\d+-pagination-page-2"/);
  assert.match(html, /aria-current="page"/);
  assert.match(html, /data-current="true"/);
  assert.match(html, /data-vize-ui="pagination-next"/);
  assert.match(html, /data-target-page="3"/);
  assert.doesNotMatch(html, /class=/);
  assert.doesNotMatch(html, /style=/);
});

test("hydrates generated pagination ids without changing the server contract", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverControls = [...host.querySelectorAll<HTMLButtonElement>("button")];
  assert.ok(serverRoot);
  assert.equal(serverControls.length, 6);
  const controlIds = serverControls.map((control) => control.id);

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
    const hydratedControls = [...host.querySelectorAll<HTMLButtonElement>("button")];
    assert.ok(host.firstElementChild === serverRoot);
    assert.deepEqual(
      hydratedControls.map((control) => control.id),
      controlIds,
    );
    assert.equal(host.firstElementChild?.getAttribute("data-page"), "2");
    assert.equal(
      host.querySelector<HTMLButtonElement>('[data-vize-ui="pagination-page"][aria-current="page"]')
        ?.id,
      controlIds[2],
    );
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
