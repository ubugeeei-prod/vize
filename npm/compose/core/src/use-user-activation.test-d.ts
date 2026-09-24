/** Compile-only assertions for the `use-user-activation` type contracts. */

import { useUserActivation } from "./use-user-activation.ts";
import type { UserActivationLike } from "./use-user-activation.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const activation = useUserActivation();
type _IsActive = Expect<Equal<typeof activation.isActive.value, boolean>>;

declare const native: UserActivation;
native satisfies UserActivationLike;

// @ts-expect-error events are strings.
useUserActivation({ events: [1] });

// @ts-expect-error activation flags are read-only.
activation.isActive.value = true;
