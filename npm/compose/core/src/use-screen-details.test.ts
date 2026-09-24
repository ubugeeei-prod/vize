import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { placeWindow, snapshotScreen, useScreenDetails } from "./use-screen-details.ts";
import type {
  ScreenDetailedLike,
  ScreenDetailsHost,
  ScreenDetailsLike,
  ScreenSnapshot,
} from "./use-screen-details.ts";

const base: ScreenSnapshot = {
  label: "Built-in",
  left: 0,
  top: 0,
  width: 1440,
  height: 900,
  availLeft: 0,
  availTop: 25,
  availWidth: 1440,
  availHeight: 875,
  devicePixelRatio: 2,
  colorDepth: 30,
  isPrimary: true,
  isInternal: true,
};

class FakeScreen extends EventTarget implements ScreenDetailedLike {
  label: string;
  left: number;
  top: number;
  width: number;
  height: number;
  availLeft: number;
  availTop: number;
  availWidth: number;
  availHeight: number;
  devicePixelRatio: number;
  colorDepth: number;
  isPrimary: boolean;
  isInternal: boolean;

  constructor(init: ScreenSnapshot) {
    super();
    this.label = init.label;
    this.left = init.left;
    this.top = init.top;
    this.width = init.width;
    this.height = init.height;
    this.availLeft = init.availLeft;
    this.availTop = init.availTop;
    this.availWidth = init.availWidth;
    this.availHeight = init.availHeight;
    this.devicePixelRatio = init.devicePixelRatio;
    this.colorDepth = init.colorDepth;
    this.isPrimary = init.isPrimary;
    this.isInternal = init.isInternal;
  }
}

class FakeScreenDetails extends EventTarget implements ScreenDetailsLike {
  screens: FakeScreen[];
  currentScreen: FakeScreen;

  constructor(screens: FakeScreen[]) {
    super();
    this.screens = screens;
    this.currentScreen = screens[0] ?? new FakeScreen(base);
  }
}

class FakeWindowScreen extends EventTarget {
  isExtended = false;
}

const external = new FakeScreen({
  ...base,
  label: "External",
  left: 1440,
  availLeft: 1440,
  availTop: 0,
  width: 2560,
  availWidth: 2560,
  height: 1440,
  availHeight: 1400,
  devicePixelRatio: 1,
  isPrimary: false,
  isInternal: false,
});

function createHost(result: () => Promise<ScreenDetailsLike>): {
  host: ScreenDetailsHost;
  screen: FakeWindowScreen;
  opened: [string, string, string][];
} {
  const screen = new FakeWindowScreen();
  const opened: [string, string, string][] = [];
  const host: ScreenDetailsHost = {
    getScreenDetails: result,
    screen,
    open: (url, target, features) => {
      opened.push([url, target, features]);
      return null;
    },
  };
  return { host, screen, opened };
}

void test("placeWindow centers, clamps, and offsets within the available area", () => {
  assert.deepEqual(placeWindow(external), { left: 1440, top: 0, width: 2560, height: 1400 });
  assert.deepEqual(placeWindow(external, { width: 800, height: 600 }), {
    left: 2320,
    top: 400,
    width: 800,
    height: 600,
  });
  assert.deepEqual(placeWindow(base, { width: 5000, left: 10, top: 2000 }), {
    left: 0,
    top: 25,
    width: 1440,
    height: 875,
  });
  assert.throws(
    () => placeWindow(base, { width: -1 }),
    /VIZE_COMPOSE_SCREEN_DETAILS_INVALID_PLACEMENT/u,
  );
});

void test("request snapshots screens and follows change events", async () => {
  const primary = new FakeScreen(base);
  const details = new FakeScreenDetails([primary]);
  const { host } = createHost(() => Promise.resolve(details));
  const state = useScreenDetails({ host });
  assert.equal(state.supported.value, true);
  assert.equal(state.screens.value.length, 0);

  const result = await state.request();
  assert.equal(result.status, "granted");
  assert.deepEqual([...state.screens.value], [base]);
  assert.deepEqual(state.currentScreen.value, base);
  assert.equal(state.pending.value, false);

  details.screens = [primary, external];
  details.dispatchEvent(new Event("screenschange"));
  assert.deepEqual(
    state.screens.value.map((screen) => screen.label),
    ["Built-in", "External"],
  );

  details.currentScreen = external;
  details.dispatchEvent(new Event("currentscreenchange"));
  assert.equal(state.currentScreen.value?.label, "External");

  external.devicePixelRatio = 1.5;
  external.dispatchEvent(new Event("change"));
  assert.equal(state.screens.value[1]?.devicePixelRatio, 1.5);
  external.devicePixelRatio = 1;
});

void test("isExtended follows window.screen changes", () => {
  const { host, screen } = createHost(() => Promise.reject(new Error("unused")));
  const state = useScreenDetails({ host });
  assert.equal(state.isExtended.value, false);
  screen.isExtended = true;
  screen.dispatchEvent(new Event("change"));
  assert.equal(state.isExtended.value, true);
});

void test("classifies request failures", async () => {
  const denial = new DOMException("denied", "NotAllowedError");
  let next: unknown = denial;
  const { host } = createHost(() => Promise.reject(next));
  const state = useScreenDetails({ host });
  assert.deepEqual(await state.request(), { status: "denied", error: denial });
  assert.equal(state.error.value, denial);
  next = new Error("boom");
  assert.equal((await state.request()).status, "failed");
});

void test("openOnScreen passes placement features to window.open", () => {
  const { host, opened } = createHost(() => Promise.reject(new Error("unused")));
  const state = useScreenDetails({ host });
  state.openOnScreen("/slides", snapshotScreen(external), {
    width: 800,
    height: 600,
    features: "popup",
  });
  state.openOnScreen("/notes", base, { target: "notes" });
  assert.deepEqual(opened, [
    ["/slides", "_blank", "left=2320,top=400,width=800,height=600,popup"],
    ["/notes", "notes", "left=0,top=25,width=1440,height=875"],
  ]);
});

void test("scope disposal removes every listener", async () => {
  const primary = new FakeScreen(base);
  const details = new FakeScreenDetails([primary]);
  const { host, screen } = createHost(() => Promise.resolve(details));
  const scope = effectScope();
  const state = scope.run(() => useScreenDetails({ host }));
  assert.ok(state);
  await state.request();
  scope.stop();

  details.screens = [primary, external];
  details.dispatchEvent(new Event("screenschange"));
  primary.dispatchEvent(new Event("change"));
  screen.isExtended = true;
  screen.dispatchEvent(new Event("change"));
  assert.equal(state.screens.value.length, 1);
  assert.equal(state.isExtended.value, false);
});

void test("reports unsupported without a host", async () => {
  const state = useScreenDetails({ host: null });
  assert.equal(state.supported.value, false);
  assert.deepEqual(await state.request(), { status: "unsupported", error: undefined });
  assert.equal(state.openOnScreen("/x", base), null);
});

void test("server rendering requests nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const details = useScreenDetails();
    return {
      supported: details.supported,
      isExtended: details.isExtended,
      screens: details.screens,
      pending: details.pending,
    };
  });
  assert.equal(state, '{"supported":false,"isExtended":false,"screens":[],"pending":false}');
});
