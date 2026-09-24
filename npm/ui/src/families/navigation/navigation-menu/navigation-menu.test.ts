import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type { NavigationMenuRootExpose } from "./navigation-menu.ts";
import NavigationMenuContent from "./navigation-menu-content.vue";
import NavigationMenuIndicator from "./navigation-menu-indicator.vue";
import NavigationMenuItem from "./navigation-menu-item.vue";
import NavigationMenuLink from "./navigation-menu-link.vue";
import NavigationMenuList from "./navigation-menu-list.vue";
import NavigationMenuRoot from "./navigation-menu-root.vue";
import NavigationMenuTrigger from "./navigation-menu-trigger.vue";
import NavigationMenuViewport from "./navigation-menu-viewport.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function mountMenu(
  props: Record<string, unknown> = {},
  contentProps: Record<string, unknown> = {},
) {
  return mountInteraction(NavigationMenuRoot, {
    props: { id: "site", ariaLabel: "Main", ...props },
    record: ["update:modelValue", "change"],
    slots: {
      default: () => [
        h(NavigationMenuList, null, () => [
          h(NavigationMenuItem, { value: "products" }, () => [
            h(NavigationMenuTrigger, null, () => "Products"),
            h(NavigationMenuContent, contentProps, () => [
              h(NavigationMenuLink, { href: "/ui" }, () => "UI kit"),
              h(NavigationMenuLink, { href: "/cli" }, () => "CLI"),
            ]),
          ]),
          h(NavigationMenuItem, { value: "docs" }, () => [
            h(NavigationMenuTrigger, null, () => "Docs"),
            h(NavigationMenuContent, null, () =>
              h(NavigationMenuLink, { href: "/guide" }, () => "Guide"),
            ),
          ]),
          h(NavigationMenuItem, { value: "blog" }, () =>
            h(NavigationMenuLink, { href: "/blog", active: true }, () => "Blog"),
          ),
          h(NavigationMenuIndicator),
        ]),
        h(NavigationMenuViewport),
      ],
    },
  });
}

function trigger(handle: ReturnType<typeof mountMenu>, name: string): HTMLButtonElement {
  return handle.getByRole("button", { name }) as HTMLButtonElement;
}

function pointer(target: Element, type: string): void {
  target.dispatchEvent(new PointerEvent(type, { bubbles: false, pointerType: "mouse" }));
}

async function wait(ms: number): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, ms));
  await nextTick();
}

test("renders a labelled disclosure navigation with wired triggers and closed flyouts", () => {
  const handle = mountMenu();
  const nav = handle.root();
  const products = trigger(handle, "Products");
  assert.equal(nav.tagName, "NAV");
  assert.equal(nav.getAttribute("aria-label"), "Main");
  assert.equal(nav.getAttribute("data-state"), "closed");
  assert.equal(products.id, "site-trigger-value-products");
  assert.equal(products.getAttribute("aria-expanded"), "false");
  assert.equal(products.getAttribute("aria-controls"), "site-content-value-products");
  const closed = nav.querySelector<HTMLDivElement>("#site-content-value-products");
  assert.equal(closed?.hidden, true, "closed flyouts keep a hidden container");
  assert.equal(closed?.childElementCount, 0, "closed flyout content is not rendered");
  const blog = handle.getByRole("link", { name: "Blog" });
  assert.equal(blog.getAttribute("aria-current"), "page");
  assert.equal(
    nav.querySelector('[data-vize-ui="navigation-menu-indicator"]')?.getAttribute("aria-hidden"),
    "true",
  );
  handle.unmount();
});

test("click toggles a flyout inline after its trigger", async () => {
  const handle = mountMenu();
  const products = trigger(handle, "Products");
  await handle.click(products);
  assert.equal(products.getAttribute("aria-expanded"), "true");
  const content = handle.root().querySelector("#site-content-value-products");
  assert.ok(content instanceof HTMLDivElement);
  assert.equal(content.getAttribute("aria-labelledby"), products.id);
  assert.equal(content.previousElementSibling, products, "flyouts keep native tab order");
  await handle.click(products);
  assert.equal(products.getAttribute("aria-expanded"), "false");
  assert.deepEqual(
    handle.wrapper.emitted("change")?.map(([value, previous, reason]) => [value, previous, reason]),
    [
      ["products", null, "toggle"],
      [null, "products", "toggle"],
    ],
  );
  handle.unmount();
});

