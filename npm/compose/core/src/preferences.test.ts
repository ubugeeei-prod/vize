import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import type { MediaQueryHost } from "./media-query.ts";
import {
  usePreferredColorScheme,
  usePreferredContrast,
  usePreferredDark,
  usePreferredReducedTransparency,
} from "./preferences.ts";

class FakeMediaQuery extends EventTarget {
  readonly media: string;
  matches: boolean;

  constructor(media: string, matches: boolean) {
    super();
    this.media = media;
    this.matches = matches;
  }
}

function mediaHost(initial: Record<string, boolean>): {
  readonly host: MediaQueryHost;
  readonly set: (query: string, matches: boolean) => void;
} {
  const lists = new Map<string, FakeMediaQuery>();
  const get = (query: string): FakeMediaQuery => {
    let list = lists.get(query);
    if (!list) {
      list = new FakeMediaQuery(query, initial[query] ?? false);
      lists.set(query, list);
    }
    return list;
  };
  return {
    host: { matchMedia: (query) => get(query) as unknown as MediaQueryList },
    set: (query, matches) => {
      const list = get(query);
      list.matches = matches;
      list.dispatchEvent(new Event("change"));
    },
  };
}

void test("server renders use the configured preferences", () => {
  const none = { host: () => undefined };
  assert.equal(usePreferredDark(none).value, false);
  assert.equal(usePreferredColorScheme(none).value, "no-preference");
  assert.equal(usePreferredColorScheme({ ...none, ssrPreference: "dark" }).value, "dark");
  assert.equal(usePreferredContrast({ ...none, ssrPreference: "more" }).value, "more");
  assert.equal(usePreferredReducedTransparency({ ...none, ssrValue: true }).value, true);
});

void test("color scheme follows media query changes", () => {
  const { host, set } = mediaHost({ "(prefers-color-scheme: light)": true });
  const scope = effectScope();
  const scheme = scope.run(() => usePreferredColorScheme({ host }));
  const dark = scope.run(() => usePreferredDark({ host }));
  assert.equal(scheme?.value, "light");
  set("(prefers-color-scheme: light)", false);
  set("(prefers-color-scheme: dark)", true);
  assert.deepEqual([scheme?.value, dark?.value], ["dark", true]);
  scope.stop();
});

void test("contrast maps every query to a closed preference", () => {
  const { host, set } = mediaHost({});
  const contrast = usePreferredContrast({ host });
  assert.equal(contrast.value, "no-preference");
  set("(prefers-contrast: custom)", true);
  assert.equal(contrast.value, "custom");
  set("(prefers-contrast: less)", true);
  assert.equal(contrast.value, "less");
  set("(prefers-contrast: more)", true);
  assert.equal(contrast.value, "more");
});
