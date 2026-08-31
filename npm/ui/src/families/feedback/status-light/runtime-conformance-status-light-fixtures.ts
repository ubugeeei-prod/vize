import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import StatusLight from "./status-light.vue";

export const statusLightRuntimeFixture: RuntimeFixture = {
  name: "status-light",
  sourceFile: "families/feedback/status-light/status-light.vue",
  render: () =>
    h(
      StatusLight,
      {
        ariaDescribedby: "service-status-help",
        ariaLabel: "Service online",
        size: "sm",
        state: "online",
        tone: "success",
      },
      {
        default: () => h("span", { id: "service-status-help" }, "API cluster"),
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<span/);
    assert.match(html, /role="img"/);
    assert.match(html, /aria-label="Service online"/);
    assert.match(html, /aria-describedby="service-status-help"/);
    assert.match(html, /data-vize-ui="status-light"/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-state="online"/);
    assert.match(html, /data-tone="success"/);
    assert.match(html, /data-size="sm"/);
    assert.match(html, /data-aria-state="img"/);
    assert.match(html, /API cluster/);
    assert.doesNotMatch(html, /class=|style=|tabindex=/);
  },
  assertHydratedDom(host) {
    const light = host.querySelector('[data-vize-ui="status-light"]');

    assert.ok(light instanceof HTMLElement);
    assert.equal(light.tagName, "SPAN");
    assert.equal(light.getAttribute("role"), "img");
    assert.equal(light.getAttribute("aria-label"), "Service online");
    assert.equal(light.getAttribute("aria-describedby"), "service-status-help");
    assert.equal(light.getAttribute("data-state"), "online");
    assert.equal(light.getAttribute("data-tone"), "success");
    assert.equal(light.getAttribute("data-size"), "sm");
    assert.equal(light.textContent, "API cluster");
  },
};
