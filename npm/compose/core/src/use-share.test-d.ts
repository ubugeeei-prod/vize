/** Compile-only assertions for the `use-share` type contracts. */

import { useShare } from "./use-share.ts";
import type { ShareHost, ShareResult } from "./use-share.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const sharing = useShare({ title: "Vize" });
type _ShareResolvesAResult = Expect<Equal<Awaited<ReturnType<typeof sharing.share>>, ShareResult>>;

declare const navigatorHost: Navigator;
navigatorHost satisfies ShareHost;

declare const result: ShareResult;
if (result.status === "shared") result.data.title satisfies string | undefined;

// @ts-expect-error share data fields are typed.
useShare({ title: 1 });

// @ts-expect-error the sharing flag is read-only.
sharing.sharing.value = true;
