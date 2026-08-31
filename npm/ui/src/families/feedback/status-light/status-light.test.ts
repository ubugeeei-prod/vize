import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import StatusLight from "./status-light.vue";
import type { StatusLightExpose, StatusLightSlotState } from "./status-light.ts";
import { mountInteraction } from "../../../testing/mount.ts";

test("renders a decorative neutral unknown light by default", async () => {
  const handle = mountInteraction(StatusLight);
  const root = handle.root();

  assert.equal(root.tagName, "SPAN");
  assert.equal(root.getAttribute("data-vize-ui"), "status-light");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-state"), "unknown");
  assert.equal(root.getAttribute("data-tone"), "neutral");
  assert.equal(root.getAttribute("data-size"), "md");
  assert.equal(root.getAttribute("data-aria-state"), "decorative");
  assert.equal(root.getAttribute("data-decorative"), "true");
  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("renders labelled image semantics with description support", () => {
  const help = document.createElement("p");
  help.id = "cluster-status-help";
  help.textContent = "API cluster";
  document.body.append(help);
  const handle = mountInteraction(StatusLight, {
    props: {
      ariaDescribedby: "cluster-status-help",
      ariaLabel: "Cluster online",
      size: "sm",
      state: "online",
      tone: "success",
    },
    slots: {
      default: (slotState: StatusLightSlotState) => `${slotState.state}:${slotState.tone}`,
    },
  });
  const light = handle.getByRole("img", { name: "Cluster online" });

  assert.equal(light.getAttribute("aria-describedby"), "cluster-status-help");
  assert.equal(light.getAttribute("aria-hidden"), null);
  assert.equal(light.getAttribute("aria-live"), null);
  assert.equal(light.getAttribute("data-state"), "online");
  assert.equal(light.getAttribute("data-tone"), "success");
  assert.equal(light.getAttribute("data-size"), "sm");
  assert.equal(light.getAttribute("data-aria-state"), "img");
  assert.equal(light.getAttribute("data-decorative"), "false");
  assert.equal(light.textContent, "online:success");
  handle.unmount();
  help.remove();
});

test("supports status announcements and labelledby names", () => {
  const label = document.createElement("span");
  label.id = "deploy-status-label";
  label.textContent = "Deploy status";
  document.body.append(label);
  const handle = mountInteraction(StatusLight, {
    props: {
      ariaLabelledby: "deploy-status-label",
      atomic: false,
      role: "status",
      size: "lg",
      state: "busy",
      tone: "warning",
    },
    slots: { default: "Deploying" },
  });
  const status = handle.getByRole("status", { name: "Deploy status" });

  assert.equal(status.getAttribute("aria-labelledby"), "deploy-status-label");
  assert.equal(status.getAttribute("aria-live"), "polite");
  assert.equal(status.getAttribute("aria-atomic"), "false");
  assert.equal(status.getAttribute("data-state"), "busy");
  assert.equal(status.getAttribute("data-tone"), "warning");
  assert.equal(status.getAttribute("data-size"), "lg");
  assert.equal(status.textContent, "Deploying");
  handle.unmount();
  label.remove();
});

test("lets ariaHidden override labelled status semantics", () => {
  const handle = mountInteraction(StatusLight, {
    props: {
      ariaDescribedby: "ignored-help",
      ariaHidden: true,
      ariaLabel: "Ignored status",
      role: "status",
      state: "offline",
      tone: "danger",
    },
  });
  const root = handle.root();

  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("aria-label"), null);
  assert.equal(root.getAttribute("aria-describedby"), null);
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("data-aria-state"), "decorative");
  assert.equal(root.getAttribute("data-decorative"), "true");
  assert.equal(handle.queryByRole("status"), null);
  assert.equal(handle.queryByRole("img"), null);
  handle.unmount();
});

test("passes slot state and exposes live status-light state", async () => {
  const handle = mountInteraction(StatusLight, {
    props: {
      ariaLabel: "Service status",
      size: "md",
      state: "away",
      tone: "info",
    },
    slots: {
      default: (slotState: StatusLightSlotState) =>
        `${slotState.state}:${slotState.tone}:${slotState.size}:${slotState.ariaState}:${slotState.decorative}`,
    },
  });
  const exposed = handle.exposes<StatusLightExpose>();
  const root = handle.root();

  assert.ok(exposed.element === root);
  assert.equal(exposed.state, "away");
  assert.equal(exposed.tone, "info");
  assert.equal(exposed.size, "md");
  assert.equal(exposed.ariaState, "img");
  assert.equal(exposed.decorative, false);
  assert.equal(root.textContent, "away:info:md:img:false");

  await handle.wrapper.setProps({ ariaHidden: true, size: "lg", state: "offline", tone: "danger" });
  assert.equal(exposed.state, "offline");
  assert.equal(exposed.tone, "danger");
  assert.equal(exposed.size, "lg");
  assert.equal(exposed.ariaState, "decorative");
  assert.equal(exposed.decorative, true);
  assert.equal(root.getAttribute("data-state"), "offline");
  assert.equal(root.getAttribute("data-tone"), "danger");
  assert.equal(root.getAttribute("data-size"), "lg");
  assert.equal(root.textContent, "offline:danger:lg:decorative:true");
  handle.unmount();
});
