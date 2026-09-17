type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;
type Assert<T extends true> = T;
type Match<T, P> = __VizePatterns.Match<T, P>;
type Subtract<T, P> = __VizePatterns.Subtract<T, P>;
type Done<T> = __VizePatterns.Exhaustiveness<T>;
type AnyPattern = ["any"];
type Lit<V> = ["literal", V];
type Obj<K extends PropertyKey, P> = ["object", [[K, P]]];
type Tuple<A, Rest = false> = ["array", A, Rest];

type Result<T> = { kind: "ok"; data: T } | { kind: "err"; error: string };
type Successful = Assert<Equal<Match<Result<number>, Obj<"kind", Lit<"ok">>>, { kind: "ok"; data: number }>>;
type Remaining = Assert<Equal<Subtract<Result<number>, Obj<"kind", Lit<"ok">>>, { kind: "err"; error: string }>>;
type Complete = Assert<Equal<Done<Subtract<"ok" | "err", ["or", [Lit<"ok">, Lit<"err">]]>>, true>>;
type ValueUnion = Assert<Equal<Subtract<"ok" | "err", ["value", "ok" | "err"]>, "ok" | "err">>;
type ValueSingleton = Assert<Equal<Subtract<"ok" | "err", ["value", "ok"]>, "err">>;
type OpenString = Assert<Equal<Subtract<string, Lit<"ok">>, string>>;
type Unknown = Assert<Equal<Subtract<unknown, Obj<"x", AnyPattern>>, unknown>>;
type Any = Assert<Equal<Subtract<any, Lit<true>>, any>>;
type CatchAll = Assert<Equal<Done<Subtract<unknown, AnyPattern>>, true>>;
type OptionalValue = Assert<Equal<Match<{ x?: string }, Obj<"x", AnyPattern>>["x"], string | undefined>>;
type OptionalMissing = Assert<Equal<{} extends Subtract<{ x?: string }, Obj<"x", AnyPattern>> ? true : false, true>>;
type OptionalUndefined = Assert<Equal<Match<{ x?: string }, Obj<"x", Lit<undefined>>>["x"], undefined>>;
type ReadonlyTuple = Assert<Equal<Match<readonly ["a" | "b", number], Tuple<[Lit<"a">], true>>, readonly ["a", number]>>;
type RestArray = Assert<Equal<Match<number[], Tuple<[AnyPattern], true>>, [number, ...number[]]>>;
type TooShort = Assert<Equal<Match<[], Tuple<[AnyPattern], true>>, never>>;
type Matrix = ["a" | "b", 1 | 2];
type A1 = Subtract<Matrix, Tuple<[Lit<"a">, Lit<1>]>>;
type A2 = Subtract<A1, Tuple<[Lit<"a">, Lit<2>]>>;
type B1 = Subtract<A2, Tuple<[Lit<"b">, Lit<1>]>>;
type B2 = Subtract<B1, Tuple<[Lit<"b">, Lit<2>]>>;
type MatrixExhaustive = Assert<Equal<B2, never>>;
type ReadonlyObject = Assert<Equal<Match<{ readonly x: "a" }, Obj<"x", Lit<"a">>>, { readonly x: "a" }>>;
type ReadonlyRefined = Assert<Equal<Match<{ readonly x: "a" | "b" }, Obj<"x", Lit<"a">>>, { readonly x: "a" }>>;
type ReadonlyOptional = Assert<Equal<Match<{ readonly x?: string }, Obj<"x", AnyPattern>>, { readonly x: string | undefined }>>;
type ReadonlyRemaining = Assert<Equal<Subtract<readonly ["a" | "b", number], Tuple<[Lit<"a">], true>>, readonly ["b", number]>>;
type Nested = Subtract<{ box: { value: 1 | 2 } }, Obj<"box", Obj<"value", Lit<1>>>>;
type NestedRest = Assert<Equal<Nested, { box: { value: 2 } }>>;

const complete: Done<never> = true;
// @ts-expect-error A finite missing arm must not silently pass.
const missing: Done<"err"> = true;
// @ts-expect-error Matching a present optional key does not cover absence.
const optional: Done<Subtract<{ x?: string }, Obj<"x", AnyPattern>>> = true;
// @ts-expect-error A value union is not a runtime catch-all.
const value: Done<Subtract<"ok" | "err", ["value", "ok" | "err"]>> = true;
// @ts-expect-error Open domains require an unguarded catch-all.
const open: Done<Subtract<string, Lit<"ok">>> = true;
declare let readonly: Match<{ readonly x?: "a" | "b" }, Obj<"x", Lit<"a">>>;
// @ts-expect-error Refining a present optional property must preserve readonly.
readonly.x = "a";
export {};
