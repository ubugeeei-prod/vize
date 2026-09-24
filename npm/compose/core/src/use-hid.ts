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
export interface HIDDeviceFilter {
  /** USB vendor id. */
  readonly vendorId?: number;
  /** USB product id. */
  readonly productId?: number;
  /** HID usage page. */
  readonly usagePage?: number;
  /** HID usage. */
  readonly usage?: number;
}

/** Minimal `HIDDevice` used by {@link useHID}. */
export interface HIDDeviceLike extends EventTarget {
  /** Whether the device is open. */
  readonly opened: boolean;
  /** USB vendor id. */
  readonly vendorId: number;
  /** USB product id. */
  readonly productId: number;
  /** Product name. */
  readonly productName: string;
  /** Open the device. */
  open(): Promise<void>;
  /** Close the device. */
  close(): Promise<void>;
  /** Send an output report. */
  sendReport(reportId: number, data: BufferSource): Promise<void>;
  /** Send a feature report. */
  sendFeatureReport(reportId: number, data: BufferSource): Promise<void>;
  /** Read a feature report. */
  receiveFeatureReport(reportId: number): Promise<DataView>;
}

/** `navigator.hid`-like capability used by {@link useHID}. */
export interface HIDHost extends EventTarget {
  /** Devices the page already has access to. */
  getDevices(): Promise<HIDDeviceLike[]>;
  /** Prompt the user for devices. */
  requestDevice(options: { readonly filters: HIDDeviceFilter[] }): Promise<HIDDeviceLike[]>;
}

/** Plain, reactive-friendly description of a granted device. */
export interface HIDDeviceInfo {
  /** The live device, for passing back to the actions. */
  readonly device: HIDDeviceLike;
  /** USB vendor id. */
  readonly vendorId: number;
  /** USB product id. */
  readonly productId: number;
  /** Product name. */
  readonly productName: string;
  /** Whether the device was open when the list was refreshed. */
  readonly opened: boolean;
}

/** An input report received from an open device. */
export interface HIDInputReport {
  /** Device that sent the report. */
  readonly device: HIDDeviceLike;
  /** Report id (`0` when the device does not use report ids). */
  readonly reportId: number;
  /** Report payload. */
  readonly data: DataView;
}

/** Options for {@link useHID}. */
export interface UseHIDOptions {
  /**
   * `navigator.hid`-like capability.
   *
   * @default window.navigator.hid when it exists
   */
  readonly host?: MaybeRefOrGetter<HIDHost | null | undefined>;

  /**
   * Receives input reports from every device opened through {@link HIDControls.open}.
   *
   * @default undefined
   */
  readonly onInputReport?: (report: HIDInputReport) => void;
}

/** Reactive state and actions returned by {@link useHID}. */
export interface HIDControls {
  /** Whether WebHID is available. */
  readonly supported: ComputedRef<boolean>;
  /** Granted devices, refreshed on `connect` / `disconnect`, open and close. */
  readonly devices: Readonly<ShallowRef<readonly HIDDeviceInfo[]>>;
  /** Most recent failure. */
  readonly error: Readonly<ShallowRef<unknown>>;
  /**
   * Prompt the user for devices (requires user activation).
   *
   * @returns The chosen devices (empty when cancelled or failed).
   */
  readonly requestDevice: (filters?: HIDDeviceFilter[]) => Promise<readonly HIDDeviceLike[]>;
  /**
   * Refresh `devices`.
   *
   * @returns The granted devices.
   */
  readonly getDevices: () => Promise<readonly HIDDeviceInfo[]>;
  /**
   * Open a device and start forwarding its input reports.
   *
   * @returns Whether the device is open.
   */
  readonly open: (device: HIDDeviceLike) => Promise<boolean>;
  /**
   * Stop forwarding reports and close a device.
   *
   * @returns Whether the device closed.
   */
  readonly close: (device: HIDDeviceLike) => Promise<boolean>;
  /**
   * Send an output report.
   *
   * @returns Whether the report was sent.
   */
  readonly sendReport: (
    device: HIDDeviceLike,
    reportId: number,
    data: BufferSource,
  ) => Promise<boolean>;
  /**
   * Send a feature report.
   *
   * @returns Whether the report was sent.
   */
  readonly sendFeatureReport: (
    device: HIDDeviceLike,
    reportId: number,
    data: BufferSource,
  ) => Promise<boolean>;
  /**
   * Read a feature report.
   *
   * @returns The report, or `undefined` on failure.
   */
  readonly receiveFeatureReport: (
    device: HIDDeviceLike,
    reportId: number,
  ) => Promise<DataView | undefined>;
}

function browserHIDHost(): HIDHost | undefined {
  if (typeof window === "undefined") return undefined;
  const { navigator } = window;
  // WebHID is not part of the DOM lib; read it through a structural guard.
  const hid: unknown = "hid" in navigator ? navigator.hid : undefined;
  return isHIDHost(hid) ? hid : undefined;
}

function isHIDHost(value: unknown): value is HIDHost {
  return typeof value === "object" && value !== null && "getDevices" in value;
}

