import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { useScreenOrientation } from "./screen-orientation.ts";
import type { ScreenOrientationLike, ScreenOrientationLockType } from "./screen-orientation.ts";

class FakeOrientation extends EventTarget implements ScreenOrientationLike {
  type: OrientationType = "landscape-primary";
  angle = 90;
  locked: ScreenOrientationLockType | null = null;
  refuse = false;

  async lock(orientation: ScreenOrientationLockType): Promise<void> {
    if (this.refuse) throw new Error("not fullscreen");
    this.locked = orientation;
  }

  unlock(): void {
    this.locked = null;
  }
}

void test("server renders expose the configured orientation", async () => {
  const screen = useScreenOrientation({
    host: () => undefined,
    ssrOrientation: "landscape-primary",
  });
  assert.deepEqual(
    [screen.isSupported.value, screen.orientation.value, screen.angle.value],
    [false, "landscape-primary", 0],
  );
  assert.equal(await screen.lockOrientation("portrait"), false);
  assert.equal(screen.unlockOrientation(), false);
});

void test("tracks changes and locks without throwing", async () => {
  const orientation = new FakeOrientation();
  const scope = effectScope();
  const screen = scope.run(() => useScreenOrientation({ host: { screen: { orientation } } }));
  assert.ok(screen);
  assert.deepEqual([screen.orientation.value, screen.angle.value], ["landscape-primary", 90]);

  orientation.type = "portrait-primary";
  orientation.angle = 0;
  orientation.dispatchEvent(new Event("change"));
  assert.equal(screen.orientation.value, "portrait-primary");

  assert.equal(await screen.lockOrientation("portrait"), true);
  assert.equal(orientation.locked, "portrait");
  assert.equal(screen.unlockOrientation(), true);
  orientation.refuse = true;
  assert.equal(await screen.lockOrientation("landscape"), false);

  scope.stop();
  orientation.type = "landscape-secondary";
  orientation.dispatchEvent(new Event("change"));
  assert.equal(screen.orientation.value, "portrait-primary");
});
