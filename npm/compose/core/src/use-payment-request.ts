import { computed, readonly, shallowRef, toValue, unref } from "vue";
import type { ComputedRef, MaybeRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Monetary amount. */
export interface PaymentAmount {
  /** ISO 4217 currency code. */
  readonly currency: string;
  /** Decimal amount, for example `"9.99"`. */
  readonly value: string;
}

/** Line item shown on the payment sheet. */
export interface PaymentLineItem {
  /** Human-readable label. */
  readonly label: string;
  /** Amount. */
  readonly amount: PaymentAmount;
  /** Whether the amount is not final yet. */
  readonly pending?: boolean;
}

/** Selectable shipping option. */
export interface PaymentShippingOption {
  /** Option id reported back in `shippingOption`. */
  readonly id: string;
  /** Human-readable label. */
  readonly label: string;
  /** Price of the option. */
  readonly amount: PaymentAmount;
  /** Whether the option is preselected. */
  readonly selected?: boolean;
}

/** Accepted payment method. */
export interface PaymentMethod {
  /** Payment method identifier (URL or standardized name). */
  readonly supportedMethods: string;
  /** Method-specific data. */
  readonly data?: unknown;
}

/** Initial payment details. */
export interface PaymentDetails {
  /** Request id. */
  readonly id?: string;
  /** Total amount. */
  readonly total: PaymentLineItem;
  /** Line items. */
  readonly displayItems?: readonly PaymentLineItem[];
  /** Shipping options. */
  readonly shippingOptions?: readonly PaymentShippingOption[];
}

/** Details passed to `updateWith` after a shipping change. */
export interface PaymentDetailsUpdate {
  /** Updated total. */
  readonly total?: PaymentLineItem;
  /** Updated line items. */
  readonly displayItems?: readonly PaymentLineItem[];
  /** Updated shipping options (empty to reject the address). */
  readonly shippingOptions?: readonly PaymentShippingOption[];
  /** Error message shown to the user. */
  readonly error?: string;
  /** Per-field shipping address errors. */
  readonly shippingAddressErrors?: Readonly<Record<string, string>>;
}

/** Shipping category of the request. */
export type PaymentShippingType = "shipping" | "delivery" | "pickup";

/** Payer information requested from the sheet. */
export interface PaymentRequestOptionsInit {
  /** Request the payer name. */
  readonly requestPayerName?: boolean;
  /** Request the payer email. */
  readonly requestPayerEmail?: boolean;
  /** Request the payer phone. */
  readonly requestPayerPhone?: boolean;
  /** Request a shipping address. */
  readonly requestShipping?: boolean;
  /** Shipping category. */
  readonly shippingType?: PaymentShippingType;
}

/** Outcome passed to {@link PaymentResponseLike.complete}. */
export type PaymentCompletion = "success" | "fail" | "unknown";

/** Minimal `PaymentResponse`. */
export interface PaymentResponseLike {
  /** Request id. */
  readonly requestId: string;
  /** Method the payer chose. */
  readonly methodName: string;
  /** Method-specific response data. */
  readonly details: unknown;
  /** Payer name, when requested. */
  readonly payerName?: string | null;
  /** Payer email, when requested. */
  readonly payerEmail?: string | null;
  /** Payer phone, when requested. */
  readonly payerPhone?: string | null;
  /** Shipping address, when requested. */
  readonly shippingAddress?: unknown;
  /** Selected shipping option id. */
  readonly shippingOption?: string | null;
  /** Close the sheet with an outcome. */
  complete(result?: PaymentCompletion): Promise<void>;
}

/** Minimal `PaymentRequest` instance. */
export interface PaymentRequestLike extends EventTarget {
  /** Current shipping address. */
  readonly shippingAddress?: unknown;
  /** Current shipping option id. */
  readonly shippingOption?: string | null;
  /** Whether a method can be used. */
  canMakePayment(): Promise<boolean>;
  /** Show the payment sheet (needs user activation). */
  show(): Promise<PaymentResponseLike>;
  /** Abort a showing sheet. */
  abort(): Promise<void>;
}

/** Minimal `PaymentRequest` constructor. */
export type PaymentRequestHost = new (
  methods: PaymentMethod[],
  details: PaymentDetails,
  options?: PaymentRequestOptionsInit,
) => PaymentRequestLike;

/** Context passed to shipping change hooks. */
export interface PaymentShippingChange {
  /** Request that fired the change. */
  readonly request: PaymentRequestLike;
  /** Current shipping address. */
  readonly shippingAddress: unknown;
  /** Current shipping option id. */
  readonly shippingOption: string | null;
}

/**
 * Shipping change hook. Return (or resolve to) a details update to call
 * `updateWith`; return `undefined` to leave the sheet unchanged.
 */
export type PaymentShippingChangeHook = (
  change: PaymentShippingChange,
) => PaymentDetailsUpdate | Promise<PaymentDetailsUpdate> | undefined;

/** Options for {@link usePaymentRequest}. */
export interface UsePaymentRequestOptions {
  /** Accepted payment methods. Reactive; read when a request is created. */
  readonly methods: MaybeRefOrGetter<readonly PaymentMethod[]>;

  /** Payment details. Reactive; read when a request is created. */
  readonly details: MaybeRefOrGetter<PaymentDetails>;

  /**
   * Payer information to request.
   *
   * @default {}
   */
  readonly paymentOptions?: MaybeRefOrGetter<PaymentRequestOptionsInit>;

  /**
   * `PaymentRequest` constructor for alternate runtimes and tests. A ref
   * (not a getter) because the host is a constructor function.
   *
   * @default window.PaymentRequest when it exists
   */
  readonly PaymentRequest?: MaybeRef<PaymentRequestHost | null | undefined>;

  /**
   * Called on `shippingaddresschange`.
   *
   * @default undefined
   */
  readonly onShippingAddressChange?: PaymentShippingChangeHook;

  /**
   * Called on `shippingoptionchange`.
   *
   * @default undefined
   */
  readonly onShippingOptionChange?: PaymentShippingChangeHook;
}

/** Lifecycle of the payment sheet. */
export type PaymentRequestState = "idle" | "interactive" | "awaiting-complete";

/** Discriminated outcome of {@link PaymentRequestControls.show}. */
export type PaymentShowResult =
  | {
      /** The payer authorized the payment; call `complete` next. */
      readonly status: "completed";
      /** Payment response. */
      readonly response: PaymentResponseLike;
    }
  | {
      /**
       * `aborted`: dismissed or aborted; `unsupported`: no API or no
       * supported method; `failed`: anything else.
       */
      readonly status: "aborted" | "unsupported" | "failed";
      /** Error thrown by the host, when one was thrown. */
      readonly error: unknown;
    };

/** Reactive state and actions returned by {@link usePaymentRequest}. */
export interface PaymentRequestControls {
  /** Whether the Payment Request API is available. */
  readonly supported: ComputedRef<boolean>;
  /** Sheet lifecycle. */
  readonly state: Readonly<ShallowRef<PaymentRequestState>>;
  /** Most recent response, cleared when a new sheet is shown. */
  readonly response: Readonly<ShallowRef<PaymentResponseLike | undefined>>;
  /** Most recent failure (not dismissals). */
  readonly error: Readonly<ShallowRef<unknown>>;
  /**
   * Whether the configured methods can be used.
   *
   * @returns `false` when unsupported or the check fails.
   */
  readonly canMakePayment: () => Promise<boolean>;
  /**
   * Show the payment sheet. Aborts a sheet that is still showing.
   *
   * @returns The discriminated outcome; never rejects.
   */
  readonly show: () => Promise<PaymentShowResult>;
  /**
   * Close the sheet after processing the response.
   *
   * @param result Processing outcome.
   * @default result "unknown"
   * @returns Whether a response was completed.
   */
  readonly complete: (result?: PaymentCompletion) => Promise<boolean>;
  /**
   * Abort the showing sheet.
   *
   * @returns Whether a sheet was aborted.
   */
  readonly abort: () => Promise<boolean>;
}

function isPaymentRequestHost(candidate: unknown): candidate is PaymentRequestHost {
  return typeof candidate === "function";
}

function browserPaymentRequest(): PaymentRequestHost | undefined {
  if (typeof window === "undefined") return undefined;
  const candidate: unknown = Reflect.get(window, "PaymentRequest");
  return isPaymentRequestHost(candidate) ? candidate : undefined;
}

function hasUpdateWith(
  event: Event,
): event is Event & { updateWith(details: Promise<PaymentDetailsUpdate>): void } {
  return "updateWith" in event && typeof event.updateWith === "function";
}

function errorName(error: unknown): unknown {
  return typeof error === "object" && error !== null && "name" in error ? error.name : undefined;
}

/**
 * Collect payments with a typed Payment Request API wrapper.
 *
 * Every `show()` constructs a fresh request from the reactive `methods`,
 * `details`, and `paymentOptions`, wires the shipping hooks (which call
 * `updateWith` synchronously when they return an update), and resolves to a
 * discriminated {@link PaymentShowResult}. Call `complete()` after
 * processing a `"completed"` result. A showing sheet is aborted when the
 * owning reactive scope stops; outside a scope call `abort()`.
 *
 * Server rendering: nothing is constructed, `supported` is false and
 * `state` is `"idle"`.
 *
 * @example
 * ```ts
 * const payment = usePaymentRequest({ methods, details: { total } });
 * const result = await payment.show();
 * if (result.status === "completed") await payment.complete(await charge(result.response));
 * ```
 *
 * @param options Methods, details, hooks, and the constructor host.
 * @returns Payment state and actions.
 */
export function usePaymentRequest(options: UsePaymentRequestOptions): PaymentRequestControls {
  const state = shallowRef<PaymentRequestState>("idle");
  const response = shallowRef<PaymentResponseLike | undefined>(undefined);
  const error = shallowRef<unknown>(undefined);
  let active: PaymentRequestLike | undefined;
  let generation = 0;
  let completing = false;

  const resolveHost = (): PaymentRequestHost | undefined =>
    options.PaymentRequest === undefined
      ? browserPaymentRequest()
      : (unref(options.PaymentRequest) ?? undefined);

  const construct = (Host: PaymentRequestHost): PaymentRequestLike =>
    new Host(
      [...toValue(options.methods)],
      toValue(options.details),
      toValue(options.paymentOptions) ?? {},
    );

  const bindHook = (
    request: PaymentRequestLike,
    type: string,
    hook: PaymentShippingChangeHook | undefined,
  ): void => {
    if (!hook) return;
    request.addEventListener(type, (event) => {
      const update = hook({
        request,
        shippingAddress: request.shippingAddress ?? null,
        shippingOption: request.shippingOption ?? null,
      });
      if (update !== undefined && hasUpdateWith(event)) event.updateWith(Promise.resolve(update));
    });
  };

  const canMakePayment = async (): Promise<boolean> => {
    const Host = resolveHost();
    if (!Host) return false;
    try {
      return await construct(Host).canMakePayment();
    } catch {
      return false;
    }
  };

  const abort = async (): Promise<boolean> => {
    const current = active;
    if (!current) return false;
    try {
      await current.abort();
      if (active === current) {
        active = undefined;
        state.value = "idle";
        generation += 1;
      }
      return true;
    } catch {
      return false;
    }
  };

  const show = async (): Promise<PaymentShowResult> => {
    const Host = resolveHost();
    if (!Host) return { status: "unsupported", error: undefined };
    if (state.value === "awaiting-complete") {
      return {
        status: "failed",
        error: new Error("[VIZE_COMPOSE_PAYMENT_RESPONSE_INCOMPLETE] call complete() first"),
      };
    }
    const currentGeneration = ++generation;
    const previous = active;
    if (previous) {
      try {
        await previous.abort();
      } catch (cause) {
        if (active === previous) {
          error.value = cause;
          return { status: "failed", error: cause };
        }
      }
      if (active === previous) {
        active = undefined;
        state.value = "idle";
      }
      if (currentGeneration !== generation) return { status: "aborted", error: undefined };
    }
    response.value = undefined;
    let request: PaymentRequestLike;
    try {
      request = construct(Host);
    } catch (cause) {
      error.value = cause;
      return { status: "failed", error: cause };
    }
    bindHook(request, "shippingaddresschange", options.onShippingAddressChange);
    bindHook(request, "shippingoptionchange", options.onShippingOptionChange);
    active = request;
    state.value = "interactive";
    try {
      const result = await request.show();
      if (active !== request || currentGeneration !== generation) {
        return { status: "aborted", error: undefined };
      }
      active = undefined;
      response.value = result;
      state.value = "awaiting-complete";
      error.value = undefined;
      return { status: "completed", response: result };
    } catch (cause) {
      if (active !== request || currentGeneration !== generation) {
        return { status: "aborted", error: cause };
      }
      active = undefined;
      state.value = "idle";
      const name = errorName(cause);
      if (name === "AbortError") return { status: "aborted", error: cause };
      if (name === "NotSupportedError") return { status: "unsupported", error: cause };
      error.value = cause;
      return { status: "failed", error: cause };
    }
  };

  const complete = async (result: PaymentCompletion = "unknown"): Promise<boolean> => {
    const current = response.value;
    if (!current || state.value !== "awaiting-complete" || completing) return false;
    completing = true;
    try {
      await current.complete(result);
      return true;
    } catch (cause) {
      error.value = cause;
      return false;
    } finally {
      completing = false;
      state.value = "idle";
    }
  };

  tryOnScopeDispose(() => {
    void abort();
  });

  return {
    supported: computed(() => resolveHost() !== undefined),
    state: readonly(state),
    response: readonly(response),
    error: readonly(error),
    canMakePayment,
    show,
    complete,
    abort,
  };
}
