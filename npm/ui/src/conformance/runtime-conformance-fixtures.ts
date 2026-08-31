import assert from "node:assert/strict";

import { defineComponent, h, type VNode } from "vue";

import Button from "../families/actions/button/button.vue";
import ErrorSummary from "../families/form/error-summary/error-summary.vue";
import IdProvider from "../families/foundations/id/deterministic-id-provider.vue";
import { useDeterministicId } from "../families/foundations/id/deterministic-id.ts";
import LinkAnchor from "../families/navigation/link/link-anchor.vue";
import { skipLinkRuntimeFixture } from "../families/navigation/skip-link/runtime-conformance-skip-link-fixtures.ts";
import { collapsibleRuntimeFixtures } from "./runtime-conformance-collapsible-fixtures.ts";
import { calloutRuntimeFixture } from "../families/feedback/callout/runtime-conformance-callout-fixtures.ts";
import { tableRuntimeFixtures } from "../families/data/table/runtime-conformance-table-fixtures.ts";
import { layoutRuntimeFixtures } from "./runtime-conformance-layout-fixtures.ts";
import { meterRuntimeFixture } from "../families/feedback/meter/runtime-conformance-meter-fixtures.ts";
import { selectionRuntimeFixtures } from "../families/selection/runtime-conformance-selection-fixtures.ts";
import { paginationRuntimeFixtures } from "../families/navigation/pagination/runtime-conformance-pagination-fixtures.ts";
import PrimitiveElement from "../families/foundations/primitive/primitive-element.vue";
import { alertRuntimeFixture } from "../families/feedback/alert/runtime-conformance-alert-fixtures.ts";
import { alertDialogRuntimeFixture } from "../families/overlays/alert-dialog/runtime-conformance-alert-dialog-fixtures.ts";
import { badgeRuntimeFixture } from "../families/feedback/badge/runtime-conformance-badge-fixtures.ts";
import { bannerRuntimeFixture } from "../families/feedback/banner/runtime-conformance-banner-fixtures.ts";
import { blockUIRuntimeFixture } from "../families/feedback/block-ui/runtime-conformance-block-ui-fixtures.ts";
import { breadcrumbRuntimeFixtures } from "../families/navigation/breadcrumb/runtime-conformance-breadcrumb-fixtures.ts";
import { emptyStateRuntimeFixture } from "../families/feedback/empty-state/runtime-conformance-empty-state-fixtures.ts";
import { buttonGroupRuntimeFixtures } from "../families/actions/button-group/runtime-conformance-button-group-fixtures.ts";
import { copyButtonRuntimeFixture } from "../families/actions/copy-button/runtime-conformance-copy-button-fixtures.ts";
import { fullscreenButtonRuntimeFixture } from "../families/actions/fullscreen-button/runtime-conformance-fullscreen-button-fixtures.ts";
import { printButtonRuntimeFixture } from "../families/actions/print-button/runtime-conformance-print-button-fixtures.ts";
import { shareButtonRuntimeFixture } from "../families/actions/share-button/runtime-conformance-share-button-fixtures.ts";
import { toolbarRuntimeFixtures } from "../families/actions/toolbar/runtime-conformance-toolbar-fixtures.ts";
import { fieldRuntimeFixtures } from "../families/form/field/runtime-conformance-field-fixtures.ts";
import { progressBarRuntimeFixture } from "../families/feedback/progress-bar/runtime-conformance-progress-bar-fixtures.ts";
import { progressRuntimeFixture } from "../families/feedback/progress/runtime-conformance-progress-fixtures.ts";
import { ratingRuntimeFixture } from "../families/form/rating/runtime-conformance-rating-fixtures.ts";
import { sliderRuntimeFixture } from "../families/form/slider/runtime-conformance-slider-fixtures.ts";
import { spinnerRuntimeFixture } from "../families/feedback/spinner/runtime-conformance-spinner-fixtures.ts";
import { statusLightRuntimeFixture } from "../families/feedback/status-light/runtime-conformance-status-light-fixtures.ts";
import { stepperRuntimeFixtures } from "../families/navigation/stepper/runtime-conformance-stepper-fixtures.ts";
import { tabsRuntimeFixtures } from "../families/navigation/tabs/runtime-conformance-tabs-fixtures.ts";
import TextInput from "../families/form/input/text-input.vue";
import SearchField from "../families/form/search-field/search-field.vue";
import TextareaControl from "../families/form/textarea/textarea-control.vue";
import { typographyRuntimeFixtures } from "./runtime-conformance-typography-fixtures.ts";
import VisuallyHidden from "../families/accessibility/visually-hidden/visually-hidden.vue";

export interface RuntimeFixture {
  /** Stable name included in assertion diagnostics. */
  readonly name: string;
  readonly sourceFile: string;
  readonly render: () => VNode;
  /** Assert server output semantics before the browser repairs or normalizes DOM. */
  readonly assertServerMarkup: (html: string) => void;
  /** Assert hydrated accessibility semantics in a browser-like DOM. */
  readonly assertHydratedDom: (host: HTMLElement) => void;
}

