import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";

import BreadcrumbRoot from "./breadcrumb.vue";
import BreadcrumbItem from "./breadcrumb-item.vue";
import BreadcrumbLink from "./breadcrumb-link.vue";
import BreadcrumbList from "./breadcrumb-list.vue";
import BreadcrumbSeparator from "./breadcrumb-separator.vue";
import type {
  BreadcrumbItemExpose,
  BreadcrumbItemSlotState,
  BreadcrumbLinkExpose,
  BreadcrumbLinkSlotState,
  BreadcrumbRootExpose,
  BreadcrumbRootSlotState,
  BreadcrumbSeparatorExpose,
} from "./breadcrumb-types.ts";
import { mountInteraction } from "./testing/mount.ts";

function breadcrumbTree() {
  return h(BreadcrumbList, null, {
    default: () => [
      h(BreadcrumbItem, null, {
        default: () => [
          h(BreadcrumbLink, { href: "/" }, { default: () => "Home" }),
          h(BreadcrumbSeparator, null, { default: () => "/" }),
        ],
      }),
      h(BreadcrumbItem, null, {
        default: () => [
          h(BreadcrumbLink, { href: "/docs" }, { default: () => "Docs" }),
          h(BreadcrumbSeparator, null, { default: () => "/" }),
        ],
      }),
      h(
        BreadcrumbItem,
        { current: true },
        {
          default: () => h(BreadcrumbLink, { current: true }, { default: () => "API" }),
        },
      ),
    ],
  });
}

