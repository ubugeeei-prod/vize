import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useUrlHash } from "./use-url-hash.ts";
import type { UrlHashHost } from "./use-url-hash.ts";

class FakeWindow extends EventTarget implements UrlHashHost {
  readonly location = { hash: "", pathname: "/docs", search: "?q=1" };
  readonly calls: { readonly kind: "push" | "replace"; readonly url: string }[] = [];
  listeners = 0;
  readonly history = {
    state: { kept: true } as unknown,
    replaceState: (_data: unknown, _unused: string, url: string): void => {
      this.calls.push({ kind: "replace", url });
      this.setUrl(url);
    },
    pushState: (_data: unknown, _unused: string, url: string): void => {
      this.calls.push({ kind: "push", url });
      this.setUrl(url);
    },
  };

  override addEventListener(type: string, listener: EventListener): void {
    this.listeners += 1;
    super.addEventListener(type, listener);
  }

  override removeEventListener(type: string, listener: EventListener): void {
    this.listeners -= 1;
    super.removeEventListener(type, listener);
  }

  setUrl(url: string): void {
    const index = url.indexOf("#");
    this.location.hash = index === -1 ? "" : url.slice(index);
  }

  navigate(hash: string): void {
    this.location.hash = hash;
    this.dispatchEvent(new Event("hashchange"));
  }
}

void test("reads and follows the fragment as a string", () => {
  const host = new FakeWindow();
  host.location.hash = "#intro%20part";
  const hash = useUrlHash({ host });

  assert.equal(hash.supported.value, true);
  assert.equal(hash.state.value, "intro part");
  host.navigate("#next");
  assert.equal(hash.state.value, "next");
  host.navigate("");
  assert.equal(hash.state.value, "");
});

void test("writes through replaceState by default and removes empty fragments", () => {
  const host = new FakeWindow();
  const hash = useUrlHash({ host });

  hash.state.value = "a b#c";
  assert.deepEqual(host.calls.at(-1), { kind: "replace", url: "/docs?q=1#a%20b%23c" });
  hash.state.value = "";
  assert.deepEqual(host.calls.at(-1), { kind: "replace", url: "/docs?q=1" });
});

void test("pushes history entries in push mode and re-reads on popstate", () => {
  const host = new FakeWindow();
  const hash = useUrlHash({ host, mode: "push" });

  hash.state.value = "one";
  assert.deepEqual(host.calls, [{ kind: "push", url: "/docs?q=1#one" }]);
  host.location.hash = "";
  host.dispatchEvent(new Event("popstate"));
  assert.equal(hash.state.value, "");
});

void test("decodes typed values and falls back on parse failures", () => {
  type Tab = "profile" | "settings";
  const host = new FakeWindow();
  host.location.hash = "#settings";
  const tab = useUrlHash({
    host,
    parse: (raw): Tab => {
      if (raw === "profile" || raw === "settings") return raw;
      throw new Error(`unknown tab ${raw}`);
    },
    serialize: (value: Tab) => value,
    default: "profile",
  });

  assert.equal(tab.state.value, "settings");
  host.navigate("#bogus");
  assert.equal(tab.state.value, "profile");
  assert.ok(tab.error.value instanceof Error);
  host.navigate("#settings");
  assert.equal(tab.error.value, undefined);
});

void test("does not echo writes caused by navigation", () => {
  const host = new FakeWindow();
  useUrlHash({ host });
  host.navigate("#x");
  assert.equal(host.calls.length, 0);
});

void test("removes listeners when the scope stops", () => {
  const host = new FakeWindow();
  const scope = effectScope();
  const hash = scope.run(() => useUrlHash({ host }));
  assert.ok(hash);
  assert.equal(host.listeners, 2);
  scope.stop();
  assert.equal(host.listeners, 0);
  host.navigate("#late");
  assert.equal(hash.state.value, "");
});

void test("defers the first read for hydration", async () => {
  const host = new FakeWindow();
  host.location.hash = "#later";
  const hash = useUrlHash({ host, initialRead: "post-flush", ssrHash: "server" });
  assert.equal(hash.state.value, "server");
  await nextTick();
  assert.equal(hash.state.value, "later");
});

void test("server rendering uses ssrHash or the default", async () => {
  const state = await renderComposableOnServer(() => {
    const plain = useUrlHash();
    const known = useUrlHash({ ssrHash: "top" });
    const typed = useUrlHash({ parse: Number, serialize: String, default: 0, ssrHash: "7" });
    return {
      plain: plain.state,
      known: known.state,
      typed: typed.state,
      supported: plain.supported,
    };
  });
  assert.equal(state, '{"plain":"","known":"top","typed":7,"supported":false}');
});
