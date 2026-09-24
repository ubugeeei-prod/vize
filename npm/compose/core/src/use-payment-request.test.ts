import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { usePaymentRequest } from "./use-payment-request.ts";
import type {
  PaymentCompletion,
  PaymentDetails,
  PaymentDetailsUpdate,
  PaymentMethod,
  PaymentRequestOptionsInit,
  PaymentResponseLike,
} from "./use-payment-request.ts";

const methods: readonly PaymentMethod[] = [{ supportedMethods: "https://pay.example" }];
const total = { label: "Total", amount: { currency: "JPY", value: "1000" } };

class FakeResponse implements PaymentResponseLike {
  requestId = "order-1";
  methodName = "https://pay.example";
  details = { token: "tok" };
  completed: PaymentCompletion[] = [];

  complete(result?: PaymentCompletion): Promise<void> {
    this.completed.push(result ?? "unknown");
    return Promise.resolve();
  }
}

class FakeUpdateEvent extends Event {
  updates: Promise<PaymentDetailsUpdate>[] = [];

  updateWith(details: Promise<PaymentDetailsUpdate>): void {
    this.updates.push(details);
  }
}

let outcome: "resolve" | "hang" | Error = "resolve";
let supportsMethod = true;
const instances: FakePaymentRequest[] = [];

class FakePaymentRequest extends EventTarget {
  shippingAddress: unknown = null;
  shippingOption: string | null = null;
  aborted = false;
  readonly details: PaymentDetails;
  readonly options: PaymentRequestOptionsInit | undefined;
  private reject: ((reason: unknown) => void) | undefined;

  constructor(
    methods: PaymentMethod[],
    details: PaymentDetails,
    options?: PaymentRequestOptionsInit,
  ) {
    super();
    this.details = details;
    this.options = options;
    if (methods.length === 0) throw new TypeError("no methods");
    instances.push(this);
  }

  canMakePayment(): Promise<boolean> {
    return Promise.resolve(supportsMethod);
  }

  show(): Promise<PaymentResponseLike> {
    if (outcome === "resolve") return Promise.resolve(new FakeResponse());
    if (outcome instanceof Error) return Promise.reject(outcome);
    return new Promise((_resolve, reject) => {
      this.reject = reject;
    });
  }

  abort(): Promise<void> {
    this.aborted = true;
    this.reject?.(new DOMException("aborted", "AbortError"));
    return Promise.resolve();
  }
}

function reset(): void {
  outcome = "resolve";
  supportsMethod = true;
  instances.length = 0;
}

void test("show resolves a completed result and complete closes it", async () => {
  reset();
  const details = ref<PaymentDetails>({ total });
  const payment = usePaymentRequest({
    methods,
    details,
    paymentOptions: { requestShipping: true },
    PaymentRequest: FakePaymentRequest,
  });
  assert.equal(payment.supported.value, true);
  assert.equal(payment.state.value, "idle");

  const result = await payment.show();
  assert.equal(result.status, "completed");
  assert.equal(payment.state.value, "awaiting-complete");
  assert.equal(instances[0]?.details, details.value);
  assert.deepEqual(instances[0]?.options, { requestShipping: true });

  assert.equal(await payment.complete("success"), true);
  assert.equal(payment.state.value, "idle");
  assert.ok(result.status === "completed" && result.response instanceof FakeResponse);
  assert.deepEqual(result.response.completed, ["success"]);
  assert.equal(await payment.complete(), false, "a response completes once");
});

void test("abort cancels the showing sheet", async () => {
  reset();
  outcome = "hang";
  const payment = usePaymentRequest({
    methods,
    details: { total },
    PaymentRequest: FakePaymentRequest,
  });
  const pending = payment.show();
  await Promise.resolve();
  assert.equal(payment.state.value, "interactive");
  assert.equal(await payment.abort(), true);
  const result = await pending;
  assert.equal(result.status, "aborted");
  assert.equal(payment.state.value, "idle");
  assert.equal(payment.error.value, undefined);
  assert.equal(await payment.abort(), false);
});

