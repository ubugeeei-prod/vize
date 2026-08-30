import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import { normalizeSurfaceIdReference, resolveSurfaceAria } from "./surface-runtime.ts";
import Surface from "./surface.vue";
import type { SurfaceExpose, SurfaceSemanticHost, SurfaceSlotState } from "./surface.ts";

const semanticHosts = [
  "section",
  "article",
  "aside",
  "div",
] as const satisfies readonly SurfaceSemanticHost[];

function formatSurfaceSlotState(state: SurfaceSlotState): string {
  const host = typeof state.as === "string" ? state.as : "component";
  return [
    host,
    state.ariaLabelledby ?? "",
    state.ariaDescribedby ?? "",
    state.tone ?? "",
    state.elevation ?? "",
    state.labelled ? "true" : "false",
    state.described ? "true" : "false",
  ].join(":");
}

test("renders a section surface by default without visual, focus, or ARIA side effects", async () => {
  const handle = mountInteraction(Surface, {
    slots: { default: "Release notes" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "SECTION");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "surface");
  assert.equal(root.getAttribute("data-tone"), null);
  assert.equal(root.getAttribute("data-elevation"), null);
  assert.equal(root.getAttribute("class"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.getAttribute("aria-labelledby"), null);
  assert.equal(root.getAttribute("aria-describedby"), null);
  assert.equal(root.textContent, "Release notes");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("renders every supported semantic host with optional hooks", () => {
  for (const as of semanticHosts) {
    const handle = mountInteraction(Surface, {
      props: {
        ariaDescribedby: "surface-help",
        ariaLabelledby: "surface-title",
        as,
        elevation: "raised",
        tone: "accent",
      },
      slots: { default: `${as} content` },
    });
    const root = handle.root();

    assert.equal(root.tagName, as.toUpperCase());
    assert.equal(root.getAttribute("aria-labelledby"), "surface-title");
    assert.equal(root.getAttribute("aria-describedby"), "surface-help");
    assert.equal(root.getAttribute("data-tone"), "accent");
    assert.equal(root.getAttribute("data-elevation"), "raised");
    assert.equal(root.getAttribute("role"), null);
    assert.equal(root.getAttribute("tabindex"), null);
    assert.equal(root.textContent, `${as} content`);
    handle.unmount();
  }
});

test("normalizes typed ARIA ID references and preserves ordinary fallthrough attrs", () => {
  assert.equal(normalizeSurfaceIdReference(undefined), undefined);
  assert.equal(normalizeSurfaceIdReference("   "), undefined);
  assert.equal(normalizeSurfaceIdReference(" title   subtitle "), "title subtitle");
  assert.deepEqual(
    resolveSurfaceAria({
      ariaDescribedby: " help   details ",
      ariaLabelledby: " title ",
    }),
    {
      ariaDescribedby: "help details",
      ariaLabelledby: "title",
    },
  );

  const handle = mountInteraction(Surface, {
    attrs: {
      "aria-label": "Manual region",
      "data-owner": "consumer",
      id: "manual-surface",
      role: "region",
      tabindex: "0",
    },
    props: {
      ariaDescribedby: " help   details ",
      ariaLabelledby: " title ",
      as: "section",
      elevation: "floating",
      tone: "muted",
    },
    slots: { default: "Manual content" },
  });
  const root = handle.root();

  assert.equal(root.id, "manual-surface");
  assert.equal(root.getAttribute("role"), "region");
  assert.equal(root.getAttribute("tabindex"), "0");
  assert.equal(root.getAttribute("aria-label"), "Manual region");
  assert.equal(root.getAttribute("aria-labelledby"), "title");
  assert.equal(root.getAttribute("aria-describedby"), "help details");
  assert.equal(root.getAttribute("data-owner"), "consumer");
  assert.equal(root.getAttribute("data-tone"), "muted");
  assert.equal(root.getAttribute("data-elevation"), "floating");
  handle.unmount();
});

test("renders a consumer component host without dropping surface hooks", () => {
  const PanelHost = defineComponent({
    name: "SurfacePanelHost",
    setup(_, { attrs, slots }) {
      return () => h("main", attrs, slots.default?.());
    },
  });
  const handle = mountInteraction(Surface, {
    attrs: {
      "data-owner": "consumer",
      id: "custom-surface",
    },
    props: {
      ariaDescribedby: "custom-help",
      ariaLabelledby: "custom-title",
      as: PanelHost,
      elevation: "overlay",
      tone: "success",
    },
    slots: { default: "Custom host" },
  });
  const root = handle.root();

  assert.equal(root.tagName, "MAIN");
  assert.equal(root.id, "custom-surface");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "surface");
  assert.equal(root.getAttribute("data-owner"), "consumer");
  assert.equal(root.getAttribute("aria-labelledby"), "custom-title");
  assert.equal(root.getAttribute("aria-describedby"), "custom-help");
  assert.equal(root.getAttribute("data-tone"), "success");
  assert.equal(root.getAttribute("data-elevation"), "overlay");
  assert.equal(root.textContent, "Custom host");
  handle.unmount();
});

test("passes slot state and exposes live surface state", async () => {
  const handle = mountInteraction(Surface, {
    props: {
      ariaDescribedby: "billing-help",
      ariaLabelledby: "billing-title",
      as: "article",
      elevation: "overlay",
      tone: "info",
    },
    slots: {
      default: formatSurfaceSlotState,
    },
  });
  const exposed = handle.exposes<SurfaceExpose>();
  const root = handle.root();

  assert.ok(exposed.element === root);
  assert.equal(exposed.as, "article");
  assert.equal(exposed.ariaLabelledby, "billing-title");
  assert.equal(exposed.ariaDescribedby, "billing-help");
  assert.equal(exposed.tone, "info");
  assert.equal(exposed.elevation, "overlay");
  assert.equal(exposed.labelled, true);
  assert.equal(exposed.described, true);
  assert.equal(root.textContent, "article:billing-title:billing-help:info:overlay:true:true");

  await handle.wrapper.setProps({
    ariaDescribedby: undefined,
    ariaLabelledby: "report-title",
    as: "aside",
    elevation: undefined,
    tone: "warning",
  });
  const updatedRoot = handle.root();

  assert.ok(exposed.element === updatedRoot);
  assert.equal(updatedRoot.tagName, "ASIDE");
  assert.equal(exposed.as, "aside");
  assert.equal(exposed.ariaLabelledby, "report-title");
  assert.equal(exposed.ariaDescribedby, undefined);
  assert.equal(exposed.tone, "warning");
  assert.equal(exposed.elevation, undefined);
  assert.equal(exposed.labelled, true);
  assert.equal(exposed.described, false);
  assert.equal(updatedRoot.getAttribute("aria-labelledby"), "report-title");
  assert.equal(updatedRoot.getAttribute("aria-describedby"), null);
  assert.equal(updatedRoot.getAttribute("data-tone"), "warning");
  assert.equal(updatedRoot.getAttribute("data-elevation"), null);
  assert.equal(updatedRoot.textContent, "aside:report-title::warning::true:false");
  handle.unmount();
});
