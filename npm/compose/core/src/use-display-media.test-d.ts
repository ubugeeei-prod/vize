/** Compile-only assertions for the `use-display-media` type contracts. */

import { useDisplayMedia } from "./use-display-media.ts";
import type { DisplayMediaHost } from "./use-display-media.ts";
import type { MediaStreamControls } from "./use-user-media.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const screen = useDisplayMedia({ video: { frameRate: 30 }, audio: true });
type _SharesTheStreamControls = Expect<Equal<typeof screen, MediaStreamControls>>;

declare const devices: MediaDevices;
devices satisfies DisplayMediaHost;

// @ts-expect-error constraints are booleans or track constraints.
useDisplayMedia({ video: "yes" });
