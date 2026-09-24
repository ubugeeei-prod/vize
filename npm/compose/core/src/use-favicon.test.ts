import assert from "node:assert/strict";
import { test } from "node:test";
import { nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useFavicon } from "./use-favicon.ts";
import type { FaviconHost, FaviconLink } from "./use-favicon.ts";

class FakeHead implements FaviconHost {
  readonly links: FaviconLink[] = [];

  findIcons(rel: string): Iterable<FaviconLink> {
    return this.links.filter((link) => link.rel.includes(rel));
  }

  createIcon(rel: string): FaviconLink {
    const link = { rel, href: "" };
    this.links.push(link);
    return link;
  }
}

void test("creates an icon link when none exists", () => {
  const host = new FakeHead();
  const { icon, supported } = useFavicon("/icon.svg", { host });
  assert.equal(supported.value, true);
  assert.deepEqual(host.links, [{ rel: "icon", href: "/icon.svg" }]);

  icon.value = "/other.svg";
  assert.deepEqual(host.links, [{ rel: "icon", href: "/other.svg" }]);
});

void test("updates every matching link with the base URL", () => {
  const host = new FakeHead();
  host.links.push({ rel: "icon", href: "a" }, { rel: "shortcut icon", href: "b" });
  host.links.push({ rel: "stylesheet", href: "c" });
  useFavicon("x.png", { host, baseUrl: "/static/" });

  assert.deepEqual(
    host.links.map((link) => link.href),
    ["/static/x.png", "/static/x.png", "c"],
  );
});

void test("follows getters and ignores nullish icons", async () => {
  const host = new FakeHead();
  const unread = ref(false);
  useFavicon(() => (unread.value ? "/unread.svg" : "/icon.svg"), { host });
  assert.equal(host.links[0]?.href, "/icon.svg");
  unread.value = true;
  await nextTick();
  assert.equal(host.links[0]?.href, "/unread.svg");

  const empty = new FakeHead();
  const { icon } = useFavicon(undefined, { host: empty });
  assert.equal(empty.links.length, 0);
  icon.value = null;
  assert.equal(empty.links.length, 0);
});

void test("uses a custom relation", () => {
  const host = new FakeHead();
  useFavicon("/touch.png", { host, rel: "apple-touch-icon" });
  assert.deepEqual(host.links, [{ rel: "apple-touch-icon", href: "/touch.png" }]);
});

void test("server rendering never touches the document", async () => {
  const state = await renderComposableOnServer(() => {
    const favicon = useFavicon("/icon.svg");
    return { icon: favicon.icon, supported: favicon.supported };
  });
  assert.equal(state, '{"icon":"/icon.svg","supported":false}');
});
