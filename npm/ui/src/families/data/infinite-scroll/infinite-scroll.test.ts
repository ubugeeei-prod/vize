import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type {
  InfiniteScrollRootExpose,
  InfiniteScrollSentinelExpose,
  InfiniteScrollSlotState,
  InfiniteScrollTrigger,
} from "./infinite-scroll.ts";
import InfiniteScrollItem from "./infinite-scroll-item.vue";
import InfiniteScrollLoadMore from "./infinite-scroll-load-more.vue";
import InfiniteScrollRoot from "./infinite-scroll-root.vue";
import InfiniteScrollSentinel from "./infinite-scroll-sentinel.vue";
import InfiniteScrollStatus from "./infinite-scroll-status.vue";
import { mountInteraction } from "../../../testing/mount.ts";

type IntersectionCallback = (
  entries: readonly { target: Element; isIntersecting: boolean; intersectionRatio: number }[],
) => void;

class FakeIntersectionObserver {
  static instances: FakeIntersectionObserver[] = [];
  readonly observed: Element[] = [];
  observeCalls = 0;

  constructor(
    readonly callback: IntersectionCallback,
    readonly init: IntersectionObserverInit,
  ) {
    FakeIntersectionObserver.instances.push(this);
  }

  observe(target: Element): void {
    this.observeCalls += 1;
    this.observed.push(target);
  }

  unobserve(target: Element): void {
    const index = this.observed.indexOf(target);
    if (index >= 0) this.observed.splice(index, 1);
  }

  disconnect(): void {
    this.observed.length = 0;
  }

  intersect(isIntersecting: boolean): void {
    this.callback(
      this.observed.map((target) => ({
        target,
        isIntersecting,
        intersectionRatio: isIntersecting ? 1 : 0,
      })),
    );
  }
}

async function withFakeIntersectionObserver(run: () => Promise<void>): Promise<void> {
  const previous = globalThis.IntersectionObserver;
  FakeIntersectionObserver.instances = [];
  globalThis.IntersectionObserver =
    FakeIntersectionObserver as unknown as typeof IntersectionObserver;
  try {
    await run();
  } finally {
    globalThis.IntersectionObserver = previous;
  }
}

function observer(): FakeIntersectionObserver {
  const instance = FakeIntersectionObserver.instances[0];
  assert.ok(instance, "the sentinel must create an intersection observer");
  return instance;
}

interface Deferred {
  readonly promise: Promise<void>;
  readonly resolve: () => void;
  readonly reject: (reason: unknown) => void;
}

function deferred(): Deferred {
  let resolve: () => void = () => undefined;
  let reject: (reason: unknown) => void = () => undefined;
  const promise = new Promise<void>((onResolve, onReject) => {
    resolve = onResolve;
    reject = onReject;
  });
  return { promise, resolve, reject };
}

async function settle(): Promise<void> {
  await nextTick();
  await Promise.resolve();
  await nextTick();
}

function mountFeed(props: Record<string, unknown> = {}, count = 3) {
  return mountInteraction(InfiniteScrollRoot, {
    props: { id: "results", ...props },
    record: ["loadMore", "error", "stateChange"],
    slots: {
      default: (state: InfiniteScrollSlotState) => [
        ...Array.from({ length: count }, (_, index) =>
          h(InfiniteScrollItem, { index, ariaLabelledby: `title-${index}` }, () =>
            h("h3", { id: `title-${index}` }, `Result ${index + 1}`),
          ),
        ),
        h(InfiniteScrollSentinel, null, () => h("span", { "data-sentinel-state": state.state })),
        h(InfiniteScrollLoadMore, null, ({ state: current }: InfiniteScrollSlotState) =>
          current === "error" ? "Retry" : "Load more",
        ),
        h(InfiniteScrollStatus, null, ({ state: current }: InfiniteScrollSlotState) =>
          current === "loading"
            ? "Loading more results"
            : current === "complete"
              ? "All loaded"
              : "",
        ),
      ],
    },
  });
}

