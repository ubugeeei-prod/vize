import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h } from "vue";

import Icon from "./icon.vue";
import type { IconButtonExpose, IconButtonSlotState } from "./icon-button.ts";
import IconButton from "./icon-button.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("renders a native icon-only button with a required accessible name", async () => {
  const handle = mountInteraction(IconButton, {
    props: {
      ariaLabel: "Open command palette",
      size: "sm",
      tone: "accent",
      variant: "soft",
    },
    record: ["press"],
    slots: {
      default: (slotState: IconButtonSlotState) =>
        h(
          Icon,
          { ariaHidden: true, size: slotState.size },
          {
            default: () => h("path", { d: "M4 6h16M4 12h16M4 18h16" }),
          },
        ),
    },
  });
  const button = handle.getByRole("button", { name: "Open command palette" });
  const icon = button.querySelector('[data-vize-ui="icon"]');

  assert.equal(button.tagName, "BUTTON");
  assert.equal(button.getAttribute("type"), "button");
  assert.equal(button.getAttribute("data-vize-ui"), "icon-button");
  assert.equal(button.getAttribute("part"), "root");
  assert.equal(button.getAttribute("data-state"), "idle");
  assert.equal(button.getAttribute("data-size"), "sm");
  assert.equal(button.getAttribute("data-tone"), "accent");
  assert.equal(button.getAttribute("data-variant"), "soft");
  assert.equal(button.getAttribute("data-name"), "present");
  assert.equal(button.getAttribute("aria-label"), "Open command palette");
  assert.equal(button.getAttribute("aria-disabled"), null);
  assert.equal(button.getAttribute("aria-busy"), null);
  assert.equal(button.getAttribute("class"), null);
  assert.equal(button.getAttribute("style"), null);
  assert.ok(icon instanceof SVGElement);
  assert.equal(icon.getAttribute("aria-hidden"), "true");

  await handle.click(button);
  assert.equal(handle.recorded().length, 1);
  assert.ok(handle.recorded()[0]?.payload[0] instanceof MouseEvent);
  handle.unmount();
});

test("emulates native keyboard activation on non-native hosts", async () => {
  const label = document.createElement("span");
  label.id = "archive-icon-button-label";
  label.textContent = "Archive";
  document.body.append(label);
  const handle = mountInteraction(IconButton, {
    props: {
      ariaLabelledby: "archive-icon-button-label",
      as: "span",
      native: false,
      variant: "outline",
    },
    slots: { default: "A" },
  });
  const button = handle.getByRole("button", { name: "Archive" });

  assert.equal(button.tagName, "SPAN");
  assert.equal(button.getAttribute("role"), "button");
  assert.equal(button.getAttribute("tabindex"), "0");
  assert.equal(button.getAttribute("aria-labelledby"), "archive-icon-button-label");
  assert.equal(button.getAttribute("aria-label"), null);
  assert.equal(button.getAttribute("data-variant"), "outline");

  const enter = await handle.press(button, "Enter");
  assert.equal(enter.activated, false);
  assert.equal(handle.wrapper.emitted("press")?.length, 1);

  const space = await handle.press(button, " ");
  assert.equal(space.keydownPrevented, true);
  assert.equal(handle.wrapper.emitted("press")?.length, 2);
  handle.unmount();
  label.remove();
});

test("removes disabled native buttons from activation and tab order", async () => {
  const handle = mountInteraction(IconButton, {
    props: {
      ariaLabel: "Delete",
      disabled: true,
      tone: "danger",
    },
    slots: { default: "D" },
  });
  const button = handle.getByRole("button", { name: "Delete" });

  assert.ok(button instanceof HTMLButtonElement);
  assert.ok(button.hasAttribute("disabled"));
  assert.equal(button.getAttribute("aria-disabled"), null);
  assert.equal(button.getAttribute("aria-busy"), null);
  assert.equal(button.getAttribute("data-state"), "disabled");
  assert.equal(button.getAttribute("data-tone"), "danger");

  await handle.click(button);
  await handle.press(button, "Enter");
  await handle.press(button, " ");
  assert.equal(handle.wrapper.emitted("press"), undefined);
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("keeps loading buttons focusable while suppressing repeated activation", async () => {
  const handle = mountInteraction(IconButton, {
    props: {
      ariaDescribedby: "refresh-help",
      ariaLabel: "Refresh",
      loading: true,
    },
    slots: { default: "R" },
  });
  const button = handle.getByRole("button", { name: "Refresh" });

  assert.equal(button.getAttribute("aria-busy"), "true");
  assert.equal(button.getAttribute("aria-disabled"), "true");
  assert.equal(button.getAttribute("aria-describedby"), "refresh-help");
  assert.equal(button.getAttribute("data-state"), "loading");
  assert.equal(button.hasAttribute("disabled"), false);

  button.focus();
  assert.ok(handle.activeElement() === button);
  await handle.click(button);
  await handle.press(button, "Enter");
  await handle.press(button, " ");
  assert.equal(handle.wrapper.emitted("press"), undefined);
  handle.unmount();
});

test("passes slot state and exposes live icon-button state", async () => {
  const handle = mountInteraction(IconButton, {
    props: {
      ariaLabel: "Pin",
      size: "lg",
      tone: "neutral",
      variant: "solid",
    },
    slots: {
      default: (slotState: IconButtonSlotState) =>
        `${slotState.state}:${slotState.unavailable}:${slotState.size}:${slotState.tone}:${slotState.variant}`,
    },
  });
  const exposed = handle.exposes<IconButtonExpose>();
  const button = handle.root();

  assert.ok(exposed.element === button);
  assert.equal(exposed.disabled, false);
  assert.equal(exposed.loading, false);
  assert.equal(exposed.unavailable, false);
  assert.equal(exposed.state, "idle");
  assert.equal(exposed.size, "lg");
  assert.equal(exposed.tone, "neutral");
  assert.equal(exposed.variant, "solid");
  assert.equal(button.textContent, "idle:false:lg:neutral:solid");

  exposed.focus();
  assert.ok(handle.activeElement() === button);
  await handle.wrapper.setProps({ loading: true, size: "sm", tone: "accent", variant: "plain" });
  assert.equal(exposed.loading, true);
  assert.equal(exposed.unavailable, true);
  assert.equal(exposed.state, "loading");
  assert.equal(exposed.size, "sm");
  assert.equal(exposed.tone, "accent");
  assert.equal(exposed.variant, "plain");
  assert.equal(button.textContent, "loading:true:sm:accent:plain");
  handle.unmount();
});
