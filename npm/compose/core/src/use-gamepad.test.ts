import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, shallowRef } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { mapGamepad, snapshotGamepad, standardGamepadButtons, useGamepad } from "./use-gamepad.ts";
import type { GamepadHost, GamepadLike, GamepadVibrationParams } from "./use-gamepad.ts";
import type { FrameScheduler } from "./use-raf-fn.ts";

class FakeWindow extends EventTarget implements GamepadHost {
  pads: (GamepadLike | null)[] = [];
  readonly navigator = { getGamepads: () => this.pads };

  connect(pad: GamepadLike): void {
    this.pads[pad.index] = pad;
    this.dispatchEvent(new Event("gamepadconnected"));
  }

  disconnect(index: number): void {
    this.pads[index] = null;
    this.dispatchEvent(new Event("gamepaddisconnected"));
  }

  listenerCount = 0;
  override addEventListener(type: string, listener: () => void): void {
    this.listenerCount += 1;
    super.addEventListener(type, listener);
  }
  override removeEventListener(type: string, listener: () => void): void {
    this.listenerCount -= 1;
    super.removeEventListener(type, listener);
  }
}

class ManualFrames implements FrameScheduler {
  callbacks = new Map<number, (timestamp: number) => void>();
  next = 0;
  now = 0;
  readonly requestAnimationFrame = (callback: (timestamp: number) => void): number => {
    this.next += 1;
    this.callbacks.set(this.next, callback);
    return this.next;
  };
  readonly cancelAnimationFrame = (handle: unknown): void => {
    if (typeof handle === "number") this.callbacks.delete(handle);
  };
  tick(): void {
    this.now += 16;
    const pending = [...this.callbacks.values()];
    this.callbacks.clear();
    for (const callback of pending) callback(this.now);
  }
}

interface MutablePad {
  id: string;
  index: number;
  connected: boolean;
  mapping: string;
  timestamp: number;
  buttons: { pressed: boolean; value: number }[];
  axes: number[];
  vibrationActuator?: {
    playEffect: (type: string, params: GamepadVibrationParams) => Promise<unknown>;
  };
}

function pad(index = 0): MutablePad {
  return {
    id: `pad-${index}`,
    index,
    connected: true,
    mapping: "standard",
    timestamp: 1,
    buttons: [
      { pressed: false, value: 0 },
      { pressed: true, value: 1 },
    ],
    axes: [0.05, -0.55],
  };
}

void test("snapshots connected pads on connection events", () => {
  const host = new FakeWindow();
  const frames = new ManualFrames();
  const { gamepads, supported } = useGamepad({ host, scheduler: frames });
  assert.equal(supported.value, true);
  assert.equal(gamepads.value.length, 0);

  const live = pad();
  host.connect(live);
  assert.equal(gamepads.value.length, 1);
  const first = gamepads.value[0];
  assert.equal(first?.id, "pad-0");
  assert.deepEqual(first?.buttons[1], { pressed: true, touched: true, value: 1 });
  live.axes[0] = 1;
  assert.equal(first?.axes[0], 0.05, "snapshots are copies, not live objects");

  host.disconnect(0);
  assert.deepEqual(gamepads.value, []);
});

void test("polls on frames and only replaces state when timestamps change", () => {
  const host = new FakeWindow();
  const frames = new ManualFrames();
  const live = pad();
  host.pads = [live];
  const { gamepads, isActive, pause, resume } = useGamepad({ host, scheduler: frames });
  assert.equal(isActive.value, true);
  const before = gamepads.value;
  frames.tick();
  assert.equal(gamepads.value, before, "unchanged timestamp keeps the same array");

  live.timestamp = 2;
  live.buttons[0] = { pressed: true, value: 1 };
  frames.tick();
  assert.equal(gamepads.value[0]?.buttons[0]?.pressed, true);

  pause();
  assert.equal(isActive.value, false);
  live.timestamp = 3;
  frames.tick();
  assert.equal(gamepads.value[0]?.timestamp, 2);
  resume();
  frames.tick();
  assert.equal(gamepads.value[0]?.timestamp, 3);
});

void test("applies the typed mapping per pad", () => {
  const host = new FakeWindow();
  host.pads = [pad()];
  const { gamepads } = useGamepad({
    host,
    scheduler: new ManualFrames(),
    mapping: {
      jump: { button: standardGamepadButtons.faceRight },
      missing: { button: 9 },
      moveX: { axis: 0, deadzone: 0.1 },
      moveY: { axis: 1, deadzone: 0.1, invert: true },
    },
  });
  const mapped = gamepads.value[0]?.mapped;
  assert.equal(mapped?.jump.pressed, true);
  assert.equal(mapped?.missing.pressed, false);
  assert.equal(mapped?.moveX, 0);
  assert.ok(Math.abs((mapped?.moveY ?? 0) - 0.5) < 1e-9);
});

void test("mapGamepad validates bindings", () => {
  const snapshot = snapshotGamepad(pad());
  assert.throws(() => mapGamepad(snapshot, { bad: { button: -1 } }), /GAMEPAD_INVALID_MAPPING/);
  assert.throws(() => mapGamepad(snapshot, { bad: { axis: 0, deadzone: 1 } }), RangeError);
  assert.throws(() => useGamepad({ host: null, mapping: { bad: { axis: 0.5 } } }), RangeError);
});

void test("vibrates through the actuator and records failures", async () => {
  const host = new FakeWindow();
  const calls: string[] = [];
  const live = pad();
  live.vibrationActuator = {
    playEffect: (type) => {
      calls.push(type);
      return calls.length > 1 ? Promise.reject(new Error("busy")) : Promise.resolve("complete");
    },
  };
  host.pads = [live];
  const controls = useGamepad({ host, scheduler: new ManualFrames() });
  assert.equal(controls.gamepads.value[0]?.vibration, true);
  assert.equal(await controls.vibrate(0, { duration: 100 }), true);
  assert.equal(await controls.vibrate(0, { duration: 100 }), false);
  assert.ok(controls.error.value instanceof Error);
  assert.equal(await controls.vibrate(3, { duration: 100 }), false);
  assert.deepEqual(calls, ["dual-rumble", "dual-rumble"]);
});

void test("follows a reactive host and cleans up with the scope", () => {
  const host = new FakeWindow();
  host.pads = [pad()];
  const frames = new ManualFrames();
  const current = shallowRef<GamepadHost | null>(null);
  const scope = effectScope();
  const controls = scope.run(() => useGamepad({ host: current, scheduler: frames }));
  assert.ok(controls);
  assert.equal(controls.supported.value, false);
  current.value = host;
  assert.equal(controls.gamepads.value.length, 1);
  assert.equal(host.listenerCount, 2);
  assert.equal(frames.callbacks.size, 1);

  scope.stop();
  assert.equal(host.listenerCount, 0);
  assert.equal(frames.callbacks.size, 0);
  assert.equal(controls.isActive.value, false);
});

void test("server rendering polls nothing", async () => {
  const state = await renderComposableOnServer(() => {
    const controls = useGamepad();
    return {
      supported: controls.supported,
      gamepads: controls.gamepads,
      isActive: controls.isActive,
    };
  });
  assert.equal(state, '{"supported":false,"gamepads":[],"isActive":false}');
});
