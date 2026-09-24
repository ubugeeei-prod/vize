import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import SelectContent from "../select/select-content.vue";
import SelectItem from "../select/select-item.vue";
import SelectViewport from "../select/select-viewport.vue";
import SelectVirtualizer from "../select/select-virtualizer.vue";
import ComboboxInput from "./combobox-input.vue";
import ComboboxRoot from "./combobox-root.vue";
import type { ComboboxLoadContext, ComboboxSlotState } from "./combobox-types.ts";
import {
  cities,
  comboboxInput,
  keydown,
  comboboxOptions,
  settle,
  type,
  visibleOptions,
} from "./combobox-test-utils.ts";
import type { City } from "./combobox-test-utils.ts";

function mountCombobox(
  rootProps: Record<string, unknown> = {},
  options: Parameters<typeof comboboxOptions>[1] = {},
) {
  return mountInteraction(ComboboxRoot, comboboxOptions(rootProps, options));
}

interface Deferred<T> {
  readonly promise: Promise<T>;
  readonly resolve: (value: T) => void;
  readonly reject: (error: unknown) => void;
}

function deferred<T>(): Deferred<T> {
  let resolve: (value: T) => void = () => {};
  let reject: (error: unknown) => void = () => {};
  const promise = new Promise<T>((onResolve, onReject) => {
    resolve = onResolve;
    reject = onReject;
  });
  return { promise, reject, resolve };
}

test("loadItems runs on open and per query, exposes loading, and ignores stale responses", async () => {
  const requests: { query: string; signal: AbortSignal; result: Deferred<readonly City[]> }[] = [];
  const handle = mountCombobox(
    {
      debounce: 0,
      loadItems: (query: string, context: ComboboxLoadContext) => {
        const result = deferred<readonly City[]>();
        requests.push({ query, result, signal: context.signal });
        return result.promise;
      },
    },
    { itemsMode: true },
  );
  const input = comboboxInput(handle);
  keydown(input, "ArrowDown");
  await settle();
  assert.deepEqual(
    requests.map((request) => request.query),
    [""],
  );
  assert.equal(handle.root().getAttribute("data-loading"), "true");
  assert.equal(
    handle.root().querySelector("[data-vize-ui='combobox-loading']")?.textContent,
    "Loading…",
  );
  assert.equal(
    handle.root().querySelector("[data-vize-ui='combobox-empty']")?.hasAttribute("hidden"),
    true,
    "empty stays hidden while loading",
  );

  await type(input, "b");
  assert.deepEqual(
    requests.map((request) => request.query),
    ["", "b"],
  );
  assert.equal(requests[0]?.signal.aborted, true, "a newer query aborts the older request");
  requests[0]?.result.resolve(cities);
  requests[1]?.result.resolve(cities.slice(0, 3));
  await settle();
  assert.deepEqual(visibleOptions(handle), ["Berlin", "Bogotá", "Boston"]);
  assert.equal(handle.root().getAttribute("data-loading"), null);

  await type(input, "zz");
  requests[2]?.result.reject(new Error("offline"));
  await settle();
  assert.equal(handle.root().getAttribute("data-loading"), null);
  assert.deepEqual(visibleOptions(handle), ["Berlin", "Bogotá", "Boston"], "keeps last results");
  handle.unmount();
  assert.equal(requests[2]?.signal.aborted, true, "unmount aborts the pending request");
});

test("loadItems is debounced", async () => {
  const queries: string[] = [];
  const handle = mountCombobox(
    {
      debounce: 20,
      loadItems: (query: string) => {
        queries.push(query);
        return cities;
      },
    },
    { itemsMode: true },
  );
  const input = comboboxInput(handle);
  await type(input, "b");
  await type(input, "bo");
  await type(input, "bos");
  await new Promise((resolve) => setTimeout(resolve, 40));
  await settle();
  assert.deepEqual(
    queries,
    ["b", "bos"],
    "opening by typing loads at once; later keystrokes load only the settled query",
  );
  handle.unmount();
});

test("a debounced query aborts the previous request before the timer fires", async () => {
  const requests: { query: string; signal: AbortSignal; result: Deferred<readonly City[]> }[] = [];
  const handle = mountCombobox(
    {
      debounce: 30,
      loadItems: (query: string, context: ComboboxLoadContext) => {
        const result = deferred<readonly City[]>();
        requests.push({ query, result, signal: context.signal });
        return result.promise;
      },
    },
    { itemsMode: true },
  );
  const input = comboboxInput(handle);
  keydown(input, "ArrowDown");
  await settle();
  assert.equal(requests.length, 1);

  await type(input, "bo");
  assert.equal(requests[0]?.signal.aborted, true);
  requests[0]?.result.resolve(cities);
  await settle();
  assert.deepEqual(visibleOptions(handle), [], "the stale result never appears during debounce");

  await new Promise((resolve) => setTimeout(resolve, 40));
  assert.equal(requests[1]?.query, "bo");
  requests[1]?.result.resolve(cities.slice(0, 3));
  await settle();
  assert.deepEqual(visibleOptions(handle), ["Berlin", "Bogotá", "Boston"]);
  handle.unmount();
});

interface Row {
  readonly id: number;
  readonly label: string;
}

const rows: readonly Row[] = Object.freeze(
  Array.from({ length: 2000 }, (_, index) => ({ id: index, label: `Row ${index}` })),
);

test("virtualized combobox lists navigate filtered results beyond the window", async () => {
  const Probe = defineComponent({
    setup: () => () =>
      h(
        ComboboxRoot<Row>,
        { by: "id", itemText: (row: Row) => row.label, items: rows },
        {
          default: (state: ComboboxSlotState<Row>) => [
            h(ComboboxInput, { ariaLabel: "Rows" }),
            h(SelectContent, { portalDisabled: true }, () =>
              h(SelectViewport, null, () =>
                h(
                  SelectVirtualizer<Row>,
                  {
                    estimateItemSize: 20,
                    initialViewportHeight: 100,
                    items: state.filteredItems,
                    overscan: 2,
                  },
                  {
                    default: ({ index, item }: { readonly index: number; readonly item: Row }) =>
                      h(SelectItem<Row>, { index, value: item }, () => item.label),
                  },
                ),
              ),
            ),
          ],
        },
      ),
  });
  const handle = mountInteraction(Probe, { record: [] });
  const input = handle.getByRole("combobox", { name: "Rows" });
  if (!(input instanceof HTMLInputElement)) assert.fail("input required");
  input.focus();
  await type(input, "Row 19");
  const count = handle.root().querySelector("[data-vize-ui='combobox-virtualizer']");
  assert.equal(count?.getAttribute("data-count"), "111", "Row 19, Row 190–199, Row 1900–1999");
  assert.ok(handle.root().querySelectorAll("[role='option']").length < 20);

  const activeText = () => {
    const id = input.getAttribute("aria-activedescendant");
    return id === null ? null : handle.root().querySelector(`[id="${id}"]`)?.textContent;
  };
  assert.equal(activeText(), "Row 19");
  for (let page = 0; page < 3; page++) {
    keydown(input, "PageDown");
    await settle();
  }
  assert.equal(activeText(), "Row 1919", "index 30 lives outside the initial window");
  keydown(input, "Enter");
  await settle();
  assert.equal(input.value, "Row 1919");
  handle.unmount();
});
