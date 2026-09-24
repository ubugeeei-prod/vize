/** Compile-only assertions for the `use-bluetooth` type contracts. */

import { useBluetooth } from "./use-bluetooth.ts";
import type { BluetoothDeviceLike, BluetoothNotificationStop } from "./use-bluetooth.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const ble = useBluetooth({
  requestOptions: { acceptAllDevices: true, optionalServices: [0x180f] },
});
type _Read = Expect<Equal<Awaited<ReturnType<typeof ble.read>>, DataView | undefined>>;
type _Device = Expect<Equal<typeof ble.device.value, BluetoothDeviceLike | undefined>>;
type _Notify = Expect<
  Equal<Awaited<ReturnType<typeof ble.notify>>, BluetoothNotificationStop | undefined>
>;

void ble.notify("battery_service", "battery_level", (value) => {
  value satisfies DataView;
});
void ble.requestDevice({ filters: [{ namePrefix: "HR" }] });

// @ts-expect-error acceptAllDevices must be true.
void ble.requestDevice({ acceptAllDevices: false });

// @ts-expect-error filters or acceptAllDevices is required.
void ble.requestDevice({ optionalServices: ["battery_service"] });

// @ts-expect-error the connection flag is read-only.
ble.connected.value = true;
