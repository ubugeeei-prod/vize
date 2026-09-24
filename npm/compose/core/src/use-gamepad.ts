import { computed, readonly, shallowReadonly, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import { useRafFn } from "./use-raf-fn.ts";
import type { FrameScheduler } from "./use-raf-fn.ts";

/** Button indices of the W3C "standard" gamepad mapping. */
export const standardGamepadButtons = {
  faceBottom: 0,
  faceRight: 1,
  faceLeft: 2,
  faceTop: 3,
  leftBumper: 4,
  rightBumper: 5,
  leftTrigger: 6,
  rightTrigger: 7,
  select: 8,
  start: 9,
  leftStick: 10,
  rightStick: 11,
  dpadUp: 12,
  dpadDown: 13,
  dpadLeft: 14,
  dpadRight: 15,
  home: 16,
} as const;

/** Axis indices of the W3C "standard" gamepad mapping. */
export const standardGamepadAxes = {
  leftStickX: 0,
  leftStickY: 1,
  rightStickX: 2,
  rightStickY: 3,
} as const;

/** Minimal `GamepadButton` read by {@link useGamepad}. */
export interface GamepadButtonLike {
  /** Whether the button is pressed. */
  readonly pressed: boolean;
  /** Whether the button is touched (defaults to `pressed` when absent). */
  readonly touched?: boolean;
  /** Analog value in `[0, 1]`. */
  readonly value: number;
}

/** Minimal `GamepadHapticActuator` used for vibration. */
export interface GamepadVibrationActuatorLike {
  /** Play a haptic effect. */
  playEffect(type: string, params: GamepadVibrationParams): Promise<unknown>;
}

/** Minimal live `Gamepad` object read by {@link useGamepad}. */
export interface GamepadLike {
  /** Device identifier string. */
  readonly id: string;
  /** Slot index in `navigator.getGamepads()`. */
  readonly index: number;
  /** Whether the pad is still connected. */
  readonly connected: boolean;
  /** Layout mapping (`"standard"` or `""`). */
  readonly mapping: string;
  /** Last update timestamp. */
  readonly timestamp: number;
  /** Button states. */
  readonly buttons: ArrayLike<GamepadButtonLike>;
  /** Axis values in `[-1, 1]`. */
  readonly axes: ArrayLike<number>;
  /** Haptic actuator, when the pad supports vibration. */
  readonly vibrationActuator?: GamepadVibrationActuatorLike | null;
}

/** Window-like capability used by {@link useGamepad}. */
export interface GamepadHost {
  /** Navigator exposing `getGamepads()`. */
  readonly navigator: {
    /** Read the current pads (live objects, `null` for empty slots). */
    getGamepads(): ArrayLike<GamepadLike | null>;
  };
  /** Subscribe to `gamepadconnected` / `gamepaddisconnected`. */
  addEventListener(type: string, listener: () => void): void;
  /** Unsubscribe from `gamepadconnected` / `gamepaddisconnected`. */
  removeEventListener(type: string, listener: () => void): void;
}

/** Parameters of a `dual-rumble` vibration effect. */
export interface GamepadVibrationParams {
  /** Effect duration in milliseconds. */
  readonly duration: number;
  /** Delay before the effect starts, in milliseconds. */
  readonly startDelay?: number;
  /** Low-frequency motor magnitude in `[0, 1]`. */
  readonly strongMagnitude?: number;
  /** High-frequency motor magnitude in `[0, 1]`. */
  readonly weakMagnitude?: number;
}

/** Plain, copied state of one button. */
export interface GamepadButtonState {
  /** Whether the button is pressed. */
  readonly pressed: boolean;
  /** Whether the button is touched. */
  readonly touched: boolean;
  /** Analog value in `[0, 1]`. */
  readonly value: number;
}

/** Immutable copy of a pad taken at one poll; never a live `Gamepad`. */
export interface GamepadSnapshot {
  /** Device identifier string. */
  readonly id: string;
  /** Slot index. */
  readonly index: number;
  /** Whether the pad was connected when copied. */
  readonly connected: boolean;
  /** Layout mapping (`"standard"` or `""`). */
  readonly mapping: string;
  /** Update timestamp of the copied state. */
  readonly timestamp: number;
  /** Copied button states. */
  readonly buttons: readonly GamepadButtonState[];
  /** Copied axis values. */
  readonly axes: readonly number[];
  /** Whether the pad exposes a vibration actuator. */
  readonly vibration: boolean;
}

/** Binds a mapping key to a button; the mapped value is a {@link GamepadButtonState}. */
export interface GamepadButtonBinding {
  /** Button index (see {@link standardGamepadButtons}). */
  readonly button: number;
}

/** Binds a mapping key to an axis; the mapped value is a number. */
export interface GamepadAxisBinding {
  /** Axis index (see {@link standardGamepadAxes}). */
  readonly axis: number;
  /** Values with a magnitude below this become `0`; the rest is rescaled to `[-1, 1]`. */
  readonly deadzone?: number;
  /** Negate the axis value. */
  readonly invert?: boolean;
}

/** Named bindings from which a typed per-pad state is derived. */
export type GamepadMapping = Readonly<Record<string, GamepadButtonBinding | GamepadAxisBinding>>;

/** Typed state produced by a {@link GamepadMapping}: axes map to numbers, buttons to states. */
export type MappedGamepad<Mapping extends GamepadMapping> = {
  readonly [Key in keyof Mapping]: Mapping[Key] extends { readonly axis: number }
    ? number
    : GamepadButtonState;
};

/** One connected pad: its snapshot plus the mapped state. */
export interface ConnectedGamepad<
  Mapping extends GamepadMapping = Record<never, never>,
> extends GamepadSnapshot {
  /** State derived from the `mapping` option. */
  readonly mapped: MappedGamepad<Mapping>;
}

/** Options for {@link useGamepad}. */
export interface UseGamepadOptions<Mapping extends GamepadMapping = Record<never, never>> {
  /**
   * Window-like host providing `navigator.getGamepads()` and connection events.
   *
   * @default window when `navigator.getGamepads` exists
   */
  readonly host?: MaybeRefOrGetter<GamepadHost | null | undefined>;

  /**
   * Named bindings applied to every pad; the keys and value kinds type `mapped`.
   *
   * @default {}
   */
  readonly mapping?: Mapping;

  /**
   * Frame host driving the polling loop.
   *
   * @default globalThis animation-frame functions
   */
  readonly scheduler?: FrameScheduler;

  /**
   * Start polling immediately.
   *
   * @default true
   */
  readonly immediate?: boolean;
}

/** Reactive state and actions returned by {@link useGamepad}. */
export interface GamepadControls<Mapping extends GamepadMapping = Record<never, never>> {
  /** Whether the Gamepad API is available. */
  readonly supported: ComputedRef<boolean>;
  /** Connected pads, copied on every poll that saw a change. */
  readonly gamepads: Readonly<ShallowRef<readonly ConnectedGamepad<Mapping>[]>>;
  /** Whether polling is running. */
  readonly isActive: Readonly<Ref<boolean>>;
  /** Most recent vibration failure. */
  readonly error: Readonly<ShallowRef<unknown>>;
  /** Stop polling. Idempotent. */
  readonly pause: () => void;
  /** Resume polling. Idempotent. */
  readonly resume: () => void;
  /** Read the pads once, outside the frame loop. */
  readonly refresh: () => void;
  /**
   * Play a vibration effect on the pad at `index`.
   *
   * @returns Whether the effect played; failures land in `error`.
   */
  readonly vibrate: (
    index: number,
    params: GamepadVibrationParams,
    type?: string,
  ) => Promise<boolean>;
}

function browserGamepadHost(): GamepadHost | undefined {
  if (typeof window === "undefined") return undefined;
  return typeof window.navigator.getGamepads === "function" ? window : undefined;
}

function invalidMapping(message: string): RangeError {
  return new RangeError(`[VIZE_COMPOSE_GAMEPAD_INVALID_MAPPING] ${message}`);
}

function isAxisBinding(
  binding: GamepadButtonBinding | GamepadAxisBinding,
): binding is GamepadAxisBinding {
  return "axis" in binding;
}

function checkIndex(index: number, key: string): void {
  if (!Number.isInteger(index) || index < 0) {
    throw invalidMapping(`"${key}" index must be a non-negative integer; received ${index}`);
  }
}

/**
 * Copy a live pad into an immutable {@link GamepadSnapshot}.
 *
 * @param pad Live `Gamepad` (or compatible) object.
 * @returns The copied state.
 */
export function snapshotGamepad(pad: GamepadLike): GamepadSnapshot {
  return {
    id: pad.id,
    index: pad.index,
    connected: pad.connected,
    mapping: pad.mapping,
    timestamp: pad.timestamp,
    buttons: Array.from(pad.buttons, (button) => ({
      pressed: button.pressed,
      touched: button.touched ?? button.pressed,
      value: button.value,
    })),
    axes: Array.from(pad.axes),
    vibration: pad.vibrationActuator != null,
  };
}

/**
 * Derive typed named state from a snapshot. Missing buttons read as released
 * and missing axes as `0`; axis dead zones are applied and rescaled.
 *
 * @example
 * ```ts
 * const state = mapGamepad(pad, { jump: { button: 0 }, moveX: { axis: 0, deadzone: 0.1 } });
 * state.jump.pressed; state.moveX; // boolean, number
 * ```
 *
 * @param snapshot Copied pad state.
 * @param mapping Named button and axis bindings.
 * @throws `RangeError` tagged `VIZE_COMPOSE_GAMEPAD_INVALID_MAPPING` for a
 * negative or fractional index or a dead zone outside `[0, 1)`.
 * @returns State keyed like `mapping`.
 */
export function mapGamepad<const Mapping extends GamepadMapping>(
  snapshot: Pick<GamepadSnapshot, "buttons" | "axes">,
  mapping: Mapping,
): MappedGamepad<Mapping> {
  const result: Record<string, number | GamepadButtonState> = {};
  for (const [key, binding] of Object.entries(mapping)) {
    if (isAxisBinding(binding)) {
      checkIndex(binding.axis, key);
      const deadzone = binding.deadzone ?? 0;
      if (!(deadzone >= 0 && deadzone < 1)) {
        throw invalidMapping(`"${key}" deadzone must be in [0, 1); received ${deadzone}`);
      }
      const raw = snapshot.axes[binding.axis] ?? 0;
      const magnitude = Math.abs(raw);
      const value =
        magnitude < deadzone ? 0 : (Math.sign(raw) * (magnitude - deadzone)) / (1 - deadzone);
      result[key] = (binding.invert ?? false) ? 0 - value : value;
    } else {
      checkIndex(binding.button, key);
      result[key] = snapshot.buttons[binding.button] ?? {
        pressed: false,
        touched: false,
        value: 0,
      };
    }
  }
  // The loop above assigns exactly one value of the declared kind per mapping key.
  return result as MappedGamepad<Mapping>;
}

/**
 * Track connected gamepads as immutable snapshots, polled on animation
 * frames, with optional typed named bindings.
 *
 * Pads are re-read every frame while active and on `gamepadconnected` /
 * `gamepaddisconnected`; `gamepads` is replaced only when a pad's timestamp
 * or connection changed. Entries are plain copies, so they are safe to keep
 * or serialize. The frame loop and listeners stop with the owning reactive
 * scope; outside a scope call `pause()`.
 *
 * Server rendering: no frames or listeners, `supported` is false and
 * `gamepads` is empty.
 *
 * @example
 * ```ts
 * const { gamepads } = useGamepad({ mapping: { jump: { button: 0 }, moveX: { axis: 0, deadzone: 0.1 } } });
 * const player = computed(() => gamepads.value[0]?.mapped);
 * ```
 *
 * @param options Host, mapping, and scheduler.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_GAMEPAD_INVALID_MAPPING` for an invalid mapping.
 * @returns Pad state and polling controls.
 */
export function useGamepad<const Mapping extends GamepadMapping = Record<never, never>>(
  options: UseGamepadOptions<Mapping> = {},
): GamepadControls<Mapping> {
  const gamepads = shallowRef<readonly ConnectedGamepad<Mapping>[]>([]);
  const error = shallowRef<unknown>(undefined);
  const mapping = options.mapping;
  if (mapping !== undefined) mapGamepad({ buttons: [], axes: [] }, mapping);

  const resolveHost = (): GamepadHost | undefined =>
    options.host === undefined ? browserGamepadHost() : (toValue(options.host) ?? undefined);

  const readPads = (host: GamepadHost): GamepadLike[] =>
    Array.from(host.navigator.getGamepads()).filter(
      (pad): pad is GamepadLike => pad !== null && pad.connected,
    );

  const refresh = (): void => {
    const host = resolveHost();
    const pads = host ? readPads(host) : [];
    const previous = gamepads.value;
    const unchanged =
      pads.length === previous.length &&
      pads.every(
        (pad, position) =>
          previous[position]?.index === pad.index &&
          previous[position]?.id === pad.id &&
          previous[position]?.timestamp === pad.timestamp,
      );
    if (unchanged) return;
    gamepads.value = pads.map((pad) => {
      const snapshot = snapshotGamepad(pad);
      const mapped = mapGamepad(snapshot, mapping ?? {});
      // Without a mapping `Mapping` defaults to the empty record, which `{}` satisfies.
      return { ...snapshot, mapped: mapped as MappedGamepad<Mapping> };
    });
  };

  const loop = useRafFn(refresh, {
    immediate: false,
    runOnServer: true,
    ...(options.scheduler === undefined ? {} : { scheduler: options.scheduler }),
  });
  const wanted = shallowRef(options.immediate ?? true);

  const resume = (): void => {
    wanted.value = true;
    if (resolveHost()) loop.resume();
  };
  const pause = (): void => {
    wanted.value = false;
    loop.pause();
  };

  watch(
    resolveHost,
    (host, _previous, onCleanup) => {
      if (!host) {
        loop.pause();
        gamepads.value = [];
        return;
      }
      host.addEventListener("gamepadconnected", refresh);
      host.addEventListener("gamepaddisconnected", refresh);
      onCleanup(() => {
        host.removeEventListener("gamepadconnected", refresh);
        host.removeEventListener("gamepaddisconnected", refresh);
      });
      refresh();
      if (wanted.value) loop.resume();
    },
    { immediate: true, flush: "sync" },
  );

  const vibrate = async (
    index: number,
    params: GamepadVibrationParams,
    type = "dual-rumble",
  ): Promise<boolean> => {
    const host = resolveHost();
    const actuator = host
      ? readPads(host).find((pad) => pad.index === index)?.vibrationActuator
      : undefined;
    if (!actuator) return false;
    try {
      await actuator.playEffect(type, params);
      error.value = undefined;
      return true;
    } catch (cause) {
      error.value = cause;
      return false;
    }
  };

  tryOnScopeDispose(pause);

  return {
    supported: computed(() => resolveHost() !== undefined),
    gamepads: shallowReadonly(gamepads),
    isActive: loop.isActive,
    error: readonly(error),
    pause,
    resume,
    refresh,
    vibrate,
  };
}
