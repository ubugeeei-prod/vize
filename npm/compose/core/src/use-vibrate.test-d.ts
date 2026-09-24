/** Compile-only assertions for the `use-vibrate` type contracts. */

import { useVibrate } from "./use-vibrate.ts";
import type { VibrationHost } from "./use-vibrate.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const vibration = useVibrate({ pattern: [100, 50] as const });
type _VibrateReturnsAcceptance = Expect<Equal<ReturnType<typeof vibration.vibrate>, boolean>>;

declare const navigatorHost: Navigator;
navigatorHost satisfies VibrationHost;

// @ts-expect-error patterns are numeric.
vibration.vibrate("long");

// @ts-expect-error the vibrating flag is read-only.
vibration.vibrating.value = true;
