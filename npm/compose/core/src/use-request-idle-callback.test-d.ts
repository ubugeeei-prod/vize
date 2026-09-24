/** Compile-only assertions for the `use-request-idle-callback` type contracts. */

import { idle, useRequestIdleCallback } from "./use-request-idle-callback.ts";
import type { IdleCallbackHost, IdleDeadlineLike } from "./use-request-idle-callback.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const controls = useRequestIdleCallback((deadline) => {
  type _Deadline = Expect<Equal<typeof deadline, IdleDeadlineLike>>;
});
type _Idle = Expect<Equal<Awaited<ReturnType<typeof idle>>, IdleDeadlineLike>>;

window satisfies IdleCallbackHost;

// @ts-expect-error timeout is a number.
useRequestIdleCallback(() => undefined, { timeout: "1s" });

// @ts-expect-error the pending flag is read-only.
controls.isPending.value = true;
