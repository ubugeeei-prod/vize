import assert from "node:assert/strict";

import { h } from "vue";

import {
  FloatingActionButton,
  SpeedDialAction,
  SpeedDialContent,
  SpeedDialRoot,
  SpeedDialTrigger,
} from "./floating-action-button.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const familyRoot = "families/actions/floating-action-button/";

function speedDial(): ReturnType<typeof h> {
  return h(SpeedDialRoot, { id: "runtime-speed-dial", defaultOpen: true }, () => [
    h(SpeedDialTrigger, { ariaLabel: "Create" }, () => "+"),
    h(SpeedDialContent, null, () => h(SpeedDialAction, { value: "note", label: "Note" })),
  ]);
}

function assertSelector(selector: string, type: typeof HTMLElement = HTMLElement) {
  return (host: HTMLElement) => {
    assert.ok(host.querySelector(selector) instanceof type);
  };
}

export const floatingActionButtonRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "floating-action-button",
    sourceFile: `${familyRoot}floating-action-button.vue`,
    render: () => h(FloatingActionButton, { ariaLabel: "Compose" }, () => "+"),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="floating-action-button"/);
      assert.match(html, /data-placement="bottom-end"/);
    },
    assertHydratedDom: assertSelector('[data-vize-ui="floating-action-button"]', HTMLButtonElement),
  },
  {
    name: "speed-dial-root",
    sourceFile: `${familyRoot}speed-dial-root.vue`,
    render: speedDial,
    assertServerMarkup(html) {
      assert.match(html, /id="runtime-speed-dial"/);
      assert.match(html, /data-state="open"/);
    },
    assertHydratedDom: assertSelector('[data-vize-ui="speed-dial-root"]'),
  },
  {
    name: "speed-dial-trigger",
    sourceFile: `${familyRoot}speed-dial-trigger.vue`,
    render: speedDial,
    assertServerMarkup(html) {
      assert.match(html, /aria-controls="runtime-speed-dial-content"/);
      assert.match(html, /aria-expanded="true"/);
    },
    assertHydratedDom: assertSelector('[data-vize-ui="speed-dial-trigger"]', HTMLButtonElement),
  },
  {
    name: "speed-dial-content",
    sourceFile: `${familyRoot}speed-dial-content.vue`,
    render: speedDial,
    assertServerMarkup(html) {
      assert.match(html, /role="menu"/);
      assert.match(html, /aria-labelledby="runtime-speed-dial-trigger"/);
    },
    assertHydratedDom: assertSelector('[data-vize-ui="speed-dial-content"]'),
  },
  {
    name: "speed-dial-action",
    sourceFile: `${familyRoot}speed-dial-action.vue`,
    render: speedDial,
    assertServerMarkup(html) {
      assert.match(html, /role="menuitem"/);
      assert.match(html, /aria-label="Note"/);
    },
    assertHydratedDom: assertSelector('[data-vize-ui="speed-dial-action"]', HTMLButtonElement),
  },
];
