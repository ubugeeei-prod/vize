import assert from "node:assert/strict";

import { h } from "vue";

import LocaleProvider from "../families/i18n/locale/locale-provider.vue";
import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";

export const i18nRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "locale-provider",
    sourceFile: "families/i18n/locale/locale-provider.vue",
    render: () =>
      h(
        LocaleProvider,
        { locale: "ja-JP", direction: "ltr" },
        {
          default: () => "本文",
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="locale"/);
      assert.match(html, /lang="ja-JP"/);
      assert.match(html, /dir="ltr"/);
      assert.match(html, /本文/);
    },
    assertHydratedDom(host) {
      const locale = host.querySelector('[data-vize-ui="locale"]');
      assert.ok(locale instanceof HTMLElement);
      assert.equal(locale.getAttribute("lang"), "ja-JP");
      assert.equal(locale.textContent, "本文");
    },
  },
];
