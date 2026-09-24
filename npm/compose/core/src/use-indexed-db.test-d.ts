/** Compile-only assertions for the `use-indexed-db` type contracts. */

import type { Ref } from "vue";

import { createIndexedDBKeyval, useIndexedDB } from "./use-indexed-db.ts";
import type { IndexedDBFactoryLike } from "./use-indexed-db.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

// The browser implementation satisfies the structural host.
declare const browserFactory: IDBFactory;
browserFactory satisfies IndexedDBFactoryLike;

const draft = useIndexedDB("draft", { title: "", tags: [] as string[] });
type _DefaultInfersShape = Expect<
  Equal<typeof draft.state, Ref<{ title: string; tags: string[] }>>
>;
type _RefreshIsAsync = Expect<Equal<ReturnType<typeof draft.refresh>, Promise<void>>>;

const keyval = createIndexedDBKeyval();
type _GetIsUnknown = Expect<Equal<Awaited<ReturnType<typeof keyval.get>>, unknown>>;

// @ts-expect-error assignments keep the inferred type.
draft.state.value = { title: 1, tags: [] };
// @ts-expect-error keys must be valid IndexedDB keys.
useIndexedDB(true, 0);
// @ts-expect-error write timing is a closed union.
useIndexedDB("k", 0, { flush: "later" });