test("renders a labelled native breadcrumb landmark with ordered semantics", () => {
  const handle = mountInteraction(BreadcrumbRoot, {
    slots: { default: breadcrumbTree },
  });
  const root = handle.root();
  const list = root.querySelector('[data-vize-ui="breadcrumb-list"]');
  const links = root.querySelectorAll('[data-vize-ui="breadcrumb-link"]');
  const items = root.querySelectorAll('[data-vize-ui="breadcrumb-item"]');
  const separators = root.querySelectorAll('[data-vize-ui="breadcrumb-separator"]');

  assert.equal(root.tagName, "NAV");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "breadcrumb");
  assert.equal(root.getAttribute("aria-label"), "Breadcrumb");
  assert.equal(root.getAttribute("class"), null);
  assert.equal(root.getAttribute("style"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.ok(list instanceof HTMLOListElement);
  assert.equal(list.getAttribute("part"), "list");
  assert.equal(list.getAttribute("role"), null);
  assert.equal(items.length, 3);
  assert.equal(items[2]?.getAttribute("data-current"), "true");
  assert.equal(links.length, 3);
  assert.equal(links[0]?.getAttribute("href"), "/");
  assert.equal(links[1]?.getAttribute("href"), "/docs");
  assert.equal(links[2]?.getAttribute("aria-current"), "page");
  assert.equal(links[2]?.getAttribute("data-current"), "true");
  assert.equal(separators.length, 2);
  for (const separator of separators) {
    assert.equal(separator.getAttribute("aria-hidden"), "true");
    assert.equal(separator.getAttribute("role"), "presentation");
    assert.equal(separator.getAttribute("part"), "separator");
  }
  assert.equal(root.textContent, "Home/Docs/API");
  handle.unmount();
});

test("supports custom landmark labels, route-aware current values, and consumer attrs", async () => {
  const handle = mountInteraction(BreadcrumbRoot, {
    attrs: { "data-suite": "router" },
    props: { as: "section", label: "Workspace path" },
    slots: {
      default: ({ label }: BreadcrumbRootSlotState) =>
        h(
          BreadcrumbList,
          { as: "ul" },
          {
            default: () =>
              h(
                BreadcrumbItem,
                { as: "div", current: true },
                {
                  default: ({ current }: BreadcrumbItemSlotState) =>
                    h(
                      BreadcrumbLink,
                      { as: "span", current: "location" },
                      {
                        default: ({ ariaCurrent }: BreadcrumbLinkSlotState) =>
                          `${label}:${current}:${ariaCurrent}`,
                      },
                    ),
                },
              ),
          },
        ),
    },
  });
  const root = handle.root();
  const item = root.querySelector('[data-vize-ui="breadcrumb-item"]');
  const link = root.querySelector('[data-vize-ui="breadcrumb-link"]');

  assert.equal(root.tagName, "SECTION");
  assert.equal(root.getAttribute("aria-label"), "Workspace path");
  assert.equal(root.getAttribute("data-suite"), "router");
  assert.equal(root.querySelector('[data-vize-ui="breadcrumb-list"]')?.tagName, "UL");
  assert.equal(item?.tagName, "DIV");
  assert.equal(item?.getAttribute("data-current"), "true");
  assert.equal(link?.tagName, "SPAN");
  assert.equal(link?.getAttribute("href"), null);
  assert.equal(link?.getAttribute("aria-current"), "location");
  assert.equal(link?.getAttribute("data-current"), "true");
  assert.equal(root.textContent, "Workspace path:true:location");
  assert.equal(await handle.tab(), null);
  handle.unmount();
});

test("normalizes unsafe link destinations without disabling router component attrs", () => {
  const handle = mountInteraction(BreadcrumbRoot, {
    slots: {
      default: () =>
        h(BreadcrumbList, null, {
          default: () => [
            h(BreadcrumbItem, null, {
              default: () =>
                h(BreadcrumbLink, { href: " javascript:alert(1)" }, { default: () => "Bad" }),
            }),
            h(BreadcrumbItem, null, {
              default: () => h(BreadcrumbLink, { href: " /docs " }, { default: () => "Docs" }),
            }),
            h(BreadcrumbItem, null, {
              default: () =>
                h(
                  BreadcrumbLink,
                  { as: "span", "data-route": "/settings", href: "data:text/html,hi" },
                  { default: () => "Settings" },
                ),
            }),
          ],
        }),
    },
  });
  const links = handle.root().querySelectorAll('[data-vize-ui="breadcrumb-link"]');

  assert.equal(links[0]?.getAttribute("href"), null);
  assert.equal(links[1]?.getAttribute("href"), "/docs");
  assert.equal(links[2]?.getAttribute("href"), null);
  assert.equal(links[2]?.getAttribute("data-route"), "/settings");
  handle.unmount();
});

test("exposes live root, item, link, and separator state", async () => {
  const Inspector = defineComponent({
    name: "BreadcrumbInspector",
    setup() {
      const label = ref("Project path");
      const current = ref<false | "page" | "step">(false);
      const root = ref<BreadcrumbRootExpose | null>(null);
      const item = ref<BreadcrumbItemExpose | null>(null);
      const link = ref<BreadcrumbLinkExpose | null>(null);
      const separator = ref<BreadcrumbSeparatorExpose | null>(null);
      return { current, item, label, link, root, separator };
    },
    render() {
      return h(
        BreadcrumbRoot,
        { ref: "root", label: this.label },
        {
          default: () =>
            h(BreadcrumbList, null, {
              default: () => [
                h(BreadcrumbItem, null, {
                  default: () => [
                    h(BreadcrumbLink, { href: "/" }, { default: () => "Home" }),
                    h(BreadcrumbSeparator, { ref: "separator" }, { default: () => "/" }),
                  ],
                }),
                h(
                  BreadcrumbItem,
                  { ref: "item", current: this.current !== false },
                  {
                    default: () =>
                      h(
                        BreadcrumbLink,
                        { ref: "link", current: this.current },
                        {
                          default: ({ current, ariaCurrent }: BreadcrumbLinkSlotState) =>
                            `${current}:${ariaCurrent ?? "none"}`,
                        },
                      ),
                  },
                ),
              ],
            }),
        },
      );
    },
  });
  const handle = mountInteraction(Inspector);
  const vm = handle.wrapper.vm as unknown as {
    current: false | "page" | "step";
    item: BreadcrumbItemExpose | null;
    label: string;
    link: BreadcrumbLinkExpose | null;
    root: BreadcrumbRootExpose | null;
    separator: BreadcrumbSeparatorExpose | null;
  };

  assert.equal(vm.root?.label, "Project path");
  assert.equal(vm.item?.current, false);
  assert.equal(vm.link?.current, false);
  assert.equal(vm.link?.ariaCurrent, undefined);
  assert.equal(vm.separator?.decorative, true);
  assert.equal(handle.root().textContent, "Home/false:none");

  vm.label = "Documentation path";
  vm.current = "step";
  await nextTick();

  assert.equal(vm.root?.label, "Documentation path");
  assert.equal(vm.item?.current, true);
  assert.equal(vm.link?.current, true);
  assert.equal(vm.link?.ariaCurrent, "step");
  assert.equal(handle.root().getAttribute("aria-label"), "Documentation path");
  assert.equal(
    handle.root().querySelector('[data-vize-ui="breadcrumb-link"][data-current="true"]')
      ?.textContent,
    "true:step",
  );

  vm.link?.focus();
  assert.equal(
    document.activeElement,
    handle.root().querySelector('[data-vize-ui="breadcrumb-link"][data-current="true"]'),
  );
  handle.unmount();
});
