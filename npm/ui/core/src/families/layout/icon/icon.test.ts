import assert from "node:assert/strict";

import { mount } from "@vue/test-utils";
import { test } from "vite-plus/test";
import { h } from "vue";

import type { IconExpose, IconSlotState } from "./icon.ts";
import Icon from "./icon.vue";

interface IconMountOptions {
  readonly props?: Record<string, unknown>;
  readonly slots?: Record<string, unknown>;
}

function mountIcon(options: IconMountOptions = {}) {
  const container = document.createElement("div");
  document.body.append(container);
  const wrapper = mount(Icon, {
    props: options.props ?? {},
    slots: options.slots ?? {},
    attachTo: container,
  });

  return {
    wrapper,
    root() {
      assert.ok(
        wrapper.element instanceof SVGSVGElement,
        "Icon must render an SVG root by default",
      );
      return wrapper.element;
    },
    unmount() {
      wrapper.unmount();
      container.remove();
    },
  };
}

test("renders a decorative SVG icon by default without styling or semantics", () => {
  const handle = mountIcon({
    slots: { default: () => h("path", { d: "M4 12h16" }) },
  });
  const root = handle.root();

  assert.equal(root.getAttribute("data-vize-ui"), "icon");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("viewBox"), "0 0 24 24");
  assert.equal(root.getAttribute("width"), "1em");
  assert.equal(root.getAttribute("height"), "1em");
  assert.equal(root.getAttribute("focusable"), "false");
  assert.equal(root.getAttribute("fill"), "none");
  assert.equal(root.getAttribute("stroke"), "currentColor");
  assert.equal(root.getAttribute("stroke-width"), "2");
  assert.equal(root.getAttribute("stroke-linecap"), "round");
  assert.equal(root.getAttribute("stroke-linejoin"), "round");
  assert.equal(root.getAttribute("data-size"), "md");
  assert.equal(root.getAttribute("data-aria-state"), "decorative");
  assert.equal(root.getAttribute("data-decorative"), "true");
  assert.equal(root.getAttribute("data-title"), "missing");
  assert.equal(root.getAttribute("data-description"), "missing");
  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("aria-label"), null);
  assert.equal(root.getAttribute("aria-labelledby"), null);
  assert.equal(root.getAttribute("aria-describedby"), null);
  assert.equal(root.getAttribute("class"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.querySelector("title"), null);
  assert.equal(root.querySelector("desc"), null);
  assert.ok(root.querySelector("path"));
  handle.unmount();
});

test("renders accessible image semantics with deterministic title and description ids", () => {
  const handle = mountIcon({
    props: {
      description: "Refreshes every dashboard panel",
      descriptionId: "refresh-icon-desc",
      height: 16,
      size: "sm",
      strokeLinecap: "square",
      strokeLinejoin: "bevel",
      strokeWidth: "1.5",
      title: "Refresh panels",
      titleId: "refresh-icon-title",
      viewBox: "0 0 16 16",
      width: 16,
    },
    slots: {
      default: (slotState: IconSlotState) =>
        h("path", {
          "data-slot-state": `${slotState.ariaState}:${slotState.decorative}:${slotState.size}`,
          d: "M3 8h10",
        }),
    },
  });
  const root = handle.root();
  const title = root.querySelector("title");
  const desc = root.querySelector("desc");
  const path = root.querySelector("path");

  assert.equal(root.getAttribute("role"), "img");
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("aria-labelledby"), "refresh-icon-title");
  assert.equal(root.getAttribute("aria-describedby"), "refresh-icon-desc");
  assert.equal(root.getAttribute("aria-label"), null);
  assert.equal(root.getAttribute("viewBox"), "0 0 16 16");
  assert.equal(root.getAttribute("width"), "16");
  assert.equal(root.getAttribute("height"), "16");
  assert.equal(root.getAttribute("stroke-linecap"), "square");
  assert.equal(root.getAttribute("stroke-linejoin"), "bevel");
  assert.equal(root.getAttribute("stroke-width"), "1.5");
  assert.equal(root.getAttribute("data-size"), "sm");
  assert.equal(root.getAttribute("data-aria-state"), "img");
  assert.equal(root.getAttribute("data-decorative"), "false");
  assert.equal(root.getAttribute("data-title"), "present");
  assert.equal(root.getAttribute("data-description"), "present");
  assert.equal(title?.getAttribute("id"), "refresh-icon-title");
  assert.equal(title?.textContent, "Refresh panels");
  assert.equal(desc?.getAttribute("id"), "refresh-icon-desc");
  assert.equal(desc?.textContent, "Refreshes every dashboard panel");
  assert.equal(path?.getAttribute("data-slot-state"), "img:false:sm");
  handle.unmount();
});

