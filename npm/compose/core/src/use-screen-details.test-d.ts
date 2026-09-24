/** Compile-only assertions for the `use-screen-details` type contracts. */

import { placeWindow, useScreenDetails } from "./use-screen-details.ts";
import type { ScreenDetailsHost, ScreenSnapshot, WindowPlacement } from "./use-screen-details.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const details = useScreenDetails();

type _Screens = Expect<Equal<typeof details.screens.value, readonly ScreenSnapshot[]>>;
type _Placement = Expect<Equal<ReturnType<typeof placeWindow>, WindowPlacement>>;

async function narrow(): Promise<void> {
  const result = await details.request();
  if (result.status === "granted") {
    type _Granted = Expect<Equal<typeof result.screens, readonly ScreenSnapshot[]>>;
  } else {
    type _Status = Expect<Equal<typeof result.status, "denied" | "unsupported" | "failed">>;
  }
}
void narrow;

declare const browserWindow: Window;
browserWindow satisfies ScreenDetailsHost;

// @ts-expect-error placement needs a screen area.
placeWindow({ width: 100 });

// @ts-expect-error screens are read-only.
details.screens.value = [];
