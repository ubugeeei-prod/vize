import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import {
  normalizeCalloutIdReferenceList,
  normalizeCalloutLabel,
  resolveCalloutAriaState,
  resolveCalloutLive,
} from "./callout-runtime.ts";
import Callout from "./callout.vue";
import type { CalloutExpose, CalloutSlotState } from "./callout.ts";

function formatCalloutSlotState(state: CalloutSlotState): string {
  return [
    state.state,
    state.role,
    state.ariaState,
    state.live ?? "off",
    state.atomic ? "true" : "false",
    state.tone,
    state.density,
    state.hasTitle ? "true" : "false",
    state.hasActions ? "true" : "false",
  ].join(":");
}

test("renders a labelled static note with structured parts by default", async () => {
  const handle = mountInteraction(Callout, {
    slots: {
      default: "Uploads continue in the background.",
      description: "Large files may take a few minutes.",
      icon: "i",
      title: "Upload queued",
    },
  });
  const root = handle.getByRole("note", { name: "Upload queued" });
  const title = root.querySelector("[data-vize-ui='callout-title']");
  const description = root.querySelector("[data-vize-ui='callout-description']");
  const icon = root.querySelector("[data-vize-ui='callout-icon']");

  assert.equal(root.tagName, "SECTION");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "callout");
  assert.equal(root.getAttribute("data-state"), "open");
  assert.equal(root.getAttribute("data-tone"), "neutral");
  assert.equal(root.getAttribute("data-density"), "comfortable");
  assert.equal(root.getAttribute("data-aria-state"), "note");
  assert.equal(root.getAttribute("data-live"), "off");
  assert.equal(root.getAttribute("data-has-icon"), "true");
  assert.equal(root.getAttribute("data-has-title"), "true");
  assert.equal(root.getAttribute("data-has-description"), "true");
  assert.equal(root.getAttribute("data-has-actions"), "false");
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.getAttribute("aria-atomic"), null);
  assert.ok(title instanceof HTMLElement);
  assert.ok(description instanceof HTMLElement);
  assert.ok(icon instanceof HTMLElement);
  assert.ok(title.id.length > 0);
  assert.ok(description.id.length > 0);
  assert.equal(root.getAttribute("aria-labelledby"), title.id);
  assert.equal(root.getAttribute("aria-describedby"), description.id);
  assert.equal(icon.getAttribute("aria-hidden"), "true");
  assert.equal(
    root.textContent,
    "iUpload queuedLarge files may take a few minutes.Uploads continue in the background.",
  );
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("supports polite status semantics with direct naming and interactive actions", async () => {
  const handle = mountInteraction(Callout, {
    props: {
      ariaLabel: "  Sync notice  ",
      atomic: false,
      density: "compact",
      role: "status",
      tone: "info",
    },
    slots: {
      actions: '<button type="button">Review</button>',
      default: "All changes are saved.",
      title: "Ignored by aria-label",
    },
  });
  const status = handle.getByRole("status", { name: "Sync notice" });
  const button = handle.getByRole("button", { name: "Review" });

  assert.equal(status.getAttribute("aria-label"), "Sync notice");
  assert.equal(status.getAttribute("aria-labelledby"), null);
  assert.equal(status.getAttribute("aria-live"), "polite");
  assert.equal(status.getAttribute("aria-atomic"), "false");
  assert.equal(status.getAttribute("data-tone"), "info");
  assert.equal(status.getAttribute("data-density"), "compact");
  assert.equal(status.getAttribute("data-live"), "polite");
  assert.equal(status.getAttribute("data-has-actions"), "true");
  assert.ok((await handle.tab()) === button);
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("renders a consumer component host without dropping callout hooks", () => {
  const ConsumerHost = defineComponent({
    inheritAttrs: false,
    setup(_, { attrs, slots }) {
      return () => h("aside", { ...attrs, "data-consumer-host": "callout" }, slots.default?.());
    },
  });
  const handle = mountInteraction(Callout, {
    props: {
      ariaLabel: "Deployment notice",
      as: ConsumerHost,
      id: "deploy-callout",
      tone: "info",
    },
    slots: {
      default: "Deployment succeeded.",
    },
  });
  const root = handle.getByRole("note", { name: "Deployment notice" });

  assert.equal(root.tagName, "ASIDE");
  assert.equal(root.getAttribute("id"), "deploy-callout");
  assert.equal(root.getAttribute("data-consumer-host"), "callout");
  assert.equal(root.getAttribute("data-vize-ui"), "callout");
  assert.equal(root.getAttribute("data-tone"), "info");
  assert.equal(root.textContent, "Deployment succeeded.");
  handle.unmount();
});

test("supports assertive alerts with consumer-owned title and description ids", () => {
  assert.equal(normalizeCalloutIdReferenceList(undefined), undefined);
  assert.equal(normalizeCalloutIdReferenceList("   "), undefined);
  assert.equal(normalizeCalloutIdReferenceList(" title   subtitle "), "title subtitle");
  assert.equal(normalizeCalloutLabel(undefined), undefined);
  assert.equal(normalizeCalloutLabel("   "), undefined);
  assert.equal(normalizeCalloutLabel("  Deploy failed  "), "Deploy failed");
  assert.equal(resolveCalloutAriaState({ ariaHidden: true, role: "alert" }), "decorative");
  assert.equal(resolveCalloutAriaState({ ariaHidden: false, role: "status" }), "status");
  assert.equal(resolveCalloutLive("alert"), "assertive");
  assert.equal(resolveCalloutLive("status"), "polite");
  assert.equal(resolveCalloutLive("note"), undefined);

  const labelled = document.createElement("span");
  labelled.id = "deploy-callout-label";
  labelled.textContent = "Deploy failed";
  document.body.append(labelled);
  const described = document.createElement("span");
  described.id = "deploy-callout-help";
  described.textContent = "Retry after checking secrets";
  document.body.append(described);
  const describedMore = document.createElement("span");
  describedMore.id = "deploy-callout-more";
  describedMore.textContent = "Rotate the deploy token before retrying";
  document.body.append(describedMore);
  const handle = mountInteraction(Callout, {
    props: {
      ariaDescribedby: " deploy-callout-help   deploy-callout-more ",
      ariaLabelledby: " deploy-callout-label ",
      descriptionId: "unused-description",
      role: "alert",
      titleId: "unused-title",
      tone: "danger",
    },
    slots: {
      description: "Internal description",
      title: "Internal title",
    },
  });
  const alert = handle.getByRole("alert", { name: "Deploy failed" });

  assert.equal(alert.getAttribute("aria-labelledby"), "deploy-callout-label");
  assert.equal(alert.getAttribute("aria-describedby"), "deploy-callout-help deploy-callout-more");
  assert.equal(alert.getAttribute("aria-live"), "assertive");
  assert.equal(alert.getAttribute("aria-atomic"), "true");
  assert.equal(alert.getAttribute("data-tone"), "danger");
  assert.equal(alert.getAttribute("data-live"), "assertive");
  assert.equal(alert.querySelector("[data-vize-ui='callout-title']")?.id, "unused-title");
  assert.equal(
    alert.querySelector("[data-vize-ui='callout-description']")?.id,
    "unused-description",
  );
  handle.unmount();
  labelled.remove();
  described.remove();
  describedMore.remove();
});

test("closed and decorative callouts stay mounted without announcing", async () => {
  const handle = mountInteraction(Callout, {
    props: {
      ariaHidden: true,
      ariaLabel: "Ignored message",
      open: false,
      role: "alert",
      tone: "warning",
    },
    slots: {
      title: "Maintenance window",
    },
  });
  const root = handle.root();

  assert.equal(root.getAttribute("hidden"), "");
  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("aria-label"), null);
  assert.equal(root.getAttribute("aria-labelledby"), null);
  assert.equal(root.getAttribute("aria-live"), null);
  assert.equal(root.getAttribute("aria-atomic"), null);
  assert.equal(root.getAttribute("data-state"), "closed");
  assert.equal(root.getAttribute("data-aria-state"), "decorative");
  assert.equal(root.getAttribute("data-live"), "off");
  assert.equal(handle.queryByRole("alert"), null);
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("passes slot state and exposes live Callout hooks", async () => {
  const handle = mountInteraction(Callout, {
    props: {
      density: "compact",
      role: "status",
      tone: "success",
    },
    slots: {
      default: formatCalloutSlotState,
    },
  });
  const exposed = handle.exposes<CalloutExpose>();
  const root = handle.root();

  assert.ok(exposed.element === root);
  assert.equal(exposed.state, "open");
  assert.equal(exposed.role, "status");
  assert.equal(exposed.ariaState, "status");
  assert.equal(exposed.live, "polite");
  assert.equal(exposed.tone, "success");
  assert.equal(exposed.density, "compact");
  assert.equal(exposed.hasTitle, false);
  assert.equal(exposed.titleId, undefined);
  assert.equal(root.textContent, "open:status:status:polite:true:success:compact:false:false");

  await handle.wrapper.setProps({
    density: "comfortable",
    open: false,
    role: "note",
    tone: "accent",
  });
  assert.equal(exposed.state, "closed");
  assert.equal(exposed.role, "note");
  assert.equal(exposed.ariaState, "decorative");
  assert.equal(exposed.live, undefined);
  assert.equal(exposed.tone, "accent");
  assert.equal(exposed.density, "comfortable");
  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("aria-labelledby"), null);
  assert.equal(root.getAttribute("data-state"), "closed");
  assert.equal(root.getAttribute("data-aria-state"), "decorative");
  assert.equal(root.getAttribute("data-live"), "off");
  assert.equal(root.textContent, "closed:note:decorative:off:true:accent:comfortable:false:false");
  handle.unmount();
});
