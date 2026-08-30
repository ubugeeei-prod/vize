/** Compile-only assertions for the public deterministic ID contract. */

import { h } from "vue";
import type { ComputedRef } from "vue";

import {
  createDeterministicIdScope,
  type DeterministicId,
  type DeterministicIdSeed,
  IdProvider,
  useDeterministicId,
} from "./id.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const stringSeed: DeterministicIdSeed = "request-42";
export const numericSeed: DeterministicIdSeed = 42;

// @ts-expect-error boolean seeds are outside the closed public seed contract.
export const booleanSeed: DeterministicIdSeed = true;

export const typedScope = createDeterministicIdScope({ prefix: "checkout", seed: stringSeed });
export const typedId = typedScope.nextId("field");
export const idIsUsableAsAString: string = typedId;

// @ts-expect-error arbitrary strings must be validated before receiving the branded type.
export const unvalidatedId: DeterministicId = "field-1";

export const composedId = useDeterministicId({
  id: () => undefined,
  hint: "control",
  prefix: "field",
});

type _ComposableResultRemainsBranded = Expect<
  Equal<typeof composedId, ComputedRef<DeterministicId>>
>;

// @ts-expect-error hints remain strings rather than widening to arbitrary values.
void useDeterministicId({ hint: 1 });

// @ts-expect-error scope seeds reject objects at compile time.
void createDeterministicIdScope({ seed: { request: 1 } });

export const providerAcceptsDocumentedProps = h(IdProvider, {
  prefix: "checkout",
  seed: "request-42",
});

export function providerRejectsBooleanSeed() {
  // @ts-expect-error provider seeds preserve the string | number contract.
  return h(IdProvider, { seed: true });
}