test("prefers labelledby names over direct labels while preserving inline SVG text", () => {
  const label = document.createElement("span");
  label.id = "search-icon-label";
  label.textContent = "Search";
  document.body.append(label);
  const handle = mountIcon({
    props: {
      ariaLabel: "Ignored direct name",
      ariaLabelledby: "search-icon-label",
      title: "Search glyph",
      titleId: "search-icon-title",
    },
  });
  const root = handle.root();

  assert.equal(root.getAttribute("role"), "img");
  assert.equal(root.getAttribute("aria-labelledby"), "search-icon-label");
  assert.equal(root.getAttribute("aria-label"), null);
  assert.equal(root.querySelector("title")?.getAttribute("id"), "search-icon-title");
  assert.equal(root.querySelector("title")?.textContent, "Search glyph");
  handle.unmount();
  label.remove();
});

test("lets ariaHidden suppress invalid runtime labels and inline text", () => {
  const handle = mountIcon({
    props: {
      ariaHidden: true,
      ariaLabel: "Ignored",
      description: "Ignored description",
      descriptionId: "ignored-description",
      title: "Ignored title",
      titleId: "ignored-title",
    },
  });
  const root = handle.root();

  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("aria-label"), null);
  assert.equal(root.getAttribute("aria-labelledby"), null);
  assert.equal(root.getAttribute("aria-describedby"), null);
  assert.equal(root.getAttribute("data-aria-state"), "decorative");
  assert.equal(root.getAttribute("data-title"), "missing");
  assert.equal(root.getAttribute("data-description"), "missing");
  assert.equal(root.querySelector("title"), null);
  assert.equal(root.querySelector("desc"), null);
  handle.unmount();
});

test("passes slot state and exposes live icon state", async () => {
  const handle = mountIcon({
    props: {
      ariaLabel: "Close panel",
      description: "Dismisses the active panel",
      descriptionId: "close-icon-description",
      size: "lg",
    },
    slots: {
      default: (slotState: IconSlotState) =>
        h("path", {
          "data-slot-state": `${slotState.ariaState}:${slotState.decorative}:${slotState.size}:${slotState.titleId ?? "none"}:${slotState.descriptionId ?? "none"}:${slotState.viewBox}`,
          d: "M6 6l12 12M18 6L6 18",
        }),
    },
  });
  const root = handle.root();
  const exposed = handle.wrapper.vm as unknown as IconExpose;

  assert.ok(exposed.element === root);
  assert.equal(exposed.ariaState, "img");
  assert.equal(exposed.decorative, false);
  assert.equal(exposed.descriptionId, "close-icon-description");
  assert.equal(exposed.size, "lg");
  assert.equal(exposed.titleId, undefined);
  assert.equal(exposed.viewBox, "0 0 24 24");
  assert.equal(
    root.querySelector("path")?.getAttribute("data-slot-state"),
    "img:false:lg:none:close-icon-description:0 0 24 24",
  );

  await handle.wrapper.setProps({ ariaHidden: true, size: "xs", viewBox: "0 0 16 16" });
  assert.equal(exposed.ariaState, "decorative");
  assert.equal(exposed.decorative, true);
  assert.equal(exposed.descriptionId, undefined);
  assert.equal(exposed.size, "xs");
  assert.equal(exposed.viewBox, "0 0 16 16");
  assert.equal(root.getAttribute("viewBox"), "0 0 16 16");
  assert.equal(root.getAttribute("data-aria-state"), "decorative");
  assert.equal(
    root.querySelector("path")?.getAttribute("data-slot-state"),
    "decorative:true:xs:none:none:0 0 16 16",
  );
  handle.unmount();
});
