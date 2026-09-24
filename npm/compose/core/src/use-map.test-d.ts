/** Compile-only assertions for the `use-map` type contracts. */

import { useMap } from "./use-map.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const counts = useMap([["a", 1]]);
type _KeyInferred = Expect<Equal<Parameters<typeof counts.get>[0], string>>;
type _ValueInferred = Expect<Equal<ReturnType<typeof counts.get>, number | undefined>>;
type _ViewIsReadonly = Expect<Equal<typeof counts.map, ReadonlyMap<string, number>>>;
type _EntriesTyped = Expect<
  Equal<typeof counts.entries.value, readonly (readonly [string, number])[]>
>;

const literal = useMap([["a", 1]] as const);
type _ConstEntriesKeepLiterals = Expect<Equal<ReturnType<typeof literal.get>, 1 | undefined>>;

const explicit = useMap<number, { readonly label: string }>();
explicit.getOrInsert(1, (key) => ({ label: String(key) })) satisfies { readonly label: string };
counts.update("a", (previous) => (previous ?? 0) + 1) satisfies number;

// @ts-expect-error values keep the inferred type.
counts.set("b", "2");

// @ts-expect-error keys keep the inferred type.
counts.get(1);

// @ts-expect-error the reactive view is read-only.
counts.map.set("b", 2);
