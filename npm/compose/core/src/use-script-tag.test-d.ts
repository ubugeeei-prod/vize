/** Compile-only assertions for the `use-script-tag` type contracts. */

import { useScriptTag } from "./use-script-tag.ts";
import type { ScriptTagElement, ScriptTagResult, ScriptTagStatus } from "./use-script-tag.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const script = useScriptTag("/widget.js", { crossOrigin: "anonymous" });
type _StatusIsClosed = Expect<Equal<typeof script.status.value, ScriptTagStatus>>;
type _LoadResolvesAResult = Expect<Equal<Awaited<ReturnType<typeof script.load>>, ScriptTagResult>>;

declare const element: HTMLScriptElement;
element satisfies ScriptTagElement;

declare const result: ScriptTagResult;
if (result.status === "loaded") result.element satisfies ScriptTagElement;

// @ts-expect-error CORS mode is a closed union.
useScriptTag("/a.js", { crossOrigin: "yes" });

// @ts-expect-error the status is read-only.
script.status.value = "loaded";
