import assert from "node:assert/strict";

import { h } from "vue";

import Cluster from "./cluster.vue";
import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";

export const clusterRuntimeFixture: RuntimeFixture = {
  name: "cluster",
  sourceFile: "cluster.vue",
  render: () =>
    h(
      Cluster,
      {
        align: "center",
        as: "nav",
        gap: 6,
        justify: "space-between",
      },
      {
        default: () => [h("a", { href: "/alpha" }, "Alpha"), h("a", { href: "/beta" }, "Beta")],
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<nav/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="cluster"/);
    assert.match(html, /data-state="clustered"/);
    assert.match(html, /data-wrap="true"/);
    assert.match(html, /data-reversed="false"/);
    assert.match(html, /data-align="center"/);
    assert.match(html, /data-justify="space-between"/);
    assert.match(html, /data-vize-cluster-direction="row"/);
    assert.match(html, /data-vize-cluster-gap="6px"/);
    assert.match(html, /--vize-ui-cluster-gap:6px/);
    assert.match(html, /flex-direction:row/);
    assert.match(html, /flex-wrap:wrap/);
    assert.match(html, /<a href="\/alpha">Alpha<\/a><a href="\/beta">Beta<\/a>/);
  },
  assertHydratedDom(host) {
    const cluster = host.querySelector('[data-vize-ui="cluster"]');
    assert.ok(cluster instanceof HTMLElement);
    assert.equal(cluster.getAttribute("role"), null);
    assert.equal(cluster.getAttribute("aria-hidden"), null);
    assert.equal(cluster.getAttribute("tabindex"), null);
    assert.equal(cluster.getAttribute("part"), "root");
    assert.equal(cluster.getAttribute("data-wrap"), "true");
    assert.equal(cluster.getAttribute("data-reversed"), "false");
    assert.equal(cluster.style.getPropertyValue("--vize-ui-cluster-gap"), "6px");
    assert.equal(cluster.style.display, "flex");
    assert.equal(cluster.style.flexDirection, "row");
    assert.equal(cluster.style.flexWrap, "wrap");
    assert.equal(cluster.textContent, "AlphaBeta");
  },
};
