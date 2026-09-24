/** Compile-only assertions for the `use-url-hash` type contracts. */

import type { Ref } from "vue";

import { useUrlHash } from "./use-url-hash.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const plain = useUrlHash();
type _PlainIsString = Expect<Equal<typeof plain.state, Ref<string>>>;

const page = useUrlHash({ parse: Number, serialize: String, default: 1 });
type _CodecInfersNumber = Expect<Equal<typeof page.state, Ref<number>>>;

// @ts-expect-error a typed hash needs parse, serialize, and default together.
useUrlHash({ parse: Number });
// @ts-expect-error history modes are a closed union.
useUrlHash({ mode: "assign" });
// @ts-expect-error assignments keep the inferred type.
page.state.value = "2";
