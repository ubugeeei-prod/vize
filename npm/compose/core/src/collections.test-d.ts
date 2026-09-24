/** Compile-only assertions for the collection, async, math, and machine contracts. */

import { computed, ref, shallowRef } from "vue";
import type { ComputedRef, ShallowRef, WritableComputedRef } from "vue";

import { useClamp, useProjection, useRound } from "./math.ts";
import {
  useArrayEvery,
  useArrayFilter,
  useArrayFind,
  useArrayMap,
  useArrayReduce,
  useArrayUnique,
} from "./use-array.ts";
import { useAsyncQueue } from "./use-async-queue.ts";
import type { AsyncQueueTaskResult } from "./use-async-queue.ts";
import { useAsyncState } from "./use-async-state.ts";
import { useConfirmDialog } from "./use-confirm-dialog.ts";
import type { ConfirmDialogResult } from "./use-confirm-dialog.ts";
import { useCycleList } from "./use-cycle-list.ts";
import { useMachine } from "./use-machine.ts";
import { useMemoize } from "./use-memoize.ts";
import { useOffsetPagination } from "./use-offset-pagination.ts";
import { useSelection } from "./use-selection.ts";
import { useSorted } from "./use-sorted.ts";
import { useStepper } from "./use-stepper.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

// Cycle lists: non-empty tuples never yield undefined.
const theme = useCycleList(["light", "dark"] as const);
type _TupleStateIsTheUnion = Expect<Equal<typeof theme.state, ShallowRef<"light" | "dark">>>;
const dynamic = useCycleList(ref<string[]>([]));
type _ArrayStateMayBeEmpty = Expect<Equal<typeof dynamic.state, ShallowRef<string | undefined>>>;

// Sorting: natural order only for naturally ordered items.
type _SortedNumbers = Expect<Equal<ReturnType<typeof useSorted<number>>, ComputedRef<number[]>>>;
useSorted([{ score: 1 }], (left, right) => left.score - right.score);
// @ts-expect-error objects need a comparator.
useSorted([{ score: 1 }]);

// Array helpers keep and narrow element types.
const mixed = ref<(string | number)[]>([]);
const strings = useArrayFilter(mixed, (value): value is string => typeof value === "string");
type _FilterNarrows = Expect<Equal<typeof strings, ComputedRef<string[]>>>;
const firstNumber = useArrayFind(mixed, (value): value is number => typeof value === "number");
type _FindNarrows = Expect<Equal<typeof firstNumber, ComputedRef<number | undefined>>>;
type _MapResult = Expect<
  Equal<ReturnType<typeof useArrayMap<number, string>>, ComputedRef<string[]>>
>;
const total = useArrayReduce([1, 2], (sum: string, value) => sum + value, "");
const maybeTotal = useArrayReduce([1, 2], (sum, value) => sum + value);
type _ReduceWithInitial = Expect<Equal<typeof total, ComputedRef<string>>>;
type _ReduceWithoutInitial = Expect<Equal<typeof maybeTotal, ComputedRef<number | undefined>>>;
useArrayEvery(mixed, (value) => value !== "") satisfies ComputedRef<boolean>;
useArrayUnique(mixed) satisfies ComputedRef<(string | number)[]>;

// Pagination: two-way refs.
const pager = useOffsetPagination({ page: ref(1), total: () => 10 });
type _PageIsWritable = Expect<Equal<typeof pager.currentPage, WritableComputedRef<number>>>;

// Stepper: step names are literal unions.
const wizard = useStepper(["account", "billing"]);
type _StepNames = Expect<Equal<typeof wizard.current.value, "account" | "billing">>;
wizard.goTo("billing");
// @ts-expect-error unknown step names are rejected.
wizard.goTo("shipping");
// @ts-expect-error the initial step must be declared.
useStepper(["a", "b"], "c");
const checkout = useStepper({ cart: { total: 1 }, done: { receipt: "r" } });
type _StepValue = Expect<Equal<ReturnType<typeof checkout.get<"done">>, { receipt: string }>>;

// Selection: mode switches the selected type; keys follow getKey.
const single = useSelection({ items: [{ id: 1 }], getKey: (item) => item.id });
type _SingleSelected = Expect<Equal<typeof single.selected.value, { id: number } | undefined>>;
type _SelectedKeys = Expect<Equal<typeof single.selectedKeys.value, readonly number[]>>;
const many = useSelection({ items: ["a", "b"], multiple: true });
type _MultipleSelected = Expect<Equal<typeof many.selected.value, string[]>>;
// @ts-expect-error initial keys follow getKey.
useSelection({ items: [{ id: 1 }], getKey: (item) => item.id, initial: ["1"] });

