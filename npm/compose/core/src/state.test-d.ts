/** Compile-only assertions for the ref helper and state factory contracts. */

import { reactive, ref, shallowRef } from "vue";
import type { ComputedRef, InjectionKey, Ref, ShallowRef, WritableComputedRef } from "vue";

import { computedAsync } from "./computed-async.ts";
import { computedWithControl } from "./computed-with-control.ts";
import type { ComputedWithControl } from "./computed-with-control.ts";
import { createEventHook } from "./create-event-hook.ts";
import { createGlobalState } from "./create-global-state.ts";
import { createInjectionState } from "./create-injection-state.ts";
import { createSharedComposable } from "./create-shared-composable.ts";
import { reactify } from "./reactify.ts";
import { refAutoReset } from "./ref-auto-reset.ts";
import { refDebounced } from "./ref-debounced.ts";
import { refDefault } from "./ref-default.ts";
import { syncRef } from "./sync-ref.ts";
import { toReactive } from "./to-reactive.ts";
import { useIdGenerator } from "./use-id-generator.ts";
import { useMounted } from "./use-mounted.ts";
import { useSupported } from "./use-supported.ts";
import { useVModel, useVModels } from "./use-v-model.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type Status = "idle" | "saving";

const status = refAutoReset<Status>("idle", 1_000);
type _AutoResetKeepsTheUnion = Expect<Equal<typeof status, Ref<Status>>>;
// @ts-expect-error writes must match the default's type.
status.value = "done";

const nullable = ref<string | null | undefined>();
const withDefault = refDefault(nullable, "fallback");
type _DefaultReadsDropNullish = Expect<
  Equal<typeof withDefault, WritableComputedRef<string, string | null | undefined>>
>;
withDefault.value = null;
// @ts-expect-error the default must be non-nullable.
refDefault(nullable, null);

const debounced = refDebounced(shallowRef(1), 100);
type _DebouncedIsReadonly = Expect<Equal<typeof debounced, Readonly<ShallowRef<number>>>>;

const loaded = computedAsync(async ({ signal }): Promise<number> => (signal.aborted ? 0 : 1), 0);
const optional = computedAsync(async () => "value");
type _AsyncWithInitial = Expect<Equal<typeof loaded, Readonly<Ref<number>>>>;
type _AsyncWithoutInitial = Expect<Equal<typeof optional, Readonly<Ref<string | undefined>>>>;

const controlled = computedWithControl(
  shallowRef(0),
  (previous: number | undefined) => (previous ?? 0) + 1,
);
type _ControlledValue = Expect<Equal<typeof controlled, ComputedWithControl<number>>>;
controlled.trigger();

// syncRef: converters are optional only for identical types, and only the
// converters the direction needs are required.
syncRef(ref(1), ref(2));
syncRef(ref(1), ref(""), { transform: { ltr: String, rtl: Number } });
syncRef(ref(1), ref(""), { direction: "ltr", transform: { ltr: String } });
// @ts-expect-error different types need converters.
syncRef(ref(1), ref(""));
// @ts-expect-error both converters are needed for two-way syncing.
syncRef(ref(1), ref(""), { transform: { ltr: String } });
// @ts-expect-error converters must produce the other side's type.
syncRef(ref(1), ref(""), { direction: "ltr", transform: { ltr: (value: number) => value } });

// useVModel: typed from props and emit.
const props = reactive({ modelValue: "text", count: 1 as number | undefined });
declare const emit: {
  (event: "update:modelValue", value: string): void;
  (event: "update:count", value: number | undefined): void;
};
const model = useVModel(props, "modelValue", emit);
type _ModelFollowsTheProp = Expect<Equal<typeof model, Ref<string>>>;
const count = useVModel(props, "count", emit, { defaultValue: 0 });
type _DefaultRemovesUndefined = Expect<Equal<typeof count, Ref<number>>>;
declare const changeEmit: (event: "change", value: string) => void;
useVModel(props, "modelValue", changeEmit, { eventName: "change" });
// @ts-expect-error the emit must accept the update event of that prop.
useVModel(props, "modelValue", changeEmit);
// @ts-expect-error keys must be props.
useVModel(props, "missing", emit);
const models = useVModels(props, emit);
type _ModelsCoverEveryKey = Expect<
  Equal<
    typeof models,
    { readonly modelValue: Ref<string>; readonly count: Ref<number | undefined> }
  >
>;
declare const partialEmit: (event: "update:modelValue", value: string) => void;
useVModels(props, partialEmit, { keys: ["modelValue"] });
// @ts-expect-error binding every key requires an emit for every key.
useVModels(props, partialEmit);

const add = reactify((left: number, right: string) => `${left}${right}`);
const joined = add(ref(1), () => "x");
type _ReactifiedResult = Expect<Equal<typeof joined, ComputedRef<string>>>;
// @ts-expect-error reactified parameters keep their types.
add("1", "x");

const view = toReactive(ref({ nested: ref(1), plain: "a" }));
type _ToReactiveUnwraps = Expect<Equal<typeof view.nested, number>>;

const useGlobal = createGlobalState(() => ({ theme: shallowRef<"light" | "dark">("light") }));
type _GlobalStateAccessor = Expect<
  Equal<ReturnType<typeof useGlobal>, { theme: ShallowRef<"light" | "dark"> }>
>;

const describe = (id: string, options?: { live: boolean }) => ({ id, options });
const useShared = createSharedComposable(describe);
type _SharedKeepsTheSignature = Expect<
  Equal<typeof useShared, (...args: Parameters<typeof describe>) => ReturnType<typeof describe>>
>;

const [useProvide, useInject, key] = createInjectionState((initial: number) => ({ initial }));
type _ProviderArguments = Expect<Equal<Parameters<typeof useProvide>, [initial: number]>>;
type _InjectedMayBeMissing = Expect<
  Equal<ReturnType<typeof useInject>, { initial: number } | undefined>
>;
type _KeyIsTyped = Expect<Equal<typeof key, InjectionKey<{ initial: number }>>>;
const [, useInjectWithDefault] = createInjectionState(() => 1, { defaultValue: 0 });
type _DefaultNarrowsInjection = Expect<Equal<ReturnType<typeof useInjectWithDefault>, number>>;

const hook = createEventHook<[id: string, count: number]>();
hook.on((id, value) => `${id}${value}`);
void hook.trigger("a", 1);
// @ts-expect-error trigger checks the payload tuple.
void hook.trigger("a");
// @ts-expect-error listeners receive the payload types.
hook.on((id: number) => id);
const emptyHook = createEventHook({ serverListeners: "register" });
// @ts-expect-error the server policy is a closed union.
createEventHook({ serverListeners: "always" });
void emptyHook.trigger();

useIdGenerator({ prefix: "x" })("hint") satisfies string;
type _MountedIsReadonly = Expect<
  Equal<ReturnType<typeof useMounted>, Readonly<ShallowRef<boolean>>>
>;
useSupported(() => "share" in globalThis) satisfies ComputedRef<boolean>;