function isInputReport(event: Event): event is Event & {
  readonly reportId: number;
  readonly data: DataView;
} {
  return "reportId" in event && "data" in event;
}

function describe(device: HIDDeviceLike): HIDDeviceInfo {
  return {
    device,
    vendorId: device.vendorId,
    productId: device.productId,
    productName: device.productName,
    opened: device.opened,
  };
}

/**
 * Talk to human-interface devices with the WebHID API.
 *
 * `devices` lists granted devices as plain snapshots and follows the host's
 * `connect` / `disconnect` events. Devices opened through `open()` forward
 * `inputreport` events to `onInputReport`. Every action resolves to a
 * boolean or value and records failures in `error`. Report listeners are
 * removed and opened devices closed when the owning reactive scope stops;
 * outside a scope call `close()` per device.
 *
 * Server rendering: `supported` is false and `devices` is empty.
 * Inside a component the host is resolved after mounting, so hydration
 * renders this server state first.
 *
 * @example
 * ```ts
 * const hid = useHID({ onInputReport: ({ reportId, data }) => read(reportId, data) });
 * const [device] = await hid.requestDevice([{ vendorId: 0x054c }]);
 * if (device) await hid.open(device);
 * ```
 *
 * @param options Host and input report callback.
 * @default options {}
 * @returns HID state and actions.
 */
export function useHID(options: UseHIDOptions = {}): HIDControls {
  const devices = shallowRef<readonly HIDDeviceInfo[]>([]);
  const error = shallowRef<unknown>(undefined);
  const reportListeners = new Map<HIDDeviceLike, (event: Event) => void>();
  const owned = new Set<HIDDeviceLike>();

  // Inside a component the host is resolved only after mounting, so a
  // hydrating client renders the server's unsupported state first. Outside
  // components it resolves synchronously.
  const hydrated = shallowRef(!hasInjectionContext());
  if (!hydrated.value) {
    watchPostEffect(() => {
      hydrated.value = true;
    });
  }

  const resolveHost = (): HIDHost | undefined => {
    if (!hydrated.value) return undefined;
    return options.host === undefined ? browserHIDHost() : (toValue(options.host) ?? undefined);
  };

  const attempt = async <Value>(
    action: () => Promise<Value>,
    clearError = true,
  ): Promise<Value | undefined> => {
    try {
      const value = await action();
      if (clearError) error.value = undefined;
      return value;
    } catch (cause) {
      error.value = cause;
      return undefined;
    }
  };

  const getDevices = async (): Promise<readonly HIDDeviceInfo[]> => {
    const host = resolveHost();
    const granted = host ? await attempt(() => host.getDevices(), false) : [];
    if (granted) devices.value = granted.map(describe);
    return devices.value;
  };

  const requestDevice = async (
    filters: HIDDeviceFilter[] = [],
  ): Promise<readonly HIDDeviceLike[]> => {
    const host = resolveHost();
    if (!host) return [];
    const chosen = await attempt(() => host.requestDevice({ filters }));
    await getDevices();
    return chosen ?? [];
  };

  const unlisten = (device: HIDDeviceLike): void => {
    const listener = reportListeners.get(device);
    if (!listener) return;
    device.removeEventListener("inputreport", listener);
    reportListeners.delete(device);
  };

  const open = async (device: HIDDeviceLike): Promise<boolean> => {
    if (!device.opened && (await attempt(() => device.open().then(() => true))) !== true) {
      return false;
    }
    owned.add(device);
    const onInputReport = options.onInputReport;
    if (onInputReport && !reportListeners.has(device)) {
      const listener = (event: Event): void => {
        if (isInputReport(event)) {
          onInputReport({ device, reportId: event.reportId, data: event.data });
        }
      };
      reportListeners.set(device, listener);
      device.addEventListener("inputreport", listener);
    }
    devices.value = devices.value.map((entry) => describe(entry.device));
    return true;
  };

  const close = async (device: HIDDeviceLike): Promise<boolean> => {
    unlisten(device);
    owned.delete(device);
    const closed =
      !device.opened || (await attempt(() => device.close().then(() => true))) === true;
    devices.value = devices.value.map((entry) => describe(entry.device));
    return closed;
  };

  const sendReport = async (
    device: HIDDeviceLike,
    reportId: number,
    data: BufferSource,
  ): Promise<boolean> =>
    (await attempt(() => device.sendReport(reportId, data).then(() => true))) === true;

  const sendFeatureReport = async (
    device: HIDDeviceLike,
    reportId: number,
    data: BufferSource,
  ): Promise<boolean> =>
    (await attempt(() => device.sendFeatureReport(reportId, data).then(() => true))) === true;

  const receiveFeatureReport = (
    device: HIDDeviceLike,
    reportId: number,
  ): Promise<DataView | undefined> => attempt(() => device.receiveFeatureReport(reportId));

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
    for (const device of owned) void close(device);
  });

  return {
    supported: computed(() => resolveHost() !== undefined),
    devices: shallowReadonly(devices),
    error: readonly(error),
    requestDevice,
    getDevices,
    open,
    close,
    sendReport,
    sendFeatureReport,
    receiveFeatureReport,
  };
}