test("renders sentinel, load-more, status, and feed semantics with deterministic wiring", async () => {
  await withFakeIntersectionObserver(async () => {
    const handle = mountFeed({ feed: true, ariaLabel: "Search results", total: 30 });
    const root = handle.root();
    await nextTick();

    assert.equal(root.id, "results");
    assert.equal(root.getAttribute("role"), "feed");
    assert.equal(root.getAttribute("aria-label"), "Search results");
    assert.equal(root.getAttribute("aria-busy"), "false");
    assert.equal(root.getAttribute("data-vize-ui"), "infinite-scroll-root");
    assert.equal(root.getAttribute("data-state"), "idle");
    assert.equal(root.getAttribute("data-scroll-root"), "viewport");
    const items = [...root.querySelectorAll("article")];
    assert.equal(items.length, 3);
    assert.equal(items[0]?.getAttribute("aria-posinset"), "1");
    assert.equal(items[2]?.getAttribute("aria-posinset"), "3");
    assert.equal(items[0]?.getAttribute("aria-setsize"), "30");
    assert.equal(items[0]?.getAttribute("tabindex"), "0");
    assert.equal(items[0]?.getAttribute("aria-labelledby"), "title-0");
    const sentinel = root.querySelector('[data-vize-ui="infinite-scroll-sentinel"]');
    assert.equal(sentinel?.getAttribute("aria-hidden"), "true");
    assert.ok(observer().observed[0] === sentinel);
    assert.equal(observer().init.rootMargin, "256px");
    assert.equal(observer().init.root, null);
    const button = handle.getByRole("button", { name: "Load more" }) as HTMLButtonElement;
    assert.equal(button.type, "button");
    assert.equal(button.getAttribute("aria-controls"), "results");
    assert.equal(button.disabled, false);
    const status = handle.getByRole("status");
    assert.equal(status.getAttribute("aria-live"), "polite");
    assert.equal(status.getAttribute("aria-atomic"), "true");
    handle.unmount();
  });
});

test("the sentinel requests once per intersection and the loader promise drives loading", async () => {
  await withFakeIntersectionObserver(async () => {
    const pending = deferred();
    const triggers: InfiniteScrollTrigger[] = [];
    const handle = mountFeed({
      loader: (trigger: InfiniteScrollTrigger) => {
        triggers.push(trigger);
        return pending.promise;
      },
    });
    const root = handle.root();
    await nextTick();

    observer().intersect(true);
    observer().intersect(true);
    await nextTick();
    assert.deepEqual(triggers, ["sentinel"]);
    assert.equal(root.getAttribute("data-state"), "loading");
    assert.equal(root.getAttribute("aria-busy"), "true");
    assert.equal(handle.getByRole("status").textContent, "Loading more results");
    assert.equal((handle.getByRole("button") as HTMLButtonElement).disabled, true);

    const callsBefore = observer().observeCalls;
    pending.resolve();
    await settle();
    assert.equal(root.getAttribute("data-state"), "idle");
    assert.equal(observer().observeCalls, callsBefore + 1, "idle re-observes the sentinel");
    assert.deepEqual(handle.wrapper.emitted("loadMore"), [["sentinel"]]);
    assert.deepEqual(handle.wrapper.emitted("stateChange"), [
      ["loading", "idle"],
      ["idle", "loading"],
    ]);
    handle.unmount();
  });
});

test("controlled loading and hasMore settle state without a loader", async () => {
  await withFakeIntersectionObserver(async () => {
    const handle = mountFeed({ loading: false });
    const root = handle.root();
    await nextTick();
    observer().intersect(true);
    await nextTick();
    assert.deepEqual(handle.wrapper.emitted("loadMore"), [["sentinel"]]);
    assert.equal(root.getAttribute("data-state"), "idle", "the parent owns the busy flag");

    await handle.wrapper.setProps({ loading: true });
    assert.equal(root.getAttribute("data-state"), "loading");
    observer().intersect(true);
    assert.equal(handle.wrapper.emitted("loadMore")?.length, 1);

    await handle.wrapper.setProps({ loading: false, hasMore: false });
    assert.equal(root.getAttribute("data-state"), "complete");
    assert.equal(handle.getByRole("status").textContent, "All loaded");
    const button = root.querySelector("button");
    assert.equal(button?.hidden, true);
    assert.equal(button?.disabled, true);
    observer().intersect(true);
    assert.equal(handle.wrapper.emitted("loadMore")?.length, 1);
    handle.unmount();
  });
});