// Async state: arity-aware.
const user = useAsyncState((id: string) => Promise.resolve({ id }), null, { immediate: false });
type _ExecuteKeepsParameters = Expect<Equal<Parameters<typeof user.execute>, [id: string]>>;
type _StateUnion = Expect<Equal<typeof user.state.value, { id: string } | null>>;
// @ts-expect-error producers with parameters cannot run immediately.
useAsyncState((id: string) => Promise.resolve(id), null);

// Async queue: each task's parameter is the previous result.
const queue = useAsyncQueue([
  async () => 2,
  async (previous) => {
    type _SecondReceivesFirst = Expect<Equal<typeof previous, number>>;
    return String(previous);
  },
  (previous) => {
    type _ThirdReceivesSecond = Expect<Equal<typeof previous, string>>;
    return previous.length > 0;
  },
]);
type _QueueRecords = Expect<
  Equal<
    typeof queue.results.value,
    readonly [
      AsyncQueueTaskResult<number>,
      AsyncQueueTaskResult<string>,
      AsyncQueueTaskResult<boolean>,
    ]
  >
>;

// Memoize: key typing follows getKey.
const byId = useMemoize((user: { id: number }) => user.id * 2, { getKey: (user) => user.id });
type _CustomKey = Expect<Equal<ReturnType<typeof byId.generateKey>, number>>;
const byJson = useMemoize((left: number, right: number) => left + right);
type _DefaultKey = Expect<Equal<ReturnType<typeof byJson.generateKey>, string>>;
// @ts-expect-error memoized functions keep the resolver's parameters.
byJson("1", 2);
// @ts-expect-error a custom cache must use the derived key type.
useMemoize((value: number) => value, {
  getKey: (value) => value,
  cache: new Map<string, number>(),
});

// Confirm dialog: payloads and discriminated results.
const dialog = useConfirmDialog<{ file: string }, "keep" | "delete", Error>();
const outcome = dialog.reveal({ file: "a" });
type _RevealResult = Expect<
  Equal<Awaited<typeof outcome>, ConfirmDialogResult<"keep" | "delete", Error>>
>;
dialog.confirm("keep");
// @ts-expect-error confirm payload is typed.
dialog.confirm("maybe");
const voidDialog = useConfirmDialog();
void voidDialog.reveal();
// @ts-expect-error void payloads take no argument.
void voidDialog.reveal("x");

// Math.
const volume = useClamp(ref(1), 0, 10);
type _WritableClamp = Expect<Equal<typeof volume, WritableComputedRef<number>>>;
const readonlyClamp = useClamp(
  computed(() => 1),
  0,
  10,
);
type _ReadonlyClamp = Expect<Equal<typeof readonlyClamp, ComputedRef<number>>>;
useRound(shallowRef(1.5), { method: "floor" }) satisfies ComputedRef<number>;
// @ts-expect-error rounding methods are a closed union.
useRound(1, { method: "bankers" });
useProjection(1, [0, 1], () => [0, 100] as const) satisfies ComputedRef<number>;

// Machine: states, events, and targets are checked.
const door = useMachine({
  initial: "closed",
  context: { opens: 0 },
  states: {
    closed: {
      on: { OPEN: { target: "open", action: (context) => ({ opens: context.opens + 1 }) } },
    },
    open: { on: { CLOSE: "closed" } },
  },
});
type _MachineState = Expect<Equal<typeof door.state.value, "closed" | "open">>;
type _MachineEvents = Expect<Equal<Parameters<typeof door.send>[0], "OPEN" | "CLOSE">>;
type _MachineContext = Expect<Equal<typeof door.context.value, { opens: number }>>;
// @ts-expect-error unknown events are rejected.
door.send("LOCK");
// @ts-expect-error unknown states are rejected by matches.
door.matches("locked");
useMachine({
  initial: "a",
  context: undefined,
  // @ts-expect-error transition targets must be declared states.
  states: { a: { on: { GO: "missing" } }, b: {} },
});
// @ts-expect-error the initial state must be declared.
useMachine({ initial: "z", context: undefined, states: { a: {} } });
