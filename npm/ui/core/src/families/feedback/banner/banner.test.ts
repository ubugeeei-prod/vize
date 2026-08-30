import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, markRaw } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import Banner from "./banner.vue";
import type { BannerExpose, BannerSlotState } from "./banner.ts";

test("renders a named persistent region with deterministic title and description ids", () => {
  const handle = mountInteraction(Banner, {
    props: {
      description: "Scheduled from 02:00 to 02:15 UTC.",
      title: "System maintenance",
      tone: "warning",
    },
    slots: {
      default: (slotState: BannerSlotState) =>
        `${slotState.state}:${slotState.role}:${slotState.tone}:${slotState.live}:${slotState.named}`,
    },
  });
  const banner = handle.getByRole("region", { name: "System maintenance" });
  const title = banner.querySelector("[data-vize-ui='banner-title']");
  const description = banner.querySelector("[data-vize-ui='banner-description']");

  assert.equal(banner.tagName, "SECTION");
  assert.equal(banner.getAttribute("data-vize-ui"), "banner");
  assert.equal(banner.getAttribute("part"), "root");
  assert.equal(banner.getAttribute("data-state"), "open");
  assert.equal(banner.getAttribute("data-tone"), "warning");
  assert.equal(banner.getAttribute("data-role"), "region");
  assert.equal(banner.getAttribute("data-live"), "off");
  assert.equal(banner.getAttribute("data-named"), "true");
  assert.equal(banner.getAttribute("data-aria-state"), "named");
  assert.ok(title instanceof HTMLElement);
  assert.ok(description instanceof HTMLElement);
  assert.equal(title.id, `${banner.id}-title`);
  assert.equal(description.id, `${banner.id}-description`);
  assert.equal(banner.getAttribute("aria-labelledby"), title.id);
  assert.equal(banner.getAttribute("aria-describedby"), description.id);
  assert.equal(banner.getAttribute("aria-live"), null);
  assert.equal(banner.getAttribute("aria-atomic"), null);
  assert.equal(banner.textContent?.includes("open:region:warning:off:true"), true);
  handle.unmount();
});

test("normalizes explicit ARIA labels and external descriptions before title ids", () => {
  const handle = mountInteraction(Banner, {
    props: {
      ariaDescribedby: "external-help",
      ariaLabel: "Billing notice",
      description: "Your invoice is ready.",
      title: "Ignored as the accessible name",
    },
  });
  const banner = handle.getByRole("region", { name: "Billing notice" });
  const description = banner.querySelector("[data-vize-ui='banner-description']");

  assert.ok(description instanceof HTMLElement);
  assert.equal(banner.getAttribute("aria-label"), "Billing notice");
  assert.equal(banner.getAttribute("aria-labelledby"), null);
  assert.equal(banner.getAttribute("aria-describedby"), `external-help ${description.id}`);
  handle.unmount();
});

test("supports status and alert live-role banners", () => {
  const statusHandle = mountInteraction(Banner, {
    props: {
      atomic: false,
      role: "status",
      tone: "info",
    },
    slots: {
      title: "Deploy status",
    },
  });
  const status = statusHandle.getByRole("status", { name: "Deploy status" });

  assert.equal(status.getAttribute("aria-labelledby"), `${status.id}-title`);
  assert.equal(status.getAttribute("aria-live"), "polite");
  assert.equal(status.getAttribute("aria-atomic"), "false");
  assert.equal(status.getAttribute("data-aria-state"), "live");
  assert.equal(status.getAttribute("data-live"), "polite");
  statusHandle.unmount();

  const alertHandle = mountInteraction(Banner, {
    props: {
      ariaLabel: "Payment failed",
      role: "alert",
      tone: "danger",
    },
  });
  const alert = alertHandle.getByRole("alert", { name: "Payment failed" });

  assert.equal(alert.getAttribute("aria-live"), "assertive");
  assert.equal(alert.getAttribute("aria-atomic"), "true");
  assert.equal(alert.getAttribute("data-live"), "assertive");
  alertHandle.unmount();
});

test("suppresses an unnamed region role instead of emitting an unnamed landmark", () => {
  const handle = mountInteraction(Banner, {
    props: {
      ariaLabel: "   ",
      title: "   ",
    },
    slots: {
      default: "Unlabelled body",
    },
  });
  const root = handle.root();

  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("aria-label"), null);
  assert.equal(root.getAttribute("aria-labelledby"), null);
  assert.equal(root.getAttribute("data-role"), "region");
  assert.equal(root.getAttribute("data-named"), "false");
  assert.equal(root.getAttribute("data-aria-state"), "unnamed");
  assert.equal(handle.queryByRole("region"), null);
  handle.unmount();
});

