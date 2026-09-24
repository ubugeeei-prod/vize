/** Compile-only assertions for the composable resolver. */

import { VizeComposableResolver, vizeComposableImports } from "./resolver.ts";
import type { VizeComposableImport } from "./resolver.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const resolve = VizeComposableResolver({ exclude: ["use-toggle"] });

type _Resolve = Expect<Equal<ReturnType<typeof resolve>, VizeComposableImport | undefined>>;
type _Preset = Expect<
  Equal<ReturnType<typeof vizeComposableImports>, Readonly<Record<string, readonly string[]>>>
>;

// @ts-expect-error source modes are closed.
VizeComposableResolver({ source: "remote" });