void test("showing again aborts the previous sheet", async () => {
  reset();
  outcome = "hang";
  const payment = usePaymentRequest({
    methods,
    details: { total },
    PaymentRequest: FakePaymentRequest,
  });
  const first = payment.show();
  await Promise.resolve();
  outcome = "resolve";
  const second = await payment.show();
  assert.equal((await first).status, "aborted");
  assert.equal(second.status, "completed");
  assert.equal(instances[0]?.aborted, true);
  assert.equal(payment.state.value, "awaiting-complete");
});

void test("shipping hooks call updateWith synchronously", async () => {
  reset();
  outcome = "hang";
  const changes: unknown[] = [];
  const payment = usePaymentRequest({
    methods,
    details: { total },
    PaymentRequest: FakePaymentRequest,
    onShippingAddressChange: ({ shippingAddress }) => {
      changes.push(shippingAddress);
      return { total: { ...total, amount: { currency: "JPY", value: "1200" } } };
    },
    onShippingOptionChange: ({ shippingOption }) => {
      changes.push(shippingOption);
      return undefined;
    },
  });
  void payment.show();
  await Promise.resolve();
  const request = instances[0];
  assert.ok(request);
  request.shippingAddress = { country: "JP" };
  const addressEvent = new FakeUpdateEvent("shippingaddresschange");
  request.dispatchEvent(addressEvent);
  assert.equal(addressEvent.updates.length, 1);
  assert.equal((await addressEvent.updates[0])?.total?.amount.value, "1200");

  request.shippingOption = "express";
  const optionEvent = new FakeUpdateEvent("shippingoptionchange");
  request.dispatchEvent(optionEvent);
  assert.equal(optionEvent.updates.length, 0);
  assert.deepEqual(changes, [{ country: "JP" }, "express"]);
  await payment.abort();
});

void test("classifies failures", async () => {
  reset();
  const failure = new Error("boom");
  outcome = failure;
  const payment = usePaymentRequest({
    methods,
    details: { total },
    PaymentRequest: FakePaymentRequest,
  });
  assert.deepEqual(await payment.show(), { status: "failed", error: failure });
  assert.equal(payment.error.value, failure);

  const unsupported = new DOMException("no", "NotSupportedError");
  outcome = unsupported;
  assert.deepEqual(await payment.show(), { status: "unsupported", error: unsupported });

  const invalid = usePaymentRequest({
    methods: [],
    details: { total },
    PaymentRequest: FakePaymentRequest,
  });
  const result = await invalid.show();
  assert.equal(result.status, "failed");
  assert.ok(invalid.error.value instanceof TypeError);
  assert.equal(invalid.state.value, "idle");
});

void test("canMakePayment reflects the host and never throws", async () => {
  reset();
  const payment = usePaymentRequest({
    methods,
    details: { total },
    PaymentRequest: FakePaymentRequest,
  });
  assert.equal(await payment.canMakePayment(), true);
  supportsMethod = false;
  assert.equal(await payment.canMakePayment(), false);
  const broken = usePaymentRequest({
    methods: [],
    details: { total },
    PaymentRequest: FakePaymentRequest,
  });
  assert.equal(await broken.canMakePayment(), false);
});

void test("reports unsupported without a constructor", async () => {
  const payment = usePaymentRequest({ methods, details: { total }, PaymentRequest: null });
  assert.equal(payment.supported.value, false);
  assert.equal(await payment.canMakePayment(), false);
  assert.deepEqual(await payment.show(), { status: "unsupported", error: undefined });
});

void test("scope disposal aborts the showing sheet", async () => {
  reset();
  outcome = "hang";
  const scope = effectScope();
  const payment = scope.run(() =>
    usePaymentRequest({ methods, details: { total }, PaymentRequest: FakePaymentRequest }),
  );
  assert.ok(payment);
  const pending = payment.show();
  await Promise.resolve();
  scope.stop();
  assert.equal((await pending).status, "aborted");
  assert.equal(instances[0]?.aborted, true);
});

void test("server rendering constructs nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const payment = usePaymentRequest({ methods, details: { total } });
    return { supported: payment.supported, state: payment.state };
  });
  assert.equal(state, '{"supported":false,"state":"idle"}');
});