const DeterministicIdProbe = defineComponent({
  name: "RuntimeConformanceDeterministicIdProbe",
  setup() {
    const id = useDeterministicId({ hint: "control" });
    return () => h("input", { id: id.value, "aria-label": "Email" });
  },
});

export const controlRuntimeFixtures: readonly RuntimeFixture[] = [
  alertRuntimeFixture,
  alertDialogRuntimeFixture,
  badgeRuntimeFixture,
  bannerRuntimeFixture,
  blockUIRuntimeFixture,
  calloutRuntimeFixture,
  ...tableRuntimeFixtures,
  ...breadcrumbRuntimeFixtures,
  ...tabsRuntimeFixtures,
  ...stepperRuntimeFixtures,
  emptyStateRuntimeFixture,
  ...collapsibleRuntimeFixtures,
  ...layoutRuntimeFixtures,
  ...typographyRuntimeFixtures,
  {
    name: "button",
    sourceFile: "families/actions/button/button.vue",
    render: () =>
      h(
        Button,
        { loading: true },
        {
          default: () => "Save changes",
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /<button/);
      assert.match(html, /aria-busy="true"/);
      assert.match(html, /data-state="loading"/);
      assert.match(html, /Save changes/);
      assert.match(html, /<\/button>/);
    },
    assertHydratedDom(host) {
      const button = host.querySelector('[data-vize-ui="button"]');
      assert.ok(button instanceof HTMLButtonElement);
      assert.equal(button.textContent, "Save changes");
      assert.equal(button.getAttribute("aria-busy"), "true");
    },
  },
  ...buttonGroupRuntimeFixtures,
  copyButtonRuntimeFixture,
  fullscreenButtonRuntimeFixture,
  printButtonRuntimeFixture,
  shareButtonRuntimeFixture,
  ...toolbarRuntimeFixtures,
  ...selectionRuntimeFixtures,
  {
    name: "deterministic-id-provider",
    sourceFile: "families/foundations/id/deterministic-id-provider.vue",
    render: () =>
      h(
        IdProvider,
        { prefix: "form", seed: "runtime" },
        {
          default: () => h(DeterministicIdProbe),
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /id="form-runtime-control-0"/);
      assert.match(html, /aria-label="Email"/);
    },
    assertHydratedDom(host) {
      const input = host.querySelector("input");
      assert.ok(input instanceof HTMLInputElement);
      assert.equal(input.id, "form-runtime-control-0");
      assert.equal(input.getAttribute("aria-label"), "Email");
    },
  },
  {
    name: "error-summary",
    sourceFile: "families/form/error-summary/error-summary.vue",
    render: () =>
      h(ErrorSummary, {
        autoFocus: false,
        fields: [{ id: "email", label: "Email", message: "Enter a valid address" }],
        heading: "There is a problem",
      }),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="error-summary-host"/);
      assert.match(html, /data-vize-ui="error-summary"/);
      assert.match(html, /role="group"/);
      assert.match(html, /tabindex="-1"/);
      assert.match(html, /href="#email"/);
      assert.match(html, /There is a problem/);
    },
    assertHydratedDom(host) {
      const summary = host.querySelector('[data-vize-ui="error-summary"]');
      assert.ok(summary instanceof HTMLElement);
      assert.equal(summary.getAttribute("role"), "group");
      assert.equal(summary.getAttribute("tabindex"), "-1");
      const link = summary.querySelector('[data-vize-ui="error-summary-link"]');
      assert.equal(link?.getAttribute("href"), "#email");
    },
  },
  ...fieldRuntimeFixtures,
  {
    name: "link",
    sourceFile: "families/navigation/link/link-anchor.vue",
    render: () =>
      h(
        LinkAnchor,
        { ariaCurrent: "page", href: "/docs", id: "docs-link" },
        {
          default: () => "Docs",
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /^<a/);
      assert.match(html, /id="docs-link"/);
      assert.match(html, /href="\/docs"/);
      assert.match(html, /aria-current="page"/);
      assert.match(html, /data-vize-ui="link"/);
      assert.match(html, /data-state="idle"/);
      assert.match(html, /Docs/);
      assert.match(html, /<\/a>$/);
    },
    assertHydratedDom(host) {
      const link = host.querySelector('[data-vize-ui="link"]');
      assert.ok(link instanceof HTMLAnchorElement);
      assert.equal(link.id, "docs-link");
      assert.equal(link.getAttribute("href"), "/docs");
      assert.equal(link.getAttribute("aria-current"), "page");
      assert.equal(link.textContent, "Docs");
    },
  },
  skipLinkRuntimeFixture,
  meterRuntimeFixture,
  ...paginationRuntimeFixtures,
  spinnerRuntimeFixture,
  statusLightRuntimeFixture,
  {
    name: "primitive",
    sourceFile: "families/foundations/primitive/primitive-element.vue",
    render: () =>
      h(
        PrimitiveElement,
        { as: "section" },
        {
          default: () => "Composable content",
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /^<section/);
      assert.match(html, /Composable content/);
      assert.match(html, /<\/section>$/);
    },
    assertHydratedDom(host) {
      const primitive = host.querySelector('[data-vize-ui="primitive"]');
      assert.ok(primitive instanceof HTMLElement);
      assert.equal(primitive.tagName, "SECTION");
    },
  },
  {
    name: "input",
    sourceFile: "families/form/input/text-input.vue",
    render: () =>
      h(TextInput, {
        ariaLabel: "Email",
        defaultValue: "hello@example.com",
        id: "email",
        name: "email",
        type: "email",
      }),
    assertServerMarkup(html) {
      assert.match(html, /^<input/);
      assert.match(html, /id="email"/);
      assert.match(html, /name="email"/);
      assert.match(html, /type="email"/);
      assert.match(html, /value="hello@example.com"/);
      assert.match(html, /aria-label="Email"/);
      assert.match(html, /data-vize-ui="input"/);
      assert.match(html, /data-state="editable"/);
      assert.match(html, /data-empty="false"/);
    },
    assertHydratedDom(host) {
      const input = host.querySelector('[data-vize-ui="input"]');
      assert.ok(input instanceof HTMLInputElement);
      assert.equal(input.type, "email");
      assert.equal(input.name, "email");
      assert.equal(input.value, "hello@example.com");
      assert.equal(input.getAttribute("data-state"), "editable");
      assert.equal(input.getAttribute("data-empty"), "false");
    },
  },
  {
    name: "textarea",
    sourceFile: "families/form/textarea/textarea-control.vue",
    render: () =>
      h(TextareaControl, {
        ariaLabel: "Bio",
        defaultValue: "Line one\nLine two",
        id: "bio",
        name: "bio",
        rows: 3,
      }),
    assertServerMarkup(html) {
      assert.match(html, /^<textarea/);
      assert.match(html, /id="bio"/);
      assert.match(html, /name="bio"/);
      assert.match(html, /rows="3"/);
      assert.match(html, /aria-label="Bio"/);
      assert.match(html, /data-vize-ui="textarea"/);
      assert.match(html, /data-state="editable"/);
      assert.match(html, /data-empty="false"/);
      assert.match(html, /Line one\nLine two/);
    },
    assertHydratedDom(host) {
      const textarea = host.querySelector('[data-vize-ui="textarea"]');
      assert.ok(textarea instanceof HTMLTextAreaElement);
      assert.equal(textarea.name, "bio");
      assert.equal(textarea.value, "Line one\nLine two");
      assert.equal(textarea.getAttribute("data-state"), "editable");
      assert.equal(textarea.getAttribute("data-empty"), "false");
    },
  },
  ratingRuntimeFixture,
  sliderRuntimeFixture,
  {
    name: "search-field",
    sourceFile: "families/form/search-field/search-field.vue",
    render: () =>
      h(SearchField, {
        ariaLabel: "Search",
        defaultValue: "vize",
        id: "query",
        name: "query",
      }),
    assertServerMarkup(html) {
      assert.match(html, /^<div/);
      assert.match(html, /role="search"/);
      assert.match(html, /data-vize-ui="search-field"/);
      assert.match(html, /id="query"/);
      assert.match(html, /type="search"/);
      assert.match(html, /value="vize"/);
      assert.match(html, /aria-label="Search"/);
      assert.match(html, /data-vize-ui="search-field-input"/);
      assert.match(html, /data-vize-ui="search-field-clear"/);
      assert.match(html, /id="query-clear"/);
    },
    assertHydratedDom(host) {
      const search = host.querySelector('[data-vize-ui="search-field"]');
      const input = host.querySelector('[data-vize-ui="search-field-input"]');
      const clear = host.querySelector('[data-vize-ui="search-field-clear"]');
      assert.ok(search instanceof HTMLElement);
      assert.ok(input instanceof HTMLInputElement);
      assert.ok(clear instanceof HTMLButtonElement);
      assert.equal(input.type, "search");
      assert.equal(input.name, "query");
      assert.equal(input.value, "vize");
      assert.equal(clear.id, "query-clear");
    },
  },
  progressRuntimeFixture,
  progressBarRuntimeFixture,
  {
    name: "visually-hidden",
    sourceFile: "families/accessibility/visually-hidden/visually-hidden.vue",
    render: () =>
      h(VisuallyHidden, null, {
        default: () => h("button", { type: "button" }, "Dismiss notification"),
      }),
    assertServerMarkup(html) {
      assert.match(html, /^<span/);
      assert.match(html, /data-vize-ui="visually-hidden"/);
      assert.match(html, /<button type="button">Dismiss notification<\/button>/);
    },
    assertHydratedDom(host) {
      const control = host.querySelector("button");
      assert.ok(control instanceof HTMLButtonElement);
      assert.equal(control.textContent, "Dismiss notification");
    },
  },
];
