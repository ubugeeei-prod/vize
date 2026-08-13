/** Compile-only assertions for the public typed context contract. */

import type { InjectionKey } from "vue";

import { createContext, type ComponentContext } from "./context.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface SelectionContext {
  readonly selectedKey: string | null;
}

export const selection = createContext<SelectionContext>("Selection");
export const provided = selection.provide({ selectedKey: "alpha" });

type _ContextPreservesGeneric = Expect<Equal<typeof selection, ComponentContext<SelectionContext>>>;
type _ProviderReturnsExactValue = Expect<Equal<typeof provided, SelectionContext>>;
type _KeyRemainsTyped = Expect<Equal<typeof selection.key, InjectionKey<SelectionContext>>>;
type _RequiredReadRemainsTyped = Expect<Equal<ReturnType<typeof selection.use>, SelectionContext>>;
type _OptionalReadRemainsTyped = Expect<
  Equal<ReturnType<typeof selection.useOptional>, SelectionContext | undefined>
>;

// @ts-expect-error provided values must satisfy the declared context type.
selection.provide({ selectedKey: 1 });

// @ts-expect-error the public injection key is readonly.
selection.key = Symbol("Other") as InjectionKey<SelectionContext>;

export const optionalUndefinedContext = createContext<string | undefined>("OptionalValue");
type _ExplicitUndefinedIsPreserved = Expect<
  Equal<ReturnType<typeof optionalUndefinedContext.use>, string | undefined>
>;
