/** Compile-only assertions for the `use-navigation-api` type contracts. */

import { useNavigationApi } from "./use-navigation-api.ts";
import type {
  NavigateEventLike,
  NavigationEntrySnapshot,
  NavigationHost,
  NavigationOutcome,
} from "./use-navigation-api.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface TabState {
  readonly tab: "home" | "settings";
}

const nav = useNavigationApi<TabState>();
type _State = Expect<Equal<typeof nav.state.value, TabState | undefined>>;
type _GetState = Expect<Equal<ReturnType<typeof nav.getState>, TabState | undefined>>;
type _Entry = Expect<Equal<typeof nav.currentEntry.value, NavigationEntrySnapshot | null>>;
type _Outcome = Expect<Equal<Awaited<ReturnType<typeof nav.back>>, NavigationOutcome>>;

const parsed = useNavigationApi({
  parseState: (raw) => (typeof raw === "number" ? raw : undefined),
});
type _Parsed = Expect<Equal<typeof parsed.state.value, number | undefined>>;

nav.onNavigate((event) => {
  type _Event = Expect<Equal<typeof event, NavigateEventLike>>;
});
void nav.navigate("/settings", { state: { tab: "settings" }, history: "replace" });

window.navigation satisfies NavigationHost;

// @ts-expect-error state must match State.
void nav.navigate("/x", { state: { tab: "profile" } });

// @ts-expect-error history modes are closed.
void nav.navigate("/x", { history: "prepend" });

// @ts-expect-error snapshots are read-only.
nav.currentEntry.value = null;
