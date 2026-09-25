import assert from "node:assert/strict";

import { h } from "vue";

import { MediaPreferencesProvider } from "./media-preferences.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const mediaPreferencesRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "media-preferences-provider",
    sourceFile: "families/accessibility/media-preferences/media-preferences-provider.vue",
    render: () =>
      h(MediaPreferencesProvider, { force: { reducedMotion: true } }, () => h("p", "Content")),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="media-preferences-provider"/);
      assert.match(html, /data-reduced-motion="true"/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="media-preferences-provider"]');
      assert.ok(root instanceof HTMLElement);
      assert.equal(root.getAttribute("data-reduced-motion"), "true");
    },
  },
];
