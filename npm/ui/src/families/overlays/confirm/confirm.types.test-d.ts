/** Compile-only assertions for the public Confirm contract. */

import type { ComputedRef } from "vue";

import type {
  ConfirmApi,
  ConfirmProviderExpose,
  ConfirmRequest,
  ConfirmRequestKind,
  ConfirmSlotProps,
  ConfirmState,
} from "./confirm.ts";
import { ConfirmProvider, useConfirm } from "./confirm.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Invoice {
  readonly number: string;
}

declare const api: ConfirmApi<Invoice>;
declare const exposed: ConfirmProviderExpose;
declare const slot: ConfirmSlotProps;
declare const request: ConfirmRequest;

type _State = Expect<Equal<ConfirmState, "idle" | "pending">>;
type _Kind = Expect<Equal<ConfirmRequestKind, "choose" | "confirm">>;
type _ConfirmResult = Expect<Equal<ReturnType<typeof api.confirm>, Promise<boolean>>>;
type _Pending = Expect<Equal<typeof api.pending, ComputedRef<number>>>;
type _ExposedPending = Expect<Equal<typeof exposed.pending, number>>;
type _ExposedActive = Expect<Equal<typeof exposed.active, ConfirmRequest | null>>;
type _SlotRequest = Expect<Equal<typeof slot.request, ConfirmRequest>>;
type _RequestData = Expect<Equal<typeof request.data, unknown>>;

const choice = api.choose({
  title: "Unsaved changes",
  actions: [
    { value: "save", label: "Save" },
    { value: "discard", label: "Discard", destructive: true },
  ],
});
type _ChoiceIsInferred = Expect<Equal<typeof choice, Promise<"discard" | "save" | null>>>;

void api.confirm({ title: "Delete?", data: { number: "INV-1" } });

// @ts-expect-error data must match the Data type argument.
void api.confirm({ title: "Delete?", data: { id: 1 } });

// @ts-expect-error title is required.
void api.confirm({ description: "Missing title" });

// @ts-expect-error choose requires at least one action.
void api.choose({ title: "Pick", actions: [] });

const providerProps: InstanceType<typeof ConfirmProvider>["$props"] = {
  cancelLabel: "Keep",
  confirmLabel: "OK",
  portalDisabled: true,
};

// @ts-expect-error portalDisabled is boolean-only.
const badProviderProps: InstanceType<typeof ConfirmProvider>["$props"] = { portalDisabled: "no" };

const typed: () => ConfirmApi<Invoice> = () => useConfirm<Invoice>();

void badProviderProps;
void providerProps;
void typed;
