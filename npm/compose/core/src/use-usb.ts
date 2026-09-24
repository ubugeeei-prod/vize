import {
  computed,
  hasInjectionContext,
  readonly,
  shallowReadonly,
  shallowRef,
  toValue,
  watch,
  watchPostEffect,
} from "vue";
import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Device filter for `requestDevice`. */
export interface USBDeviceFilter {
  /** USB vendor id. */
  readonly vendorId?: number;
  /** USB product id. */
  readonly productId?: number;
  /** Device or interface class code. */
  readonly classCode?: number;
  /** Device or interface subclass code. */
  readonly subclassCode?: number;
  /** Device or interface protocol code. */
  readonly protocolCode?: number;
  /** Serial number. */
  readonly serialNumber?: string;
}

/** Transfer status reported by WebUSB. */
export type USBTransferStatus = "ok" | "stall" | "babble";

/** Result of an IN transfer. */
export interface USBInTransferResultLike {
  /** Transfer status. */
  readonly status: USBTransferStatus;
  /** Received bytes. */
  readonly data?: DataView | null;
}

/** Result of an OUT transfer. */
export interface USBOutTransferResultLike {
  /** Transfer status. */
  readonly status: USBTransferStatus;
  /** Number of bytes written. */
  readonly bytesWritten: number;
}

/** Setup packet of a control transfer. */
export interface USBControlTransferSetup {
  /** Request category. */
  readonly requestType: "standard" | "class" | "vendor";
  /** Request target. */
  readonly recipient: "device" | "interface" | "endpoint" | "other";
  /** `bRequest`. */
  readonly request: number;
  /** `wValue`. */
  readonly value: number;
  /** `wIndex`. */
  readonly index: number;
}

/** Minimal `USBDevice` used by {@link useUSB}. */
export interface USBDeviceLike {
  /** USB vendor id. */
  readonly vendorId: number;
  /** USB product id. */
  readonly productId: number;
  /** Product name. */
  readonly productName?: string | undefined;
  /** Serial number. */
  readonly serialNumber?: string | undefined;
  /** Whether a session is open. */
  readonly opened: boolean;
  /** Start a session. */
  open(): Promise<void>;
  /** End the session. */
  close(): Promise<void>;
  /** Select a configuration by value. */
  selectConfiguration(configurationValue: number): Promise<void>;
  /** Claim an interface. */
  claimInterface(interfaceNumber: number): Promise<void>;
  /** Release a claimed interface. */
  releaseInterface(interfaceNumber: number): Promise<void>;
  /** Bulk/interrupt IN transfer. */
  transferIn(endpointNumber: number, length: number): Promise<USBInTransferResultLike>;
  /** Bulk/interrupt OUT transfer. */
  transferOut(endpointNumber: number, data: BufferSource): Promise<USBOutTransferResultLike>;
  /** Control IN transfer. */
  controlTransferIn(
    setup: USBControlTransferSetup,
    length: number,
  ): Promise<USBInTransferResultLike>;
  /** Control OUT transfer. */
  controlTransferOut(
    setup: USBControlTransferSetup,
    data?: BufferSource,
  ): Promise<USBOutTransferResultLike>;
}

/** `navigator.usb`-like capability used by {@link useUSB}. */
export interface USBHost extends EventTarget {
  /** Devices the page already has access to. */
  getDevices(): Promise<USBDeviceLike[]>;
  /** Prompt the user for a device. */
  requestDevice(options: { readonly filters: USBDeviceFilter[] }): Promise<USBDeviceLike>;
}

/** Plain, reactive-friendly description of a granted device. */
export interface USBDeviceInfo {
  /** The live device, for passing back to the actions. */
  readonly device: USBDeviceLike;
  /** USB vendor id. */
  readonly vendorId: number;
  /** USB product id. */
  readonly productId: number;
  /** Product name (empty when unknown). */
  readonly productName: string;
  /** Serial number (empty when unknown). */
  readonly serialNumber: string;
  /** Whether a session was open when the list was refreshed. */
  readonly opened: boolean;
}

/** Options for {@link useUSB}. */
export interface UseUSBOptions {
  /**
   * `navigator.usb`-like capability.
   *
   * @default window.navigator.usb when it exists
   */
  readonly host?: MaybeRefOrGetter<USBHost | null | undefined>;
}

