import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { SplitterGroup, SplitterHandle, SplitterPanel } from "./splitter.ts";
import type {
  SplitterGroupSlotState,
  SplitterHandleSlotState,
  SplitterPanelSlotState,
} from "./splitter-types.ts";

function renderSplitter(
  panel: (slot: SplitterPanelSlotState) => string = () => "Panel",
  grip: (slot: SplitterHandleSlotState) => string = () => "",
  group: (slot: SplitterGroupSlotState) => string = () => "",
) {
  return h(
    SplitterGroup,
    { id: "workspace" },
    {
      default: (slot: SplitterGroupSlotState) => [
        group(slot),
        h(
          SplitterPanel,
          { id: "workspace-nav", defaultSize: 30, minSize: 10, collapsible: true },
          { default: panel },
        ),
        h(SplitterHandle, { ariaLabel: "Resize navigation" }, { default: grip }),
        h(SplitterPanel, { id: "workspace-main", defaultSize: 70 }, { default: panel }),
      ],
    },
  );
}

export const splitterRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "splitter-group",
    sourceFile: "families/layout/splitter/splitter-group.vue",
    render: () => renderSplitter(undefined, undefined, ({ state }) => `group:${state}`),
    assertServerMarkup(html) {
      assert.match(html, /id="workspace"/);
      assert.match(html, /data-vize-ui="splitter-group"/);
      assert.match(html, /display:flex/);
      assert.match(html, /group:idle/);
    },
    assertHydratedDom(host) {
      const group = host.querySelector('[data-vize-ui="splitter-group"]');
      assert.ok(group instanceof HTMLDivElement);
      assert.equal(group.getAttribute("data-orientation"), "horizontal");
      assert.equal(group.style.flexDirection, "row");
    },
  },
  {
    name: "splitter-panel",
    sourceFile: "families/layout/splitter/splitter-panel.vue",
    render: () => renderSplitter(({ size, state }) => `${size}:${state}`),
    assertServerMarkup(html) {
      assert.match(html, /id="workspace-nav"/);
      assert.match(html, /flex-grow:30/);
      assert.match(html, /30:expanded/);
      assert.match(html, /70:expanded/);
    },
    assertHydratedDom(host) {
      const nav = host.querySelector("#workspace-nav");
      assert.ok(nav instanceof HTMLDivElement);
      assert.equal(nav.style.flexGrow, "30");
      assert.equal(nav.textContent, "30:expanded");
    },
  },
  {
    name: "splitter-handle",
    sourceFile: "families/layout/splitter/splitter-handle.vue",
    render: () => renderSplitter(undefined, ({ value, state }) => `${value}:${state}`),
    assertServerMarkup(html) {
      assert.match(html, /role="separator"/);
      assert.match(html, /aria-valuenow="30"/);
      assert.match(html, /aria-valuemin="0"/);
      assert.match(html, /aria-controls="workspace-nav"/);
      assert.match(html, /30:idle/);
    },
    assertHydratedDom(host) {
      const separator = host.querySelector('[role="separator"]');
      assert.ok(separator instanceof HTMLDivElement);
      assert.equal(separator.getAttribute("aria-orientation"), "vertical");
      assert.equal(separator.getAttribute("tabindex"), "0");
      assert.equal(separator.textContent, "30:idle");
    },
  },
];
