/** Compile-only assertions for the `use-web-locks` type contracts. */

import { useWebLocks } from "./use-web-locks.ts";
import type { LockManagerLike, WebLockAttempt } from "./use-web-locks.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const locks = useWebLocks();

const waited = locks.request("job", async () => 1);
type _WaitResolvesCallbackResult = Expect<Equal<Awaited<typeof waited>, number>>;

const attempt = locks.request("job", () => "done", { ifAvailable: true });
type _AttemptIsDiscriminated = Expect<Equal<Awaited<typeof attempt>, WebLockAttempt<string>>>;

declare const outcome: WebLockAttempt<string>;
if (outcome.acquired) {
  type _Value = Expect<Equal<typeof outcome.value, string>>;
} else {
  // @ts-expect-error a busy lock carries no value.
  void outcome.value;
}

declare const manager: LockManager;
manager satisfies LockManagerLike;

// @ts-expect-error only exclusive and shared modes exist.
void locks.request("job", () => 1, { mode: "read" });

// @ts-expect-error held names are read-only.
locks.held.value = [];
