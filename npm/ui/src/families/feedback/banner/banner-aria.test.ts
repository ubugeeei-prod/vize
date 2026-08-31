import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { normalizeBannerAria } from "./banner-aria.ts";

test("normalizes labelled live banners with deterministic descriptions", () => {
  assert.deepEqual(
    normalizeBannerAria({
      ariaDescribedby: " external-help   second-help ",
      ariaLabel: " Ignored ",
      ariaLabelledby: " visible-label ",
      atomic: true,
      descriptionId: "banner-description",
      hasDescription: true,
      hasTitle: true,
      role: "status",
      titleId: "banner-title",
    }),
    {
      ariaAtomic: "true",
      ariaDescribedby: "external-help second-help banner-description",
      ariaLabel: undefined,
      ariaLabelledby: "visible-label",
      ariaLive: "polite",
      ariaState: "live",
      live: "polite",
      named: true,
      role: "status",
    },
  );
});

test("suppresses unnamed region semantics", () => {
  assert.deepEqual(
    normalizeBannerAria({
      atomic: true,
      descriptionId: "banner-description",
      hasDescription: false,
      hasTitle: false,
      role: "region",
      titleId: "banner-title",
    }),
    {
      ariaAtomic: undefined,
      ariaDescribedby: undefined,
      ariaLabel: undefined,
      ariaLabelledby: undefined,
      ariaLive: undefined,
      ariaState: "unnamed",
      live: "off",
      named: false,
      role: undefined,
    },
  );
});

test("keeps alert banners live even when unnamed", () => {
  assert.deepEqual(
    normalizeBannerAria({
      atomic: false,
      descriptionId: "banner-description",
      hasDescription: false,
      hasTitle: false,
      role: "alert",
      titleId: "banner-title",
    }),
    {
      ariaAtomic: "false",
      ariaDescribedby: undefined,
      ariaLabel: undefined,
      ariaLabelledby: undefined,
      ariaLive: "assertive",
      ariaState: "live",
      live: "assertive",
      named: false,
      role: "alert",
    },
  );
});
