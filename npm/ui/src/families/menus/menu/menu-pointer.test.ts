import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";

import { mountInteraction } from "../../../testing/mount.ts";

import { MenuFixture } from "./menu-fixture.ts";
import {
  createMenuMounter,
  focusedText,
  item,
  mouseClick,
  one,
  openMenus,
  pointer,
  settle,
  stubRect,
  wait,
} from "./menu-test-utils.ts";

const { cleanup, mountMenu } = createMenuMounter(mountInteraction);

afterEach(cleanup);

async function openByPointer(): Promise<void> {
  await mouseClick(one('[data-vize-ui="menu-trigger"]'));
}

test("hovering items moves the highlight; leaving an item clears it", async () => {
  mountMenu(MenuFixture);
  await openByPointer();
  pointer(item("Open"), "pointermove");
  await settle();
  assert.equal(focusedText(), "Open");
  assert.equal(item("Open").getAttribute("data-highlighted"), "true");
  pointer(item("Open"), "pointerleave");
  await settle();
  assert.equal(document.activeElement?.getAttribute("role"), "menu");
  assert.equal(item("Open").hasAttribute("data-highlighted"), false);
});

test("hovering a disabled item clears the highlight; touch pointers never highlight", async () => {
  mountMenu(MenuFixture);
  await openByPointer();
  pointer(item("Open"), "pointermove");
  pointer(item("Delete"), "pointermove");
  await settle();
  assert.equal(document.activeElement?.getAttribute("role"), "menu");
  pointer(item("Open"), "pointermove", { pointerType: "touch" });
  await settle();
  assert.equal(document.activeElement?.getAttribute("role"), "menu");
});

test("hovering a submenu trigger opens it without moving focus into it", async () => {
  mountMenu(MenuFixture);
  await openByPointer();
  pointer(item("Share"), "pointermove");
  await settle();
  assert.equal(openMenus().length, 2);
  assert.equal(focusedText(), "Share");
  pointer(item("Email"), "pointermove");
  await settle();
  assert.equal(focusedText(), "Email");
  assert.equal(item("Share").getAttribute("data-highlighted"), "true");
});

test("pointer grace keeps the submenu open along the safe triangle and closes it outside", async () => {
  mountMenu(MenuFixture);
  await openByPointer();
  pointer(item("Share"), "pointermove", { clientX: 90, clientY: 205 });
  await settle();
  const sub = one('[data-vize-ui="menu-sub-content"]');
  stubRect(sub, { x: 100, y: 200, width: 100, height: 100 });
  // Leaving the trigger toward the submenu arms the safe triangle.
  pointer(item("Share"), "pointerleave", { clientX: 95, clientY: 205 });
  pointer(item("Large"), "pointermove", { clientX: 97, clientY: 206 });
  await settle();
  assert.equal(openMenus().length, 2, "moving inside the triangle keeps the submenu");
  assert.equal(item("Large").hasAttribute("data-highlighted"), false);

  pointer(item("Large"), "pointermove", { clientX: 10, clientY: 180 });
  await settle();
  assert.equal(openMenus().length, 1, "leaving the triangle highlights the item and closes it");
  assert.equal(focusedText(), "Large");
});

test("pointer grace expires after its delay", async () => {
  mountMenu(MenuFixture);
  await openByPointer();
  pointer(item("Share"), "pointermove", { clientX: 90, clientY: 205 });
  await settle();
  const sub = one('[data-vize-ui="menu-sub-content"]');
  stubRect(sub, { x: 100, y: 200, width: 100, height: 100 });
  pointer(item("Share"), "pointerleave", { clientX: 95, clientY: 205 });
  pointer(item("Large"), "pointermove", { clientX: 97, clientY: 206 });
  await wait(350);
  pointer(item("Large"), "pointermove", { clientX: 97, clientY: 206 });
  await settle();
  assert.equal(openMenus().length, 1);
});

test("entering the submenu clears the grace; clicking outside every menu closes the tree", async () => {
  const log: string[] = [];
  mountMenu(MenuFixture, { props: { log } });
  await openByPointer();
  pointer(item("Share"), "pointermove");
  await settle();
  const sub = one('[data-vize-ui="menu-sub-content"]');
  pointer(sub, "pointerenter");
  pointer(item("Copy link"), "pointermove");
  await settle();
  assert.equal(focusedText(), "Copy link");
  pointer(document.body, "pointerdown");
  await settle();
  assert.equal(openMenus().length, 0);
  assert.deepEqual(log, ["open:true", "open:false"]);
});

test("pointer-down inside the parent menu closes only the submenu", async () => {
  mountMenu(MenuFixture);
  await openByPointer();
  pointer(item("Share"), "pointermove");
  await settle();
  pointer(item("New file"), "pointerdown");
  await settle();
  assert.equal(openMenus().length, 1);
});

test("clicking a submenu item selects it and closes every level", async () => {
  const log: string[] = [];
  mountMenu(MenuFixture, { props: { log } });
  await openByPointer();
  pointer(item("Share"), "pointermove");
  await settle();
  await mouseClick(item("Email"));
  assert.deepEqual(log, ["open:true", "select:email", "open:false"]);
  assert.equal(openMenus().length, 0);
});

test("clicking a submenu trigger with a pointer opens it", async () => {
  mountMenu(MenuFixture);
  await openByPointer();
  await mouseClick(item("Share"));
  assert.equal(openMenus().length, 2);
});
