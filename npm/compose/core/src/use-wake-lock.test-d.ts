/** Compile-only assertions for the `use-wake-lock` type contracts. */

import { useWakeLock } from "./use-wake-lock.ts";
import type { WakeLockSentinelLike } from "./use-wake-lock.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const lock = useWakeLock();
type _RequestResolvesBoolean = Expect<Equal<Awaited<ReturnType<typeof lock.request>>, boolean>>;

declare const sentinel: WakeLockSentinel;
sentinel satisfies WakeLockSentinelLike;

// @ts-expect-error only screen locks exist.
void lock.request("system");

// @ts-expect-error the active flag is read-only.
lock.active.value = true;
