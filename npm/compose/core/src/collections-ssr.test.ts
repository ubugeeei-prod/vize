/**
 * Server-rendering contract for the collection, async, math, and machine
 * composables: rendering twice yields identical markup equal to the
 * client's first render, and async work never blocks or changes it.
 */
import assert from "node:assert/strict";
import { test } from "node:test";
import { createSSRApp, defineComponent, h, shallowRef } from "vue";
import type { Component } from "vue";
import { renderToString } from "vue/server-renderer";

import { useClamp, useProjection, useRound } from "./math.ts";
import {
  useArrayEvery,
  useArrayFilter,
  useArrayFind,
  useArrayMap,
  useArrayReduce,
  useArraySome,
  useArrayUnique,
} from "./use-array.ts";
import { useAsyncQueue } from "./use-async-queue.ts";
import { useAsyncState } from "./use-async-state.ts";
import { useConfirmDialog } from "./use-confirm-dialog.ts";
import { useCycleList } from "./use-cycle-list.ts";
import { useMachine } from "./use-machine.ts";
import { useMemoize } from "./use-memoize.ts";
import { useOffsetPagination } from "./use-offset-pagination.ts";
import { useSelection } from "./use-selection.ts";
import { useSorted } from "./use-sorted.ts";
import { useStepper } from "./use-stepper.ts";

async function renderTwice(render: () => string): Promise<string> {
  const component = (): Component =>
    defineComponent({
      setup() {
        const text = render();
        return () => h("output", text);
      },
    });
  const first = await renderToString(createSSRApp(component()));
  const second = await renderToString(createSSRApp(component()));
  assert.equal(first, second, "server output must be deterministic");
  return first;
}

void test("collections render derived state synchronously", async () => {
  const html = await renderTwice(() => {
    const list = shallowRef([3, 1, 2, 2]);
    const cycle = useCycleList(["a", "b", "c"] as const, { initialValue: "b" });
    const pager = useOffsetPagination({ total: 95, page: 3 });
    const stepper = useStepper(["cart", "pay"], "pay");
    const selection = useSelection({ items: list, multiple: true, initial: [2] });
    return [
      useSorted(list).value.join(","),
      useArrayFilter(list, (value) => value > 1).value.join(","),
      useArrayMap(list, (value) => value * 2).value.join(","),
      useArrayFind(list, (value) => value === 2).value,
      useArrayReduce(list, (sum, value) => sum + value, 0).value,
      useArraySome(list, (value) => value > 2).value,
      useArrayEvery(list, (value) => value > 0).value,
      useArrayUnique(list).value.join(","),
      cycle.state.value,
      `${pager.currentPage.value}/${pager.pageCount.value}`,
      stepper.current.value,
      selection.selected.value.join(","),
    ].join("|");
  });
  assert.equal(html, "<output>1,2,2,3|3,2,2|6,2,4,4|2|8|true|true|3,1,2|b|3/10|pay|2,2</output>");
});

void test("async state renders its initial, loading snapshot", async () => {
  const html = await renderTwice(() => {
    const state = useAsyncState(async () => "late", "initial");
    const queue = useAsyncQueue([async () => 1, async (previous) => previous + 1]);
    const memoized = useMemoize((value: number) => value * 3);
    const dialog = useConfirmDialog();
    return [
      state.state.value,
      state.isLoading.value,
      queue.results.value.map((record) => record.state).join(","),
      memoized(2),
      dialog.state.value,
    ].join("|");
  });
  assert.equal(html, "<output>initial|true|running,pending|6|idle</output>");
});

void test("math and machines are pure functions of their inputs", async () => {
  const html = await renderTwice(() => {
    const machine = useMachine({
      initial: "closed",
      context: undefined,
      states: { closed: { on: { OPEN: "open" } }, open: { on: { CLOSE: "closed" } } },
    });
    machine.send("OPEN");
    return [
      useClamp(150, 0, 100).value,
      useRound(1.005, { precision: 2 }).value,
      useProjection(5, [0, 10], [0, 100]).value,
      machine.state.value,
    ].join("|");
  });
  assert.equal(html, "<output>100|1.01|50|open</output>");
});