test("loader failures stop automatic loading until an explicit retry", async () => {
  await withFakeIntersectionObserver(async () => {
    let attempt = 0;
    const failure = new Error("offline");
    const handle = mountFeed({
      loader: () => {
        attempt += 1;
        return attempt === 1 ? Promise.reject(failure) : Promise.resolve();
      },
    });
    const root = handle.root();
    await nextTick();

    observer().intersect(true);
    await settle();
    assert.equal(root.getAttribute("data-state"), "error");
    assert.deepEqual(handle.wrapper.emitted("error"), [[failure, "sentinel"]]);
    assert.equal(handle.exposes<InfiniteScrollRootExpose>().error, failure);

    observer().intersect(true);
    assert.equal(attempt, 1, "the sentinel does not retry a failed page");
    const retry = handle.getByRole("button", { name: "Retry" });
    await handle.click(retry);
    await settle();
    assert.equal(attempt, 2);
    assert.equal(root.getAttribute("data-state"), "idle");
    assert.equal(handle.exposes<InfiniteScrollRootExpose>().error, undefined);

    const synchronous = mountFeed({
      loader: () => {
        throw failure;
      },
    });
    await nextTick();
    assert.equal(synchronous.exposes<InfiniteScrollRootExpose>().loadMore(), true);
    await nextTick();
    assert.equal(synchronous.root().getAttribute("data-state"), "error");
    assert.equal(synchronous.exposes<InfiniteScrollRootExpose>().retry(), true);
    synchronous.unmount();
    handle.unmount();
  });
});

test("the load-more button requests pages and honors preventDefault", async () => {
  const handle = mountInteraction(InfiniteScrollRoot, {
    record: ["loadMore"],
    slots: {
      default: () =>
        h(
          InfiniteScrollLoadMore,
          {
            onClick: (event: MouseEvent) => {
              if (event.shiftKey) event.preventDefault();
            },
          },
          () => "More",
        ),
    },
  });
  const button = handle.getByRole("button", { name: "More" });
  button.dispatchEvent(
    new MouseEvent("click", { bubbles: true, cancelable: true, shiftKey: true }),
  );
  await nextTick();
  assert.equal(handle.wrapper.emitted("loadMore"), undefined);
  await handle.click(button);
  assert.deepEqual(handle.wrapper.emitted("loadMore"), [["button"]]);
  handle.unmount();
});

test("disabled roots suppress every request", async () => {
  await withFakeIntersectionObserver(async () => {
    const handle = mountFeed({ disabled: true });
    await nextTick();
    observer().intersect(true);
    await handle.click(handle.getByRole("button"));
    assert.equal(handle.root().getAttribute("data-state"), "disabled");
    assert.equal(handle.exposes<InfiniteScrollRootExpose>().loadMore(), false);
    assert.equal(handle.wrapper.emitted("loadMore"), undefined);
    handle.unmount();
  });
});

test("self scroll roots observe the sentinel against the root element", async () => {
  await withFakeIntersectionObserver(async () => {
    const handle = mountFeed({ scrollRoot: "self", rootMargin: "0px 0px 64px 0px" });
    await nextTick();
    assert.ok(observer().init.root === handle.root());
    assert.equal(observer().init.rootMargin, "0px 0px 64px 0px");
    assert.equal(handle.root().getAttribute("data-scroll-root"), "self");
    const sentinel = handle.exposes<InfiniteScrollRootExpose>();
    const before = observer().observeCalls;
    sentinel.refresh();
    await nextTick();
    assert.equal(observer().observeCalls, before + 1);
    handle.unmount();
  });
});

