import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { useMediaQuery, useReducedMotion } from "./media-query.ts";
import type { MediaQueryHost } from "./media-query.ts";

class ObservableMediaQuery extends EventTarget {
  readonly media: string;
  matches: boolean;

  constructor(media: string, matches = false) {
    super();
    this.media = media;
    this.matches = matches;
  }

  setMatches(matches: boolean): void {
    this.matches = matches;
    this.dispatchEvent(new Event("change"));
  }

  asMediaQueryList(): MediaQueryList {
    return this as unknown as MediaQueryList;
  }
}

function createHost(initialMatches = false): {
  readonly host: MediaQueryHost;
  readonly results: Map<string, ObservableMediaQuery>;
} {
  const results = new Map<string, ObservableMediaQuery>();
  return {
    host: {
      matchMedia: (query) => {
        let result = results.get(query);
        if (!result) {
          result = new ObservableMediaQuery(query, initialMatches);
          results.set(query, result);
        }
        return result.asMediaQueryList();
      },
    },
    results,
  };
}

void test("uses an explicit server value without a media-query capability", () => {
  const matches = useMediaQuery("(width > 40rem)", {
    host: () => undefined,
    ssrValue: true,
  });

  assert.equal(matches.value, true);
});

void test("rebinds reactive queries and removes obsolete listeners", async () => {
  const { host, results } = createHost();
  const query = ref("(width > 40rem)");
  const scope = effectScope();
  const matches = scope.run(() => useMediaQuery(query, { host }));

  assert.ok(matches);
  const first = results.get("(width > 40rem)");
  assert.ok(first);
  first.setMatches(true);
  assert.equal(matches.value, true);

  query.value = "(orientation: portrait)";
  await nextTick();
  const second = results.get("(orientation: portrait)");
  assert.ok(second);
  assert.equal(matches.value, false);
  first.setMatches(false);
  assert.equal(matches.value, false);
  second.setMatches(true);
  assert.equal(matches.value, true);

  scope.stop();
  second.setMatches(false);
  assert.equal(matches.value, true);
});

void test("maps the motion query to a closed preference type", () => {
  const { host, results } = createHost(true);
  const preference = useReducedMotion({ host });
  const motion = results.get("(prefers-reduced-motion: reduce)");

  assert.ok(motion);
  assert.equal(preference.value, "reduce");
  motion.setMatches(false);
  assert.equal(preference.value, "no-preference");
});
