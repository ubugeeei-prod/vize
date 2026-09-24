import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import {
  NavigationMenuContent,
  NavigationMenuIndicator,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuRoot,
  NavigationMenuTrigger,
  NavigationMenuViewport,
} from "./navigation-menu.ts";
import type {
  NavigationMenuItemSlotState,
  NavigationMenuLinkSlotState,
  NavigationMenuMeasuredSlotState,
} from "./navigation-menu-types.ts";

const itemText = ({ value, state }: NavigationMenuItemSlotState) => `${value}:${state}`;
const measured = ({ state }: NavigationMenuMeasuredSlotState) => `measured:${state}`;
const linkText = ({ active }: NavigationMenuLinkSlotState) => `link:${active}`;

function renderMenu() {
  return h(
    NavigationMenuRoot,
    { ariaLabel: "Primary", defaultValue: "learn", id: "primary" },
    {
      default: () => [
        h(NavigationMenuList, null, {
          default: () => [
            h(
              NavigationMenuItem,
              { value: "learn" },
              {
                default: () => [
                  h(NavigationMenuTrigger, null, { default: itemText }),
                  h(NavigationMenuContent, null, {
                    default: () =>
                      h(
                        NavigationMenuLink,
                        { href: "/guide", active: true },
                        { default: linkText },
                      ),
                  }),
                ],
              },
            ),
            h(NavigationMenuIndicator, null, { default: measured }),
          ],
        }),
        h(NavigationMenuViewport, null, { default: measured }),
      ],
    },
  );
}

function fixture(name: string, file: string, marker: string): RuntimeFixture {
  return {
    name,
    sourceFile: `families/navigation/navigation-menu/${file}`,
    render: renderMenu,
    assertServerMarkup(html) {
      assert.match(html, new RegExp(`data-vize-ui="${marker}"`));
      assert.match(html, /aria-label="Primary"/);
      assert.match(html, /aria-expanded="true"/);
      assert.match(html, /aria-controls="primary-content-value-learn"/);
      assert.match(html, /learn:open/);
      assert.match(html, /link:true/);
      assert.match(html, /measured:open/);
    },
    assertHydratedDom(host) {
      const trigger = host.querySelector("#primary-trigger-value-learn");
      assert.ok(trigger instanceof HTMLButtonElement);
      assert.equal(trigger.getAttribute("aria-expanded"), "true");
      const content = host.querySelector("#primary-content-value-learn");
      assert.ok(content instanceof HTMLDivElement);
      assert.equal(content.getAttribute("aria-labelledby"), "primary-trigger-value-learn");
      assert.ok(host.querySelector(`[data-vize-ui="${marker}"]`) instanceof HTMLElement);
    },
  };
}

export const navigationMenuRuntimeFixtures: readonly RuntimeFixture[] = [
  fixture("navigation-menu", "navigation-menu-root.vue", "navigation-menu"),
  fixture("navigation-menu-list", "navigation-menu-list.vue", "navigation-menu-list"),
  fixture("navigation-menu-item", "navigation-menu-item.vue", "navigation-menu-item"),
  fixture("navigation-menu-trigger", "navigation-menu-trigger.vue", "navigation-menu-trigger"),
  fixture("navigation-menu-content", "navigation-menu-content.vue", "navigation-menu-content"),
  fixture("navigation-menu-link", "navigation-menu-link.vue", "navigation-menu-link"),
  fixture(
    "navigation-menu-indicator",
    "navigation-menu-indicator.vue",
    "navigation-menu-indicator",
  ),
  fixture("navigation-menu-viewport", "navigation-menu-viewport.vue", "navigation-menu-viewport"),
];
