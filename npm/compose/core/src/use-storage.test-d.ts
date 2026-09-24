/** Compile-only assertions for the `use-storage` type contracts. */

import type { Ref } from "vue";

import { storageSerializers, useLocalStorage, useStorage } from "./use-storage.ts";
import type { StorageFailure, StorageSerializer } from "./use-storage.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const count = useStorage("count", 0);
type _NumberDefaultInfersNumber = Expect<Equal<typeof count.state, Ref<number>>>;

const prefs = useLocalStorage("prefs", { dense: false, tags: [] as string[] });
type _ObjectDefaultInfersShape = Expect<
  Equal<typeof prefs.state.value, { dense: boolean; tags: string[] }>
>;

type Theme = "light" | "dark";
const theme = useStorage<Theme>("theme", "light", {
  validate: (candidate): candidate is Theme => candidate === "light" || candidate === "dark",
});
type _ExplicitUnionIsPreserved = Expect<Equal<typeof theme.state.value, Theme>>;
type _ErrorIsTyped = Expect<Equal<typeof theme.error.value, StorageFailure | undefined>>;

type _DateSerializer = Expect<Equal<typeof storageSerializers.date, StorageSerializer<Date>>>;

// @ts-expect-error the serializer must match the default's type.
useStorage("n", 0, { serializer: storageSerializers.string });

// @ts-expect-error assignments must keep the inferred type.
count.state.value = "1";

// @ts-expect-error the initial read timing is a closed union.
useStorage("n", 0, { initialRead: "lazy" });