test("the sentinel exposes its intersection state", async () => {
  await withFakeIntersectionObserver(async () => {
    const handle = mountInteraction(InfiniteScrollRoot, {
      props: { loading: true },
      slots: { default: () => h(InfiniteScrollSentinel, { ref: "sentinel" }) },
    });
    await nextTick();
    const sentinel = handle.wrapper.findComponent(InfiniteScrollSentinel);
    const exposed = sentinel.vm as unknown as InfiniteScrollSentinelExpose;
    assert.equal(exposed.intersecting, false);
    observer().intersect(true);
    assert.equal(exposed.intersecting, true);
    observer().callback([]);
    assert.equal(exposed.intersecting, true);
    handle.unmount();
  });
});

test("feed PageDown and PageUp move focus between articles and request at the end", async () => {
  await withFakeIntersectionObserver(async () => {
    const handle = mountFeed({ feed: true, loader: () => undefined });
    const items = [...handle.root().querySelectorAll<HTMLElement>("article")];
    items[0]?.focus();

    const down = new KeyboardEvent("keydown", { key: "PageDown", bubbles: true, cancelable: true });
    items[0]?.querySelector("h3")?.dispatchEvent(down);
    assert.equal(down.defaultPrevented, true);
    assert.ok(document.activeElement === items[1]);
    items[1]?.dispatchEvent(new KeyboardEvent("keydown", { key: "PageUp", bubbles: true }));
    assert.ok(document.activeElement === items[0]);
    items[2]?.focus();
    items[2]?.dispatchEvent(new KeyboardEvent("keydown", { key: "PageDown", bubbles: true }));
    assert.ok(document.activeElement === items[2]);
    assert.deepEqual(handle.wrapper.emitted("loadMore"), [["api"]]);

    const ignored = new KeyboardEvent("keydown", {
      key: "ArrowDown",
      bubbles: true,
      cancelable: true,
    });
    items[2]?.dispatchEvent(ignored);
    assert.equal(ignored.defaultPrevented, false);
    handle.unmount();

    const plain = mountFeed();
    const plainItems = [...plain.root().querySelectorAll<HTMLElement>("article")];
    assert.equal(plain.root().getAttribute("role"), null);
    assert.equal(plainItems[0]?.getAttribute("tabindex"), null);
    assert.equal(plainItems[0]?.getAttribute("aria-posinset"), null);
    const unhandled = new KeyboardEvent("keydown", {
      key: "PageDown",
      bubbles: true,
      cancelable: true,
    });
    plainItems[0]?.dispatchEvent(unhandled);
    assert.equal(unhandled.defaultPrevented, false);
    plain.unmount();
  });
});

test("exposes typed state and imperative paging controls", async () => {
  const handle = mountFeed({ loader: () => undefined, feed: true });
  const exposed = handle.exposes<InfiniteScrollRootExpose>();
  assert.equal(exposed.state, "idle");
  assert.equal(exposed.busy, false);
  assert.equal(exposed.hasMore, true);
  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.retry(), false, "retry requires an error");
  assert.equal(exposed.loadMore(), true);
  assert.deepEqual(handle.wrapper.emitted("loadMore"), [["api"]]);
  assert.equal(
    handle.root().querySelector("article")?.getAttribute("aria-setsize"),
    "-1",
    "unknown totals announce -1",
  );
  handle.unmount();
});

test("compound parts require a matching root provider", () => {
  for (const [part, props] of [
    [InfiniteScrollSentinel, {}],
    [InfiniteScrollLoadMore, {}],
    [InfiniteScrollStatus, {}],
    [InfiniteScrollItem, { index: 0 }],
  ] as const) {
    assert.throws(
      () => mountInteraction(part, { props }),
      /VIZE_UI_CONTEXT_MISSING: InfiniteScroll requires a matching provider/,
    );
  }
});
