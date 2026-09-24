/** Compile-only assertions for the `use-payment-request` type contracts. */

import { usePaymentRequest } from "./use-payment-request.ts";
import type { PaymentRequestState, PaymentResponseLike } from "./use-payment-request.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const total = { label: "Total", amount: { currency: "USD", value: "1.00" } };
const payment = usePaymentRequest({
  methods: [{ supportedMethods: "basic-card" }],
  details: { total },
});

type _State = Expect<Equal<typeof payment.state.value, PaymentRequestState>>;

async function narrow(): Promise<void> {
  const result = await payment.show();
  if (result.status === "completed") {
    type _Response = Expect<Equal<typeof result.response, PaymentResponseLike>>;
  } else {
    type _Status = Expect<Equal<typeof result.status, "aborted" | "unsupported" | "failed">>;
  }
}
void narrow;

// @ts-expect-error completion is a closed union.
void payment.complete("done");

// @ts-expect-error shippingType is a closed union.
usePaymentRequest({ methods: [], details: { total }, paymentOptions: { shippingType: "drone" } });

// @ts-expect-error details require a total.
usePaymentRequest({ methods: [], details: {} });

// @ts-expect-error state is read-only.
payment.state.value = "idle";
