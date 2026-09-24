import { computed, readonly, ref, shallowReadonly, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** GATT service or characteristic UUID: a full UUID string, an alias name, or a 16/32-bit number. */
export type BluetoothUUIDLike = string | number;

/** Scan filter for `requestDevice`. */
export interface BluetoothDeviceFilter {
  /** Services the device must advertise. */
  readonly services?: readonly BluetoothUUIDLike[];
  /** Exact device name. */
  readonly name?: string;
  /** Device name prefix. */
  readonly namePrefix?: string;
}

/** Options for `requestDevice`: explicit filters or every nearby device. */
export type BluetoothRequestOptions =
  | {
      /** Accepted device filters. */
      readonly filters: readonly BluetoothDeviceFilter[];
      /** Extra services the page may access. */
      readonly optionalServices?: readonly BluetoothUUIDLike[];
    }
  | {
      /** Offer every nearby device. */
      readonly acceptAllDevices: true;
      /** Extra services the page may access. */
      readonly optionalServices?: readonly BluetoothUUIDLike[];
    };

/** Minimal `BluetoothRemoteGATTCharacteristic` used by {@link useBluetooth}. */
export interface BluetoothCharacteristicLike extends EventTarget {
  /** Last read or notified value. */
  readonly value?: DataView | null | undefined;
  /** Read the value. */
  readValue(): Promise<DataView>;
  /** Write and wait for the device's acknowledgement. */
  writeValueWithResponse(value: BufferSource): Promise<void>;
  /** Write without acknowledgement. */
  writeValueWithoutResponse(value: BufferSource): Promise<void>;
  /** Start `characteristicvaluechanged` notifications. */
  startNotifications(): Promise<unknown>;
  /** Stop notifications. */
  stopNotifications(): Promise<unknown>;
}

/** Minimal `BluetoothRemoteGATTService`. */
export interface BluetoothServiceLike {
  /** Look up a characteristic. */
  getCharacteristic(characteristic: BluetoothUUIDLike): Promise<BluetoothCharacteristicLike>;
}

/** Minimal `BluetoothRemoteGATTServer`. */
export interface BluetoothGATTServerLike {
  /** Whether the server is connected. */
  readonly connected: boolean;
  /** Connect to the server. */
  connect(): Promise<unknown>;
  /** Disconnect from the server. */
  disconnect(): void;
  /** Look up a primary service. */
  getPrimaryService(service: BluetoothUUIDLike): Promise<BluetoothServiceLike>;
}

/** Minimal `BluetoothDevice` used by {@link useBluetooth}. */
export interface BluetoothDeviceLike extends EventTarget {
  /** Opaque device identifier. */
  readonly id: string;
  /** Advertised name. */
  readonly name?: string | undefined;
  /** GATT server, when the device supports GATT. */
  readonly gatt?: BluetoothGATTServerLike | undefined;
}

/** `navigator.bluetooth`-like capability used by {@link useBluetooth}. */
export interface BluetoothHost extends EventTarget {
  /** Whether a Bluetooth adapter is available. */
  getAvailability(): Promise<boolean>;
  /** Prompt the user for a device. */
  requestDevice(options: BluetoothRequestOptions): Promise<BluetoothDeviceLike>;
}

/** Options for {@link useBluetooth}. */
export interface UseBluetoothOptions {
  /**
   * `navigator.bluetooth`-like capability.
   *
   * @default window.navigator.bluetooth when it exists
   */
  readonly host?: MaybeRefOrGetter<BluetoothHost | null | undefined>;

  /**
   * Default options for `requestDevice()`.
   *
   * @default { acceptAllDevices: true }
   */
  readonly requestOptions?: BluetoothRequestOptions;
}

/** Stops a notification subscription created by {@link BluetoothControls.notify}. */
export type BluetoothNotificationStop = () => Promise<void>;

/** Reactive state and actions returned by {@link useBluetooth}. */
export interface BluetoothControls {
  /** Whether Web Bluetooth is available. */
  readonly supported: ComputedRef<boolean>;
  /** Whether an adapter is available, following `availabilitychanged`. */
  readonly available: Readonly<Ref<boolean>>;
  /** Most recently chosen device. */
  readonly device: Readonly<ShallowRef<BluetoothDeviceLike | undefined>>;
  /** Whether the device's GATT server is connected. */
  readonly connected: Readonly<Ref<boolean>>;
  /** Most recent failure. */
  readonly error: Readonly<ShallowRef<unknown>>;
  /**
   * Prompt the user for a device (requires user activation). Disconnects
   * the previous device.
   *
   * @returns The chosen device, or `undefined` when cancelled or failed.
   */
  readonly requestDevice: (
    options?: BluetoothRequestOptions,
  ) => Promise<BluetoothDeviceLike | undefined>;
  /**
   * Connect to the device's GATT server.
   *
   * @returns Whether the server is connected.
   */
  readonly connect: () => Promise<boolean>;
  /** Stop notifications and disconnect the GATT server. Idempotent. */
  readonly disconnect: () => void;
  /**
   * Read a characteristic.
   *
   * @returns The value, or `undefined` on failure.
   */
  readonly read: (
    service: BluetoothUUIDLike,
    characteristic: BluetoothUUIDLike,
  ) => Promise<DataView | undefined>;
  /**
   * Write a characteristic, with a response unless `withoutResponse`.
   *
   * @returns Whether the write succeeded.
   */
  readonly write: (
    service: BluetoothUUIDLike,
    characteristic: BluetoothUUIDLike,
    value: BufferSource,
    options?: { readonly withoutResponse?: boolean },
  ) => Promise<boolean>;
  /**
   * Subscribe to characteristic notifications.
   *
   * @returns A stop function, or `undefined` on failure.
   */
  readonly notify: (
    service: BluetoothUUIDLike,
    characteristic: BluetoothUUIDLike,
    listener: (value: DataView) => void,
  ) => Promise<BluetoothNotificationStop | undefined>;
}

function browserBluetoothHost(): BluetoothHost | undefined {
  if (typeof window === "undefined") return undefined;
  const { navigator } = window;
  // Web Bluetooth is not part of the DOM lib; read it through a structural guard.
  const bluetooth: unknown = "bluetooth" in navigator ? navigator.bluetooth : undefined;
  return isBluetoothHost(bluetooth) ? bluetooth : undefined;
}

function isBluetoothHost(value: unknown): value is BluetoothHost {
  return typeof value === "object" && value !== null && "requestDevice" in value;
}

function availabilityOf(event: Event): boolean | undefined {
  return "value" in event && typeof event.value === "boolean" ? event.value : undefined;
}

/**
 * Connect to Bluetooth Low Energy devices with the Web Bluetooth API.
 *
 * `requestDevice()` prompts for a device, `connect()` connects its GATT
 * server, and `read` / `write` / `notify` address characteristics by service
 * and characteristic UUID. `connected` follows `gattserverdisconnected` and
 * `available` follows `availabilitychanged`. Failures land in `error`.
 * Notifications are stopped and the server disconnected when the owning
 * reactive scope stops; outside a scope call `disconnect()`.
 *
 * Server rendering: `supported`, `available` and `connected` are false and
 * nothing is queried.
 *
 * @example
 * ```ts
 * const ble = useBluetooth({ requestOptions: { filters: [{ services: ["heart_rate"] }] } });
 * await ble.requestDevice();
 * await ble.connect();
 * await ble.notify("heart_rate", "heart_rate_measurement", (value) => show(value.getUint8(1)));
 * ```
 *
 * @param options Host and default request options.
 * @default options {}
 * @returns Bluetooth state and actions.
 */
export function useBluetooth(options: UseBluetoothOptions = {}): BluetoothControls {
  const available = ref(false);
  const device = shallowRef<BluetoothDeviceLike | undefined>(undefined);
  const connected = ref(false);
  const error = shallowRef<unknown>(undefined);
  const notifications = new Set<BluetoothNotificationStop>();

  const resolveHost = (): BluetoothHost | undefined =>
    options.host === undefined ? browserBluetoothHost() : (toValue(options.host) ?? undefined);

  const onDisconnected = (): void => {
    connected.value = false;
    for (const stop of notifications) void stop();
  };

  const disconnect = (): void => {
    for (const stop of notifications) void stop();
    notifications.clear();
    device.value?.gatt?.disconnect();
    connected.value = false;
  };

  const requestDevice = async (
    requestOptions: BluetoothRequestOptions = options.requestOptions ?? { acceptAllDevices: true },
  ): Promise<BluetoothDeviceLike | undefined> => {
    const host = resolveHost();
    if (!host) return undefined;
    try {
      const chosen = await host.requestDevice(requestOptions);
      disconnect();
      device.value?.removeEventListener("gattserverdisconnected", onDisconnected);
      chosen.addEventListener("gattserverdisconnected", onDisconnected);
      device.value = chosen;
      connected.value = chosen.gatt?.connected ?? false;
      error.value = undefined;
      return chosen;
    } catch (cause) {
      error.value = cause;
      return undefined;
    }
  };

  const connect = async (): Promise<boolean> => {
    const server = device.value?.gatt;
    if (!server) return false;
    try {
      await server.connect();
      connected.value = server.connected;
      error.value = undefined;
    } catch (cause) {
      error.value = cause;
      connected.value = false;
    }
    return connected.value;
  };

  const characteristicOf = async (
    service: BluetoothUUIDLike,
    characteristic: BluetoothUUIDLike,
  ): Promise<BluetoothCharacteristicLike> => {
    const server = device.value?.gatt;
    if (!server?.connected) {
      throw new Error(
        "[VIZE_COMPOSE_BLUETOOTH_NOT_CONNECTED] connect() to a device before accessing characteristics",
      );
    }
    return (await server.getPrimaryService(service)).getCharacteristic(characteristic);
  };

  const attempt = async <Value>(action: () => Promise<Value>): Promise<Value | undefined> => {
    try {
      const value = await action();
      error.value = undefined;
      return value;
    } catch (cause) {
      error.value = cause;
      return undefined;
    }
  };

  const read = (
    service: BluetoothUUIDLike,
    characteristic: BluetoothUUIDLike,
  ): Promise<DataView | undefined> =>
    attempt(async () => (await characteristicOf(service, characteristic)).readValue());

  const write = async (
    service: BluetoothUUIDLike,
    characteristic: BluetoothUUIDLike,
    value: BufferSource,
    writeOptions: { readonly withoutResponse?: boolean } = {},
  ): Promise<boolean> =>
    (await attempt(async () => {
      const target = await characteristicOf(service, characteristic);
      await ((writeOptions.withoutResponse ?? false)
        ? target.writeValueWithoutResponse(value)
        : target.writeValueWithResponse(value));
      return true;
    })) === true;

  const notify = (
    service: BluetoothUUIDLike,
    characteristic: BluetoothUUIDLike,
    listener: (value: DataView) => void,
  ): Promise<BluetoothNotificationStop | undefined> =>
    attempt(async () => {
      const target = await characteristicOf(service, characteristic);
      const onChange = (): void => {
        if (target.value) listener(target.value);
      };
      target.addEventListener("characteristicvaluechanged", onChange);
      try {
        await target.startNotifications();
      } catch (cause) {
        target.removeEventListener("characteristicvaluechanged", onChange);
        throw cause;
      }
      const stop: BluetoothNotificationStop = async () => {
        if (!notifications.delete(stop)) return;
        target.removeEventListener("characteristicvaluechanged", onChange);
        try {
          await target.stopNotifications();
        } catch (cause) {
          error.value = cause;
        }
      };
      notifications.add(stop);
      return stop;
    });

  watch(
    resolveHost,
    (host, _previous, onCleanup) => {
      available.value = false;
      if (!host) return;
      let active = true;
      const onAvailability = (event: Event): void => {
        const value = availabilityOf(event);
        if (value !== undefined) available.value = value;
      };
      host.addEventListener("availabilitychanged", onAvailability);
      onCleanup(() => {
        active = false;
        host.removeEventListener("availabilitychanged", onAvailability);
      });
      host.getAvailability().then(
        (value) => {
          if (active) available.value = value;
        },
        (cause: unknown) => {
          if (active) error.value = cause;
        },
      );
    },
    { immediate: true, flush: "sync" },
  );

  tryOnScopeDispose(() => {
    disconnect();
    device.value?.removeEventListener("gattserverdisconnected", onDisconnected);
  });

  return {
    supported: computed(() => resolveHost() !== undefined),
    available: readonly(available),
    device: shallowReadonly(device),
    connected: readonly(connected),
    error: readonly(error),
    requestDevice,
    connect,
    disconnect,
    read,
    write,
    notify,
  };
}
