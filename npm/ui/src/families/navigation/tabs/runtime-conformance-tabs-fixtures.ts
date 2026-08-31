import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { TabsContent, TabsList, TabsRoot, TabsTrigger } from "./tabs.ts";
import type {
  TabsContentSlotState,
  TabsListSlotState,
  TabsTriggerSlotState,
} from "./tabs-types.ts";

export const tabsRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "tabs",
    sourceFile: "families/navigation/tabs/tabs-root.vue",
    render: () =>
      h(
        TabsRoot,
        { defaultValue: "overview", id: "account-tabs" },
        {
          default: () => [
            h(
              TabsList,
              { ariaLabel: "Account sections" },
              {
                default: () => [
                  h(TabsTrigger, { value: "overview" }, () => "Overview"),
                  h(TabsTrigger, { value: "billing" }, () => "Billing"),
                ],
              },
            ),
            h(TabsContent, { value: "overview" }, () => "Overview panel"),
            h(TabsContent, { value: "billing" }, () => "Billing panel"),
          ],
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /id="account-tabs"/);
      assert.match(html, /data-vize-ui="tabs-root"/);
      assert.match(html, /role="tablist"/);
      assert.match(html, /aria-label="Account sections"/);
      assert.match(html, /role="tab"/);
      assert.match(html, /aria-selected="true"/);
      assert.match(html, /aria-controls="account-tabs-content-value-overview"/);
      assert.match(html, /role="tabpanel"/);
      assert.match(html, /aria-labelledby="account-tabs-trigger-value-overview"/);
      assert.match(html, /Overview panel/);
      assert.match(html, /hidden/);
      assert.match(html, /Billing panel/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="tabs-root"]');
      const list = host.querySelector('[data-vize-ui="tabs-list"]');
      const overview = host.querySelector('[data-vize-ui="tabs-trigger"][data-value="overview"]');
      const panel = host.querySelector('[data-vize-ui="tabs-content"][data-value="overview"]');
      const hiddenPanel = host.querySelector('[data-vize-ui="tabs-content"][data-value="billing"]');

      assert.ok(root instanceof HTMLDivElement);
      assert.equal(root.id, "account-tabs");
      assert.ok(list instanceof HTMLDivElement);
      assert.equal(list.getAttribute("role"), "tablist");
      assert.ok(overview instanceof HTMLButtonElement);
      assert.equal(overview.getAttribute("aria-selected"), "true");
      assert.equal(overview.getAttribute("aria-controls"), "account-tabs-content-value-overview");
      assert.ok(panel instanceof HTMLDivElement);
      assert.equal(panel.hidden, false);
      assert.equal(panel.getAttribute("aria-labelledby"), "account-tabs-trigger-value-overview");
      assert.ok(hiddenPanel instanceof HTMLDivElement);
      assert.equal(hiddenPanel.hidden, true);
    },
  },
  {
    name: "tabs-content",
    sourceFile: "families/navigation/tabs/tabs-content.vue",
    render: () =>
      h(
        TabsRoot,
        { defaultValue: "details", id: "content-tabs" },
        {
          default: () => [
            h(TabsList, null, {
              default: () => h(TabsTrigger, { value: "details" }, () => "Details"),
            }),
            h(
              TabsContent,
              { value: "details" },
              {
                default: ({ selected, state }: TabsContentSlotState) => `${selected}:${state}`,
              },
            ),
          ],
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /id="content-tabs-content-value-details"/);
      assert.match(html, /role="tabpanel"/);
      assert.match(html, /aria-labelledby="content-tabs-trigger-value-details"/);
      assert.match(html, /data-state="active"/);
      assert.match(html, /true:active/);
      assert.doesNotMatch(html, /hidden/);
    },
    assertHydratedDom(host) {
      const content = host.querySelector('[data-vize-ui="tabs-content"]');
      assert.ok(content instanceof HTMLDivElement);
      assert.equal(content.hidden, false);
      assert.equal(content.getAttribute("data-state"), "active");
      assert.equal(content.textContent, "true:active");
    },
  },
  {
    name: "tabs-list",
    sourceFile: "families/navigation/tabs/tabs-list.vue",
    render: () =>
      h(
        TabsRoot,
        { activationMode: "manual", defaultValue: "overview", id: "list-tabs" },
        {
          default: () =>
            h(
              TabsList,
              { ariaLabel: "Manual sections" },
              {
                default: ({ activationMode, listId }: TabsListSlotState) =>
                  `${listId}:${activationMode}`,
              },
            ),
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /id="list-tabs-list"/);
      assert.match(html, /role="tablist"/);
      assert.match(html, /aria-label="Manual sections"/);
      assert.match(html, /data-activation-mode="manual"/);
      assert.match(html, /list-tabs-list:manual/);
    },
    assertHydratedDom(host) {
      const list = host.querySelector('[data-vize-ui="tabs-list"]');
      assert.ok(list instanceof HTMLDivElement);
      assert.equal(list.getAttribute("role"), "tablist");
      assert.equal(list.getAttribute("data-activation-mode"), "manual");
      assert.equal(list.textContent, "list-tabs-list:manual");
    },
  },
  {
    name: "tabs-trigger",
    sourceFile: "families/navigation/tabs/tabs-trigger.vue",
    render: () =>
      h(
        TabsRoot,
        { defaultValue: "settings", id: "trigger-tabs" },
        {
          default: () =>
            h(TabsList, null, {
              default: () =>
                h(
                  TabsTrigger,
                  { value: "settings" },
                  {
                    default: ({ selected, state }: TabsTriggerSlotState) => `${selected}:${state}`,
                  },
                ),
            }),
        },
      ),
    assertServerMarkup(html) {
      assert.match(html, /id="trigger-tabs-trigger-value-settings"/);
      assert.match(html, /role="tab"/);
      assert.match(html, /aria-selected="true"/);
      assert.match(html, /data-state="active"/);
      assert.match(html, /true:active/);
    },
    assertHydratedDom(host) {
      const trigger = host.querySelector('[data-vize-ui="tabs-trigger"]');
      assert.ok(trigger instanceof HTMLButtonElement);
      assert.equal(trigger.getAttribute("aria-selected"), "true");
      assert.equal(trigger.getAttribute("data-state"), "active");
      assert.equal(trigger.textContent, "true:active");
    },
  },
];