/** Reactive state and actions returned by {@link useUSB}. */
export interface USBControls {
  /** Whether WebUSB is available. */
  readonly supported: ComputedRef<boolean>;
  /** Granted devices, refreshed on `connect` / `disconnect` and after every action. */
  readonly devices: Readonly<ShallowRef<readonly USBDeviceInfo[]>>;
  /** Most recent failure. */
  readonly error: Readonly<ShallowRef<unknown>>;
  /**
   * Prompt the user for a device (requires user activation).
   *
   * @returns The chosen device, or `undefined` when cancelled or failed.
   */
  readonly requestDevice: (filters?: USBDeviceFilter[]) => Promise<USBDeviceLike | undefined>;
  /**
   * Refresh `devices`.
   *
   * @returns The granted devices.
   */
  readonly getDevices: () => Promise<readonly USBDeviceInfo[]>;
  /**
   * Open a session.
   *
   * @returns Whether it succeeded.
   */
  readonly open: (device: USBDeviceLike) => Promise<boolean>;
  /**
   * Close a session.
   *
   * @returns Whether it succeeded.
   */
  readonly close: (device: USBDeviceLike) => Promise<boolean>;
  /**
   * Select a configuration.
   *
   * @returns Whether it succeeded.
   */
  readonly selectConfiguration: (device: USBDeviceLike, value: number) => Promise<boolean>;
  /**
   * Claim an interface.
   *
   * @returns Whether it succeeded.
   */
  readonly claimInterface: (device: USBDeviceLike, interfaceNumber: number) => Promise<boolean>;
  /**
   * Release an interface.
   *
   * @returns Whether it succeeded.
   */
  readonly releaseInterface: (device: USBDeviceLike, interfaceNumber: number) => Promise<boolean>;
  /**
   * Bulk/interrupt IN transfer.
   *
   * @returns The result, or `undefined` on failure.
   */
  readonly transferIn: (
    device: USBDeviceLike,
    endpoint: number,
    length: number,
  ) => Promise<USBInTransferResultLike | undefined>;
  /**
   * Bulk/interrupt OUT transfer.
   *
   * @returns The result, or `undefined` on failure.
   */
  readonly transferOut: (
    device: USBDeviceLike,
    endpoint: number,
    data: BufferSource,
  ) => Promise<USBOutTransferResultLike | undefined>;
  /**
   * Control IN transfer.
   *
   * @returns The result, or `undefined` on failure.
   */
  readonly controlTransferIn: (
    device: USBDeviceLike,
    setup: USBControlTransferSetup,
    length: number,
  ) => Promise<USBInTransferResultLike | undefined>;
  /**
   * Control OUT transfer.
   *
   * @returns The result, or `undefined` on failure.
   */
  readonly controlTransferOut: (
    device: USBDeviceLike,
    setup: USBControlTransferSetup,
    data?: BufferSource,
  ) => Promise<USBOutTransferResultLike | undefined>;
}

function browserUSBHost(): USBHost | undefined {
  if (typeof window === "undefined") return undefined;
  const { navigator } = window;
  // WebUSB is not part of the DOM lib; read it through a structural guard.
  const usb: unknown = "usb" in navigator ? navigator.usb : undefined;
  return isUSBHost(usb) ? usb : undefined;
}

function isUSBHost(value: unknown): value is USBHost {
  return typeof value === "object" && value !== null && "requestDevice" in value;
}

function describe(device: USBDeviceLike): USBDeviceInfo {
  return {
    device,
    vendorId: device.vendorId,
    productId: device.productId,
    productName: device.productName ?? "",
    serialNumber: device.serialNumber ?? "",
    opened: device.opened,
  };
}

/**
 * Talk to USB devices with the WebUSB API.
 *
 * `devices` lists granted devices as plain snapshots and follows the host's
 * `connect` / `disconnect` events. The device actions are thin wrappers over
 * `USBDevice` that resolve to a boolean or the transfer result and record
 * failures in `error` instead of rejecting. Host listeners are removed and
 * sessions opened through `open()` are closed when the owning reactive scope
 * stops; outside a scope call `close()` per device.
 *
 * Server rendering: `supported` is false and `devices` is empty.
 *
 * @example
 * ```ts
 * const usb = useUSB();
 * const device = await usb.requestDevice([{ vendorId: 0x2341 }]);
 * if (device && (await usb.open(device))) await usb.claimInterface(device, 0);
 * ```
 *
 * @param options Capability host.
 * @default options {}
 * @returns USB state and actions.
 */
