import assert from "node:assert/strict";

import { defineComponent, h, type VNode } from "vue";

import ActionButton from "./action-button.vue";
import CheckboxControl from "./checkbox-control.vue";
import IdProvider from "./deterministic-id-provider.vue";
import { useDeterministicId } from "./deterministic-id.ts";
import ErrorSummary from "./error-summary.vue";
import PrimitiveElement from "./primitive-element.vue";
import TextInput from "./text-input.vue";
import ToggleButton from "./toggle-button.vue";
import VisuallyHidden from "./visually-hidden.vue";

export interface RuntimeFixture {
  /** Stable name included in assertion diagnostics. */
  readonly name: string;
  /** Canonical SFC whose SSR and hydration behavior this fixture covers. */
  readonly sourceFile: string;
  /** Build a fresh vnode so no request can inherit another request's state. */
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
  {
    name: "button",
    sourceFile: "action-button.vue",
    render: () =>
      h(
        ActionButton,
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
  {
    name: "checkbox",
    sourceFile: "checkbox-control.vue",
    render: () =>
      h(CheckboxControl, {
        ariaLabel: "Accept terms",
        defaultChecked: true,
      }),
    assertServerMarkup(html) {
      assert.match(html, /type="checkbox"/);
      assert.match(html, /aria-label="Accept terms"/);
      assert.match(html, /aria-checked="true"/);
      assert.match(html, /checked/);
    },
    assertHydratedDom(host) {
      const checkbox = host.querySelector('[data-vize-ui="checkbox"]');
      assert.ok(checkbox instanceof HTMLInputElement);
      assert.equal(checkbox.checked, true);
      assert.equal(checkbox.getAttribute("aria-checked"), "true");
    },
  },
  {
    name: "deterministic-id-provider",
    sourceFile: "deterministic-id-provider.vue",
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
    sourceFile: "error-summary.vue",
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
  {
    name: "primitive",
    sourceFile: "primitive-element.vue",
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
    sourceFile: "text-input.vue",
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
    name: "toggle",
    sourceFile: "toggle-button.vue",
    render: () =>
      h(
        ToggleButton,
        { defaultPressed: true },
        {
          default: () => "Bold",
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /^<button/);
      assert.match(html, /type="button"/);
      assert.match(html, /aria-pressed="true"/);
      assert.match(html, /data-vize-ui="toggle"/);
      assert.match(html, /data-state="pressed"/);
      assert.match(html, /Bold/);
    },
    assertHydratedDom(host) {
      const toggle = host.querySelector('[data-vize-ui="toggle"]');
      assert.ok(toggle instanceof HTMLButtonElement);
      assert.equal(toggle.type, "button");
      assert.equal(toggle.getAttribute("aria-pressed"), "true");
      assert.equal(toggle.getAttribute("data-state"), "pressed");
      assert.equal(toggle.textContent, "Bold");
    },
  },
  {
    name: "visually-hidden",
    sourceFile: "visually-hidden.vue",
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
