import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import Icon from "./icon.vue";
import IconButton from "./icon-button.vue";

export const iconRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "icon",
    sourceFile: "families/layout/icon/icon.vue",
    render: () =>
      h(
        Icon,
        {
          description: "Reloads every dashboard panel",
          descriptionId: "refresh-icon-description",
          size: "sm",
          title: "Refresh panels",
          titleId: "refresh-icon-title",
        },
        {
          default: () => h("path", { d: "M4 12h16" }),
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /^<svg/);
      assert.match(html, /role="img"/);
      assert.match(html, /aria-labelledby="refresh-icon-title"/);
      assert.match(html, /aria-describedby="refresh-icon-description"/);
      assert.match(html, /data-vize-ui="icon"/);
      assert.match(html, /part="root"/);
      assert.match(html, /data-size="sm"/);
      assert.match(html, /data-aria-state="img"/);
      assert.match(html, /data-decorative="false"/);
      assert.match(html, /<title id="refresh-icon-title"[^>]*>Refresh panels<\/title>/);
      assert.match(
        html,
        /<desc id="refresh-icon-description"[^>]*>Reloads every dashboard panel<\/desc>/,
      );
      assert.doesNotMatch(html, /class=|style=|tabindex=|aria-hidden=/);
    },
    assertHydratedDom(host) {
      const icon = host.querySelector('[data-vize-ui="icon"]');

      assert.ok(icon instanceof SVGSVGElement);
      assert.equal(icon.getAttribute("role"), "img");
      assert.equal(icon.getAttribute("aria-labelledby"), "refresh-icon-title");
      assert.equal(icon.getAttribute("aria-describedby"), "refresh-icon-description");
      assert.equal(icon.getAttribute("data-size"), "sm");
      assert.equal(icon.querySelector("title")?.textContent, "Refresh panels");
      assert.equal(icon.querySelector("desc")?.textContent, "Reloads every dashboard panel");
    },
  },
  {
    name: "icon-button",
    sourceFile: "families/layout/icon/icon-button.vue",
    render: () =>
      h(
        IconButton,
        {
          ariaLabel: "Refresh feed",
          size: "sm",
          tone: "accent",
          variant: "soft",
        },
        {
          default: () =>
            h(
              Icon,
              { ariaHidden: true, size: "sm" },
              {
                default: () => h("path", { d: "M4 12h16" }),
              },
            ),
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /^<button/);
      assert.match(html, /type="button"/);
      assert.match(html, /aria-label="Refresh feed"/);
      assert.match(html, /data-vize-ui="icon-button"/);
      assert.match(html, /data-state="idle"/);
      assert.match(html, /data-size="sm"/);
      assert.match(html, /data-tone="accent"/);
      assert.match(html, /data-variant="soft"/);
      assert.match(html, /data-name="present"/);
      assert.match(html, /data-vize-ui="icon"/);
      assert.match(html, /aria-hidden="true"/);
      assert.doesNotMatch(html, /class=|style=|tabindex=|aria-busy=/);
    },
    assertHydratedDom(host) {
      const button = host.querySelector('[data-vize-ui="icon-button"]');
      const icon = host.querySelector('[data-vize-ui="icon"]');

      assert.ok(button instanceof HTMLButtonElement);
      assert.equal(button.getAttribute("aria-label"), "Refresh feed");
      assert.equal(button.getAttribute("data-state"), "idle");
      assert.equal(button.getAttribute("data-size"), "sm");
      assert.equal(button.getAttribute("data-tone"), "accent");
      assert.equal(button.getAttribute("data-variant"), "soft");
      assert.ok(icon instanceof SVGSVGElement);
      assert.equal(icon.getAttribute("aria-hidden"), "true");
    },
  },
];
