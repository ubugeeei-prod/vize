import assert from "node:assert/strict";

import { h } from "vue";

import RadioGroup from "./radio-group.vue";
import RadioGroupItem from "./radio-group-item.vue";
import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";

function renderRadioGroupFixture() {
  return h(
    RadioGroup,
    {
      ariaLabel: "Email frequency",
      defaultValue: "weekly",
      id: "frequency",
      name: "frequency",
      orientation: "horizontal",
      required: true,
    },
    {
      default: () => [
        h("label", [h(RadioGroupItem, { id: "daily-radio", value: "daily" }), "Daily"]),
        h("label", [h(RadioGroupItem, { id: "weekly-radio", value: "weekly" }), "Weekly"]),
      ],
    },
  );
}

function assertRadioGroupServerMarkup(html: string): void {
  assert.match(html, /^<div/);
  assert.match(html, /id="frequency"/);
  assert.match(html, /role="radiogroup"/);
  assert.match(html, /aria-label="Email frequency"/);
  assert.match(html, /aria-orientation="horizontal"/);
  assert.match(html, /aria-required="true"/);
  assert.match(html, /data-vize-ui="radio-group"/);
  assert.match(html, /data-state="selected"/);
  assert.match(html, /data-orientation="horizontal"/);
  assert.match(html, /type="radio"/);
  assert.match(html, /name="frequency"/);
  assert.match(html, /value="weekly"/);
  assert.match(html, /checked/);
}

function assertRadioGroupHydratedDom(host: HTMLElement): void {
  const group = host.querySelector('[data-vize-ui="radio-group"]');
  const daily = host.querySelector<HTMLInputElement>("#daily-radio");
  const weekly = host.querySelector<HTMLInputElement>("#weekly-radio");
  assert.ok(group instanceof HTMLDivElement);
  assert.ok(daily instanceof HTMLInputElement);
  assert.ok(weekly instanceof HTMLInputElement);
  assert.equal(group.getAttribute("role"), "radiogroup");
  assert.equal(group.getAttribute("aria-orientation"), "horizontal");
  assert.equal(group.getAttribute("data-value"), "weekly");
  assert.equal(daily.checked, false);
  assert.equal(weekly.checked, true);
  assert.equal(weekly.name, "frequency");
  assert.equal(weekly.required, true);
}

export const radioGroupRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "radio-group",
    sourceFile: "families/selection/radio-group/radio-group.vue",
    render: renderRadioGroupFixture,
    assertServerMarkup: assertRadioGroupServerMarkup,
    assertHydratedDom: assertRadioGroupHydratedDom,
  },
  {
    name: "radio-group-item",
    sourceFile: "families/selection/radio-group/radio-group-item.vue",
    render: renderRadioGroupFixture,
    assertServerMarkup: assertRadioGroupServerMarkup,
    assertHydratedDom: assertRadioGroupHydratedDom,
  },
];