test("hover opens after the delay, skips it between triggers, and closes after a grace period", async () => {
  const handle = mountMenu({ delayDuration: 30, skipDelayDuration: 200, closeDelay: 30 });
  const products = trigger(handle, "Products");
  const docs = trigger(handle, "Docs");
  pointer(products, "pointerenter");
  await nextTick();
  assert.equal(products.getAttribute("aria-expanded"), "false", "hover waits for the delay");
  await wait(50);
  assert.equal(products.getAttribute("aria-expanded"), "true");

  pointer(products, "pointerleave");
  pointer(docs, "pointerenter");
  await nextTick();
  assert.equal(docs.getAttribute("aria-expanded"), "true", "moving between triggers is instant");

  const docsContent = handle.root().querySelector("#site-content-value-docs");
  assert.ok(docsContent);
  pointer(docs, "pointerleave");
  pointer(docsContent, "pointerenter");
  await wait(50);
  assert.equal(docs.getAttribute("aria-expanded"), "true", "entering the flyout cancels closing");
  pointer(docsContent, "pointerleave");
  await wait(50);
  assert.equal(docs.getAttribute("aria-expanded"), "false");

  pointer(products, "pointerenter");
  await nextTick();
  assert.equal(products.getAttribute("aria-expanded"), "true", "the skip window reopens instantly");
  await handle.click(products);
  assert.equal(
    products.getAttribute("aria-expanded"),
    "true",
    "the click after hover keeps it open",
  );
  handle.unmount();
});

test("touch pointers never open on hover", async () => {
  const handle = mountMenu({ delayDuration: 0 });
  const products = trigger(handle, "Products");
  products.dispatchEvent(new PointerEvent("pointerenter", { pointerType: "touch" }));
  await wait(10);
  assert.equal(products.getAttribute("aria-expanded"), "false");
  handle.unmount();
});

test("arrow keys move between top-level entries and the open key enters the flyout", async () => {
  const handle = mountMenu();
  const products = trigger(handle, "Products");
  const docs = trigger(handle, "Docs");
  const blog = handle.getByRole("link", { name: "Blog" });
  products.focus();
  await handle.press(products, "ArrowRight");
  assert.ok(handle.activeElement() === docs);
  await handle.press(docs, "ArrowRight");
  assert.ok(handle.activeElement() === blog, "top-level links are entries too");
  await handle.press(blog, "ArrowRight");
  assert.ok(handle.activeElement() === products, "arrow navigation wraps");
  await handle.press(products, "End");
  assert.ok(handle.activeElement() === blog);
  await handle.press(blog, "Home");
  assert.ok(handle.activeElement() === products);

  const down = await handle.press(products, "ArrowDown");
  assert.equal(down.keydownPrevented, true);
  await nextTick();
  assert.equal(products.getAttribute("aria-expanded"), "true");
  assert.ok(handle.activeElement() === handle.getByRole("link", { name: "UI kit" }));
  handle.unmount();
});

test("Escape, outside pointerdown, focus leaving, and link selection dismiss the flyout", async () => {
  const handle = mountMenu();
  const products = trigger(handle, "Products");
  await handle.click(products);
  const uiKit = handle.getByRole("link", { name: "UI kit" });
  uiKit.focus();
  await handle.press(uiKit, "Escape");
  assert.equal(products.getAttribute("aria-expanded"), "false");
  assert.ok(handle.activeElement() === products, "Escape returns focus to the trigger");

  await handle.click(products);
  document.body.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true }));
  await nextTick();
  assert.equal(products.getAttribute("aria-expanded"), "false");

  await handle.click(products);
  const outside = document.createElement("button");
  document.body.append(outside);
  products.dispatchEvent(new FocusEvent("focusout", { bubbles: true, relatedTarget: outside }));
  await nextTick();
  assert.equal(products.getAttribute("aria-expanded"), "false");
  outside.remove();

  await handle.click(products);
  const link = handle.getByRole("link", { name: "CLI" });
  link.addEventListener("click", (event) => event.preventDefault(), { once: true });
  await handle.click(link);
  assert.equal(products.getAttribute("aria-expanded"), "false");
  assert.equal(handle.wrapper.emitted("change")?.at(-1)?.[2], "link");
  handle.unmount();
});

