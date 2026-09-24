import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useNavigationApi } from "./use-navigation-api.ts";
import type {
  NavigateEventLike,
  NavigationDestinationLike,
  NavigationHistoryEntryLike,
  NavigationHistoryMode,
  NavigationHost,
  NavigationInterceptInit,
  NavigationResultLike,
  NavigationTypeName,
} from "./use-navigation-api.ts";

class FakeEntry implements NavigationHistoryEntryLike {
  readonly id: string;
  readonly sameDocument = true;
  readonly url: string;
  readonly key: string;
  readonly index: number;
  readonly state: unknown;

  constructor(url: string, key: string, index: number, state: unknown) {
    this.url = url;
    this.key = key;
    this.index = index;
    this.state = state;
    this.id = `id-${key}`;
  }

  getState(): unknown {
    return this.state;
  }
}

class FakeNavigateEvent extends Event implements NavigateEventLike {
  readonly signal = new AbortController().signal;
  readonly userInitiated = false;
  readonly intercepted: NavigationInterceptInit[] = [];
  readonly navigationType: NavigationTypeName = "push";
  readonly info: unknown = undefined;
  readonly destination: NavigationDestinationLike;
  readonly canIntercept: boolean;
  readonly hashChange: boolean;
  readonly downloadRequest: string | null;

  constructor(
    destination: NavigationDestinationLike,
    canIntercept: boolean,
    hashChange: boolean,
    downloadRequest: string | null,
  ) {
    super("navigate", { cancelable: true });
    this.destination = destination;
    this.canIntercept = canIntercept;
    this.hashChange = hashChange;
    this.downloadRequest = downloadRequest;
  }

  intercept(options?: NavigationInterceptInit): void {
    this.intercepted.push(options ?? {});
  }
}

class FakeEntryChangeEvent extends Event {
  readonly navigationType: NavigationTypeName | null;
  readonly from: NavigationHistoryEntryLike;

  constructor(navigationType: NavigationTypeName | null, from: NavigationHistoryEntryLike) {
    super("currententrychange");
    this.navigationType = navigationType;
    this.from = from;
  }
}

class FakeNavigation extends EventTarget implements NavigationHost {
  list: FakeEntry[] = [new FakeEntry("https://app.test/", "a", 0, { page: 1 })];
  position = 0;
  transition: NavigationHost["transition"] = null;
  lastEvent: FakeNavigateEvent | undefined;
  failNext: unknown;
  keys = 0;

  get currentEntry(): FakeEntry | null {
    return this.list[this.position] ?? null;
  }

  get canGoBack(): boolean {
    return this.position > 0;
  }

  get canGoForward(): boolean {
    return this.position < this.list.length - 1;
  }

  entries(): readonly FakeEntry[] {
    return this.list;
  }

  dispatchNavigate(
    url: string,
    options: { canIntercept?: boolean; hashChange?: boolean; downloadRequest?: string | null } = {},
  ): FakeNavigateEvent {
    const event = new FakeNavigateEvent(
      { url, index: -1, sameDocument: false, getState: () => undefined },
      options.canIntercept ?? true,
      options.hashChange ?? false,
      options.downloadRequest ?? null,
    );
    this.lastEvent = event;
    this.dispatchEvent(event);
    return event;
  }

  commit(type: NavigationTypeName, update: () => void): NavigationResultLike {
    if (this.failNext !== undefined) {
      const failure = Promise.reject(this.failNext);
      this.failNext = undefined;
      this.dispatchEvent(Object.assign(new Event("navigateerror"), { error: "failed" }));
      return { committed: failure, finished: failure };
    }
    const from = this.currentEntry;
    update();
    const entry = this.currentEntry;
    assert.ok(from && entry);
    this.dispatchEvent(new FakeEntryChangeEvent(type, from));
    this.dispatchEvent(new Event("navigatesuccess"));
    return { committed: Promise.resolve(entry), finished: Promise.resolve(entry) };
  }

  navigate(
    url: string,
    options?: { state?: unknown; history?: NavigationHistoryMode; info?: unknown },
  ): NavigationResultLike {
    this.dispatchNavigate(url);
    return this.commit("push", () => {
      this.keys += 1;
      const replace = options?.history === "replace";
      const index = replace ? this.position : this.position + 1;
      this.list = [
        ...this.list.slice(0, index),
        new FakeEntry(url, `k${this.keys}`, index, options?.state),
      ];
      this.position = index;
    });
  }

  back(): NavigationResultLike {
    return this.commit("traverse", () => {
      this.position -= 1;
    });
  }

  forward(): NavigationResultLike {
    return this.commit("traverse", () => {
      this.position += 1;
    });
  }

  traverseTo(key: string): NavigationResultLike {
    return this.commit("traverse", () => {
      this.position = this.list.findIndex((entry) => entry.key === key);
    });
  }

  reload(options?: { state?: unknown }): NavigationResultLike {
    return this.commit("reload", () => {
      const entry = this.currentEntry;
      assert.ok(entry);
      this.list[this.position] = new FakeEntry(entry.url, entry.key, entry.index, options?.state);
    });
  }

  updateCurrentEntry(options: { state: unknown }): void {
    if (options.state === "invalid") throw new DOMException("bad", "DataCloneError");
    const entry = this.currentEntry;
    assert.ok(entry);
    this.list[this.position] = new FakeEntry(entry.url, entry.key, entry.index, options.state);
    this.dispatchEvent(new FakeEntryChangeEvent(null, entry));
  }
}

interface PageState {
  readonly page: number;
}

