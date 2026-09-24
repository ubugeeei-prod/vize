/** Compile-only assertions for the `use-eye-dropper` type contracts. */

import { useEyeDropper } from "./use-eye-dropper.ts";
import type { EyeDropperResult } from "./use-eye-dropper.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const dropper = useEyeDropper();
type _OpenResolvesAResult = Expect<
  Equal<Awaited<ReturnType<typeof dropper.open>>, EyeDropperResult>
>;

declare const result: EyeDropperResult;
if (result.status === "picked") result.sRGBHex satisfies string;

// @ts-expect-error hosts are constructors, not instances.
useEyeDropper({ host: { open: () => Promise.resolve({ sRGBHex: "" }) } });

// @ts-expect-error the picked color is read-only.
dropper.sRGBHex.value = "#fff";