test("motion attributes follow the direction between items and forceMount keeps hidden flyouts", async () => {
  const handle = mountMenu({}, { forceMount: true });
  const products = trigger(handle, "Products");
  const docs = trigger(handle, "Docs");
  const productsContent = handle
    .root()
    .querySelector<HTMLDivElement>("#site-content-value-products");
  assert.ok(productsContent);
  assert.equal(productsContent.hidden, true);
  assert.equal(productsContent.getAttribute("data-state"), "closed");
  await handle.click(products);
  assert.equal(productsContent.hidden, false);
  await handle.click(docs);
  const docsContent = handle.root().querySelector("#site-content-value-docs");
  assert.equal(docsContent?.getAttribute("data-motion"), "from-end");
  assert.equal(productsContent.getAttribute("data-motion"), "to-start");
  await handle.click(products);
  assert.equal(productsContent.getAttribute("data-motion"), "from-start");
  handle.unmount();
});

test("controlled values wait for the parent and the root exposes open/close", async () => {
  const handle = mountMenu({ modelValue: null });
  const products = trigger(handle, "Products");
  await handle.click(products);
  assert.equal(products.getAttribute("aria-expanded"), "false");
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [["products"]]);
  await handle.wrapper.setProps({ modelValue: "docs" });
  assert.equal(trigger(handle, "Docs").getAttribute("aria-expanded"), "true");
  handle.unmount();

  const uncontrolled = mountMenu();
  const expose = uncontrolled.exposes<NavigationMenuRootExpose>();
  assert.equal(expose.open("docs"), true);
  await nextTick();
  assert.equal(expose.value, "docs");
  assert.equal(expose.close(), true);
  assert.equal(expose.close(), false);
  uncontrolled.unmount();
});

test("the indicator and viewport publish geometry for the open item", async () => {
  const handle = mountMenu();
  const docs = trigger(handle, "Docs");
  Object.defineProperty(docs, "offsetLeft", { configurable: true, value: 120 });
  Object.defineProperty(docs, "offsetWidth", { configurable: true, value: 64 });
  await handle.click(docs);
  await nextTick();
  const indicator = handle
    .root()
    .querySelector<HTMLElement>('[data-vize-ui="navigation-menu-indicator"]');
  assert.equal(indicator?.getAttribute("data-state"), "open");
  assert.equal(
    indicator?.style.getPropertyValue("--vize-navigation-menu-indicator-offset"),
    "120px",
  );
  assert.equal(indicator?.style.getPropertyValue("--vize-navigation-menu-indicator-size"), "64px");
  const viewport = handle
    .root()
    .querySelector<HTMLElement>('[data-vize-ui="navigation-menu-viewport"]');
  assert.equal(viewport?.getAttribute("data-state"), "open");
  assert.equal(viewport?.style.getPropertyValue("--vize-navigation-menu-viewport-width"), "0px");
  await handle.click(docs);
  await nextTick();
  assert.equal(indicator?.getAttribute("style"), null);
  handle.unmount();
});

test("links drop script-capable hrefs and render custom elements", () => {
  const handle = mountInteraction(NavigationMenuRoot, {
    slots: {
      default: () => [
        h(NavigationMenuLink, { href: "javascript:alert(1)" }, () => "Unsafe"),
        h(NavigationMenuLink, { as: "span" }, () => "Custom"),
      ],
    },
  });
  const links = handle.root().querySelectorAll('[data-vize-ui="navigation-menu-link"]');
  assert.equal(links[0]?.getAttribute("href"), null);
  assert.equal(links[1]?.tagName, "SPAN");
  handle.unmount();
});

test("disabled triggers ignore click, hover, and the open key", async () => {
  const handle = mountInteraction(NavigationMenuRoot, {
    props: { delayDuration: 0 },
    slots: {
      default: () =>
        h(NavigationMenuList, null, () =>
          h(NavigationMenuItem, { value: "x" }, () => [
            h(NavigationMenuTrigger, { disabled: true }, () => "X"),
            h(NavigationMenuContent, null, () => "Flyout"),
          ]),
        ),
    },
  });
  const x = handle.getByRole("button", { name: "X" });
  pointer(x, "pointerenter");
  x.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
  await wait(10);
  assert.equal(x.getAttribute("aria-expanded"), "false");
  assert.equal(x.getAttribute("data-disabled"), "true");
  handle.unmount();
});

test("compound parts require matching providers", () => {
  assert.throws(() => mountInteraction(NavigationMenuList), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(
    () => mountInteraction(NavigationMenuItem, { props: { value: "a" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
  assert.throws(
    () => mountInteraction(NavigationMenuLink, { props: { href: "/" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
  assert.throws(() => mountInteraction(NavigationMenuIndicator), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(NavigationMenuViewport), /VIZE_UI_CONTEXT_MISSING/);
});