void test("snapshots the current entry, entries and state", () => {
  const navigation = new FakeNavigation();
  const nav = useNavigationApi<PageState>({ navigation });
  assert.equal(nav.supported.value, true);
  assert.deepEqual(nav.currentEntry.value, {
    url: "https://app.test/",
    key: "a",
    id: "id-a",
    index: 0,
    sameDocument: true,
  });
  assert.equal(nav.entries.value.length, 1);
  assert.equal(nav.canGoBack.value, false);
  assert.deepEqual(nav.state.value, { page: 1 });
  assert.deepEqual(nav.getState(), { page: 1 });
});

void test("navigates and traverses with outcomes and refreshed state", async () => {
  const navigation = new FakeNavigation();
  const nav = useNavigationApi<PageState>({ navigation });

  const outcome = await nav.navigate("https://app.test/two", { state: { page: 2 } });
  assert.equal(outcome.status, "success");
  assert.equal(outcome.status === "success" && outcome.entry.url, "https://app.test/two");
  assert.deepEqual(nav.state.value, { page: 2 });
  assert.equal(nav.canGoBack.value, true);

  await nav.back();
  assert.equal(nav.currentEntry.value?.key, "a");
  assert.equal(nav.canGoForward.value, true);
  await nav.forward();
  assert.equal(nav.currentEntry.value?.key, "k1");
  await nav.traverseTo("a");
  assert.equal(nav.currentEntry.value?.index, 0);
  await nav.reload({ state: { page: 9 } });
  assert.deepEqual(nav.state.value, { page: 9 });
});

void test("updateCurrentEntry replaces state and reports failures", () => {
  const navigation = new FakeNavigation();
  const nav = useNavigationApi({
    navigation,
    parseState: (raw) => (typeof raw === "string" ? raw : undefined),
  });
  const changes: (NavigationTypeName | null)[] = [];
  nav.onCurrentEntryChange((change) => changes.push(change.navigationType));

  assert.equal(nav.state.value, undefined, "parser rejects foreign state");
  assert.equal(nav.updateCurrentEntry({ state: "draft" }), true);
  assert.equal(nav.state.value, "draft");
  assert.deepEqual(changes, [null]);
  assert.equal(nav.updateCurrentEntry({ state: "invalid" }), false);
  assert.ok(nav.error.value instanceof DOMException);
});

void test("typed hooks receive navigation events", async () => {
  const navigation = new FakeNavigation();
  const nav = useNavigationApi({ navigation });
  const seen: string[] = [];
  const off = nav.onNavigate((event) => seen.push(`navigate ${event.destination.url}`));
  nav.onNavigateSuccess(() => seen.push("success"));
  nav.onNavigateError((error) => seen.push(`error ${String(error)}`));

  await nav.navigate("https://app.test/x");
  navigation.failNext = new Error("aborted");
  const failed = await nav.navigate("https://app.test/y");
  assert.ok(nav.error.value instanceof Error);
  off();
  await nav.navigate("https://app.test/z");

  assert.equal(failed.status, "error");
  assert.deepEqual(seen, [
    "navigate https://app.test/x",
    "success",
    "navigate https://app.test/y",
    "error failed",
    "success",
  ]);
  assert.equal(nav.error.value, undefined, "a later success clears the error");
});

void test("intercepts matching same-origin navigations only", async () => {
  const navigation = new FakeNavigation();
  const nav = useNavigationApi({ navigation });
  const handled: string[] = [];
  const stop = nav.intercept((url) => url.pathname.startsWith("/app/"), {
    handler: (event) => {
      handled.push(event.destination.url);
    },
    scroll: "manual",
  });

  const match = navigation.dispatchNavigate("https://app.test/app/1");
  assert.equal(match.intercepted.length, 1);
  assert.equal(match.intercepted[0]?.scroll, "manual");
  assert.equal(match.intercepted[0]?.focusReset, undefined);
  await match.intercepted[0]?.handler?.();
  assert.deepEqual(handled, ["https://app.test/app/1"]);

  for (const event of [
    navigation.dispatchNavigate("https://app.test/other"),
    navigation.dispatchNavigate("https://evil.test/app/1"),
    navigation.dispatchNavigate("https://app.test/app/2", { canIntercept: false }),
    navigation.dispatchNavigate("https://app.test/app/#x", { hashChange: true }),
    navigation.dispatchNavigate("https://app.test/app/f", { downloadRequest: "f.txt" }),
  ]) {
    assert.equal(event.intercepted.length, 0);
  }
  stop();
  assert.equal(navigation.dispatchNavigate("https://app.test/app/3").intercepted.length, 0);
});

void test("actions resolve unsupported without a host", async () => {
  const nav = useNavigationApi({ navigation: null });
  assert.equal(nav.supported.value, false);
  assert.deepEqual(await nav.navigate("/x"), { status: "unsupported" });
  assert.deepEqual(await nav.back(), { status: "unsupported" });
  assert.equal(nav.updateCurrentEntry({ state: 1 }), false);
  assert.equal(nav.getState(), undefined);
});

void test("removes listeners with the scope", () => {
  const navigation = new FakeNavigation();
  const scope = effectScope();
  const nav = scope.run(() => useNavigationApi({ navigation }));
  assert.ok(nav);
  let calls = 0;
  nav.intercept(() => true, {});
  nav.onNavigate(() => (calls += 1));
  scope.stop();
  const event = navigation.dispatchNavigate("https://app.test/after");
  assert.equal(calls, 0);
  assert.equal(event.intercepted.length, 0);
});

void test("server rendering reads nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const nav = useNavigationApi();
    return {
      supported: nav.supported,
      currentEntry: nav.currentEntry,
      entries: nav.entries,
      canGoBack: nav.canGoBack,
      transition: nav.transition,
    };
  });
  assert.equal(
    state,
    '{"supported":false,"currentEntry":null,"entries":[],"canGoBack":false,"transition":null}',
  );
});
