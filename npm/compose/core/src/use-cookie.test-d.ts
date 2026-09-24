/** Compile-only assertions for the `use-cookie` type contracts. */

import type { Ref } from "vue";

import { serializeCookie, useCookie } from "./use-cookie.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const raw = useCookie("session");
type _RawCookieMayBeAbsent = Expect<Equal<typeof raw.state, Ref<string | undefined>>>;

const count = useCookie("count", { default: 0 });
type _DefaultInfersNumber = Expect<Equal<typeof count.state, Ref<number>>>;

type Locale = "en" | "ja";
const locale = useCookie<Locale>("locale", {
  default: "en",
  validate: (candidate): candidate is Locale => candidate === "en" || candidate === "ja",
});
type _ExplicitUnion = Expect<Equal<typeof locale.state.value, Locale>>;

// @ts-expect-error SameSite is a closed union.
serializeCookie("a", "1", { sameSite: "Lax" });
// @ts-expect-error assignments keep the inferred type.
count.state.value = "1";
// @ts-expect-error the default must satisfy the explicit type.
useCookie<Locale>("locale", { default: "fr" });
