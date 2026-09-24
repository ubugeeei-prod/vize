import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import {
  breakpointsBootstrapV5,
  breakpointsTailwind,
  toPixels,
  useBreakpoints,
} from "./breakpoints.ts";
import type { MediaQueryHost } from "./media-query.ts";

class ViewportHost implements MediaQueryHost {
  width: number;
  readonly lists = new Map<string, EventTarget & { matches: boolean }>();

  constructor(width: number) {
    this.width = width;
  }

  evaluate(query: string): boolean {
    const match = /\((min|max)-width: ([\d.]+)px\)/.exec(query);
    if (!match) return false;
    const pixels = Number(match[2]);
    return match[1] === "min" ? this.width >= pixels : this.width <= pixels;
  }

  matchMedia(query: string): MediaQueryList {
    const list = Object.assign(new EventTarget(), { matches: this.evaluate(query), media: query });
    this.lists.set(query, list);
    return list as unknown as MediaQueryList;
  }

  resize(width: number): void {
    this.width = width;
    for (const [query, list] of this.lists) {
      list.matches = this.evaluate(query);
      list.dispatchEvent(new Event("change"));
    }
  }
}

void test("mobile-first per-name refs, comparisons, current, and active", () => {
  const host = new ViewportHost(800);
  const scope = effectScope();
  const bp = scope.run(() => useBreakpoints(breakpointsTailwind, { host }));
  assert.ok(bp);
  assert.deepEqual([bp.sm.value, bp.md.value, bp.lg.value], [true, true, false]);
  assert.deepEqual(bp.current.value, ["sm", "md"]);
  assert.equal(bp.active.value, "md");
  assert.equal(bp.smaller("lg").value, true);
  assert.equal(bp.between("md", "lg").value, true);

  host.resize(768);
  assert.equal(bp.greater("md").value, false);
  assert.equal(bp.greaterOrEqual("md").value, true);
  assert.equal(bp.smallerOrEqual("md").value, true);
  host.resize(100);
  assert.equal(bp.active.value, null);
  scope.stop();
});

void test("evaluates queries arithmetically against ssrWidth on the server", () => {
  const bp = useBreakpoints(breakpointsBootstrapV5, { host: () => undefined, ssrWidth: 1000 });
  assert.deepEqual(bp.current.value, ["xs", "sm", "md", "lg"]);
  assert.equal(bp.smaller("xl").value, true);
  const none = useBreakpoints({ tablet: "48rem", desktop: "64em" }, { host: () => undefined });
  assert.deepEqual(none.current.value, []);
});

void test("converts rem/em/px lengths", () => {
  assert.equal(toPixels("48rem"), 768);
  assert.equal(toPixels("10em", 10), 100);
  assert.equal(toPixels("320px"), 320);
  assert.equal(toPixels(5), 5);
});