export function useUSB(options: UseUSBOptions = {}): USBControls {
  const devices = shallowRef<readonly USBDeviceInfo[]>([]);
  const error = shallowRef<unknown>(undefined);
  const owned = new Set<USBDeviceLike>();

  // Inside a component the host resolves only after mount (a post-flush
  // job), so a hydrating client first renders the same unsupported state as
  // the server. Outside components it resolves immediately.
  const hydrated = shallowRef(!hasInjectionContext());
  if (!hydrated.value) {
    watchPostEffect(() => {
      hydrated.value = true;
    });
  }
  const resolveHost = (): USBHost | undefined =>
    !hydrated.value
      ? undefined
      : options.host === undefined
        ? browserUSBHost()
        : (toValue(options.host) ?? undefined);

  const snapshot = (): void => {
    devices.value = devices.value.map((entry) => describe(entry.device));
  };

  const attempt = async <Value>(action: () => Promise<Value>): Promise<Value | undefined> => {
    try {
      const value = await action();
      error.value = undefined;
      return value;
    } catch (cause) {
      error.value = cause;
      return undefined;
    } finally {
      snapshot();
    }
  };

  const succeeded = async (action: () => Promise<void>): Promise<boolean> =>
    (await attempt(() => action().then(() => true))) === true;

  const getDevices = async (): Promise<readonly USBDeviceInfo[]> => {
    const host = resolveHost();
    if (!host) return devices.value;
    try {
      devices.value = (await host.getDevices()).map(describe);
    } catch (cause) {
      error.value = cause;
    }
    return devices.value;
  };

  const requestDevice = async (
    filters: USBDeviceFilter[] = [],
  ): Promise<USBDeviceLike | undefined> => {
    const host = resolveHost();
    if (!host) return undefined;
    const chosen = await attempt(() => host.requestDevice({ filters }));
    if (chosen) await getDevices();
    return chosen;
  };

  const open = async (device: USBDeviceLike): Promise<boolean> => {
    const opened = await succeeded(() => device.open());
    if (opened) owned.add(device);
    return opened;
  };

  const close = (device: USBDeviceLike): Promise<boolean> => {
    owned.delete(device);
    return succeeded(() => device.close());
  };

  const onConnectionChange = (): void => {
    void getDevices();
  };

  watch(
    resolveHost,
    (host, _previous, onCleanup) => {
      if (!host) {
        devices.value = [];
        return;
      }
      host.addEventListener("connect", onConnectionChange);
      host.addEventListener("disconnect", onConnectionChange);
      onCleanup(() => {
        host.removeEventListener("connect", onConnectionChange);
        host.removeEventListener("disconnect", onConnectionChange);
      });
      void getDevices();
    },
    { immediate: true, flush: "sync" },
  );

  tryOnScopeDispose(() => {
    for (const device of owned) if (device.opened) void close(device);
  });

  return {
    supported: computed(() => resolveHost() !== undefined),
    devices: shallowReadonly(devices),
    error: readonly(error),
    requestDevice,
    getDevices,
    open,
    close,
    selectConfiguration: (device, value) => succeeded(() => device.selectConfiguration(value)),
    claimInterface: (device, number) => succeeded(() => device.claimInterface(number)),
    releaseInterface: (device, number) => succeeded(() => device.releaseInterface(number)),
    transferIn: (device, endpoint, length) => attempt(() => device.transferIn(endpoint, length)),
    transferOut: (device, endpoint, data) => attempt(() => device.transferOut(endpoint, data)),
    controlTransferIn: (device, setup, length) =>
      attempt(() => device.controlTransferIn(setup, length)),
    controlTransferOut: (device, setup, data) =>
      attempt(() =>
        data === undefined
          ? device.controlTransferOut(setup)
          : device.controlTransferOut(setup, data),
      ),
  };
}
