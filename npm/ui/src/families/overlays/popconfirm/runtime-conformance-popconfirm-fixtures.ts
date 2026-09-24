import assert from "node:assert/strict";

import { h } from "vue";

import {
  PopconfirmCancel,
  PopconfirmConfirm,
  PopconfirmContent,
  PopconfirmRoot,
  PopconfirmTrigger,
} from "./popconfirm.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const popconfirmFamilyRoot = "families/overlays/popconfirm/";

function popconfirm(): ReturnType<typeof h> {
  return h(PopconfirmRoot, { defaultOpen: true, id: "runtime-popconfirm" }, () => [
    h(PopconfirmTrigger, null, () => "Delete"),
    h(PopconfirmContent, { portalDisabled: true, title: "Delete?" }, () => [
      h(PopconfirmCancel, null, () => "Keep"),
      h(PopconfirmConfirm, null, () => "Delete"),
    ]),
  ]);
}

function assertPart(host: HTMLElement, name: string): HTMLElement {
  const element = host.querySelector(`[data-vize-ui="${name}"]`);
  assert.ok(element instanceof HTMLElement, name);
  return element;
}

export const popconfirmRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "popconfirm-root",
    sourceFile: `${popconfirmFamilyRoot}popconfirm-root.vue`,
    render: popconfirm,
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="popconfirm-root"/);
      assert.match(html, /data-popconfirm-state="idle"/);
    },
    assertHydratedDom(host) {
      assertPart(host, "popconfirm-root");
    },
  },
  {
    name: "popconfirm-trigger",
    sourceFile: `${popconfirmFamilyRoot}popconfirm-trigger.vue`,
    render: popconfirm,
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="popconfirm-trigger"/);
      assert.match(html, /aria-expanded="true"/);
    },
    assertHydratedDom(host) {
      assert.ok(assertPart(host, "popconfirm-trigger") instanceof HTMLButtonElement);
    },
  },
  {
    name: "popconfirm-content",
    sourceFile: `${popconfirmFamilyRoot}popconfirm-content.vue`,
    render: popconfirm,
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="popconfirm-content"/);
      assert.match(html, /aria-labelledby="runtime-popconfirm-title"/);
    },
    assertHydratedDom(host) {
      assertPart(host, "popconfirm-content");
    },
  },
  {
    name: "popconfirm-confirm",
    sourceFile: `${popconfirmFamilyRoot}popconfirm-confirm.vue`,
    render: popconfirm,
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-popconfirm-confirm"/);
    },
    assertHydratedDom(host) {
      assert.ok(assertPart(host, "popconfirm-confirm") instanceof HTMLButtonElement);
    },
  },
  {
    name: "popconfirm-cancel",
    sourceFile: `${popconfirmFamilyRoot}popconfirm-cancel.vue`,
    render: popconfirm,
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-popconfirm-cancel"/);
    },
    assertHydratedDom(host) {
      assert.ok(assertPart(host, "popconfirm-cancel") instanceof HTMLButtonElement);
    },
  },
];