test("requests controlled dismissal without mutating local visibility", async () => {
  const handle = mountInteraction(Banner, {
    props: {
      dismissLabel: "Close update",
      dismissible: true,
      title: "Release available",
    },
    record: ["dismiss", "update:open"],
  });
  const button = handle.getByRole("button", { name: "Close update" });

  await handle.click(button);

  assert.equal(handle.root().getAttribute("data-state"), "open");
  assert.deepEqual(
    handle.recorded().map((entry) => [entry.event, entry.payload.length]),
    [
      ["update:open", 1],
      ["dismiss", 1],
    ],
  );
  assert.deepEqual(handle.recorded()[0]?.payload, [false]);
  assert.ok(handle.recorded()[1]?.payload[0] instanceof MouseEvent);
  handle.unmount();
});

test("falls back to a named dismiss control for blank labels", () => {
  const handle = mountInteraction(Banner, {
    props: {
      dismissLabel: "   ",
      dismissible: true,
      title: "Release available",
    },
  });
  const button = handle.getByRole("button", { name: "Dismiss banner" });

  assert.equal(button.getAttribute("aria-label"), "Dismiss banner");
  assert.equal(button.textContent, "Dismiss banner");
  handle.unmount();
});

test("hides closed banners and exposes closed state", () => {
  const handle = mountInteraction(Banner, {
    props: {
      open: false,
      title: "Hidden update",
    },
  });
  const exposed = handle.exposes<BannerExpose>();
  const root = handle.root();

  assert.equal(root.tagName, "SECTION");
  assert.equal(root.hidden, true);
  assert.equal(root.getAttribute("aria-hidden"), "true");
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("aria-labelledby"), null);
  assert.equal(root.getAttribute("data-state"), "closed");
  assert.ok(exposed.element === root);
  assert.equal(exposed.state, "closed");
  assert.equal(exposed.named, true);
  handle.unmount();
});

test("forwards root attributes and slots through a custom as component", () => {
  const CustomBannerHost = defineComponent({
    name: "CustomBannerHost",
    inheritAttrs: false,
    setup(_, { attrs, expose, slots }) {
      let host: HTMLElement | null = null;

      expose({
        focus(options?: FocusOptions) {
          host?.focus(options);
        },
      });

      return () =>
        h(
          "article",
          {
            ...attrs,
            ref: (node: unknown) => {
              host = node instanceof HTMLElement ? node : null;
            },
            "data-forwarded": "yes",
            tabindex: -1,
          },
          slots.default?.(),
        );
    },
  });
  const handle = mountInteraction(Banner, {
    props: {
      as: markRaw(CustomBannerHost),
      title: "Workspace notice",
      tone: "accent",
    },
    attrs: {
      "data-consumer": "kept",
    },
    slots: {
      actions: (slotState: BannerSlotState) => h("a", { href: "/updates" }, slotState.titleId),
    },
  });
  const article = handle.root();
  const exposed = handle.exposes<BannerExpose>();

  assert.equal(article.tagName, "ARTICLE");
  assert.equal(article.getAttribute("role"), "region");
  assert.equal(article.getAttribute("aria-labelledby"), `${article.id}-title`);
  assert.equal(article.getAttribute("data-forwarded"), "yes");
  assert.equal(article.getAttribute("data-consumer"), "kept");
  assert.equal(article.getAttribute("data-tone"), "accent");
  assert.match(article.textContent ?? "", new RegExp(`${article.id}-title`));
  assert.equal(exposed.element !== null, true);
  exposed.focus();
  assert.equal(handle.activeElement(), article);
  handle.unmount();
});

test("does not focus closed custom hosts", () => {
  const FocusableHost = defineComponent({
    name: "ClosedBannerFocusableHost",
    inheritAttrs: false,
    setup(_, { attrs, expose, slots }) {
      let host: HTMLElement | null = null;

      expose({
        focus(options?: FocusOptions) {
          host?.focus(options);
        },
      });

      return () =>
        h(
          "article",
          {
            ...attrs,
            ref: (node: unknown) => {
              host = node instanceof HTMLElement ? node : null;
            },
            tabindex: -1,
          },
          slots.default?.(),
        );
    },
  });
  const handle = mountInteraction(Banner, {
    props: {
      as: markRaw(FocusableHost),
      open: false,
      title: "Hidden update",
    },
  });
  const exposed = handle.exposes<BannerExpose>();

  exposed.focus();
  assert.notEqual(handle.activeElement(), handle.root());
  handle.unmount();
});
