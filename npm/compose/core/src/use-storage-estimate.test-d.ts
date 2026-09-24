/** Compile-only assertions for the `use-storage-estimate` type contracts. */

import { usePersistentStorage, useStorageEstimate } from "./use-storage-estimate.ts";
import type { StorageManagerLike } from "./use-storage-estimate.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const estimate = useStorageEstimate();
type _UsageIsNullable = Expect<Equal<typeof estimate.usage.value, number | null>>;
type _PercentIsNullable = Expect<Equal<typeof estimate.percentUsed.value, number | null>>;

const persistent = usePersistentStorage();
type _PersistResolvesBoolean = Expect<
  Equal<Awaited<ReturnType<typeof persistent.persist>>, boolean>
>;

declare const storage: StorageManager;
storage satisfies StorageManagerLike;

// @ts-expect-error usage is read-only.
estimate.usage.value = 1;

// @ts-expect-error the interval is a number of milliseconds.
useStorageEstimate({ interval: "1s" });
