/** Compile-only assertions for the `use-view-transition` type contracts. */

import { useViewTransition } from "./use-view-transition.ts";
import type { ViewTransitionDocumentHost, ViewTransitionLike } from "./use-view-transition.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const view = useViewTransition();

const sync = view.start(() => "done");
type _Sync = Expect<Equal<typeof sync, Promise<string>>>;

const async = view.start(async () => 1);
type _Async = Expect<Equal<typeof async, Promise<number>>>;

type _Transition = Expect<Equal<typeof view.transition.value, ViewTransitionLike | null>>;

document satisfies ViewTransitionDocumentHost;

// @ts-expect-error types are strings.
void view.start(() => undefined, { types: [1] });

// @ts-expect-error the transition is read-only.
view.transition.value = null;
