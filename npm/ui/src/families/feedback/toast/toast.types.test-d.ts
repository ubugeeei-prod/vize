/** Compile-only assertions for the public Toast contract. */

import type {
  ToastDismissReason,
  ToastId,
  ToastOptions,
  ToastPriority,
  ToastRecord,
  ToastStore,
  ToastSwipeDirection,
  ToastType,
  ToastViewportSlotState,
} from "./toast.ts";
import {
  Toast,
  ToastAction,
  ToastClose,
  ToastProvider,
  ToastRoot,
  ToastViewport,
  Toaster,
  createToastStore,
  useToast,
} from "./toast.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Order {
  readonly orderId: number;
}

const store = createToastStore<Order>({ duration: 4000, limit: 2 });
const id = store.toast({ title: "Order placed", data: { orderId: 7 } });
const record = store.get(id);

type _StoreIsTyped = Expect<Equal<typeof store, ToastStore<Order>>>;
type _IdIsString = Expect<Equal<typeof id, ToastId>>;
type _RecordData = Expect<Equal<NonNullable<typeof record>["data"], Order | undefined>>;
type _TypeUnion = Expect<
  Equal<ToastType, "default" | "error" | "info" | "loading" | "success" | "warning">
>;
type _PriorityUnion = Expect<Equal<ToastPriority, "high" | "low" | "normal">>;
type _DirectionUnion = Expect<Equal<ToastSwipeDirection, "down" | "left" | "right" | "up">>;
type _ReasonUnion = Expect<
  Equal<ToastDismissReason, "action" | "api" | "close" | "escape" | "swipe" | "timeout">
>;
type _VisibleIsReadonly = Expect<
  Equal<typeof store.visibleToasts.value, readonly ToastRecord<Order>[]>
>;
type _SlotToast = Expect<Equal<ToastViewportSlotState<Order>["toast"], ToastRecord<Order>>>;

const upload: Promise<{ readonly name: string }> = Promise.resolve({ name: "report.pdf" });
const typedPromise = store.promise(upload, {
  loading: "Uploading",
  success: (value) => {
    type _ValueIsResolved = Expect<Equal<typeof value, { readonly name: string }>>;
    return `Uploaded ${value.name}`;
  },
  error: (reason) => {
    type _ReasonIsUnknown = Expect<Equal<typeof reason, unknown>>;
    return { title: "Failed", data: { orderId: 0 } };
  },
});
type _PromiseReturned = Expect<Equal<typeof typedPromise, Promise<{ readonly name: string }>>>;

const options: ToastOptions<Order> = {
  onDismiss: (toast, reason) => {
    type _DismissToast = Expect<Equal<typeof toast, ToastRecord<Order>>>;
    type _DismissReason = Expect<Equal<typeof reason, ToastDismissReason>>;
  },
};

const injected = useToast<Order>();
type _InjectedStore = Expect<Equal<typeof injected, ToastStore<Order>>>;

const actionProps: InstanceType<typeof ToastAction>["$props"] = { altText: "Undo in history" };
const closeProps: InstanceType<typeof ToastClose>["$props"] = { ariaLabel: "Close" };

// @ts-expect-error toast types are a closed union.
store.toast({ type: "danger" });

// @ts-expect-error priority is a closed union.
store.toast({ priority: "urgent" });

// @ts-expect-error data must match the store's Data.
store.toast({ data: { orderId: "7" } });

// @ts-expect-error variant helpers fix the type.
store.success({ type: "error" });

// @ts-expect-error actions require alt text.
store.toast({ action: { label: "Undo" } });

// @ts-expect-error ToastAction requires altText.
const badActionProps: InstanceType<typeof ToastAction>["$props"] = {};

void Toast;
void ToastProvider;
void ToastRoot;
void ToastViewport;
void Toaster;
void actionProps;
void badActionProps;
void closeProps;
void options;
void typedPromise;
