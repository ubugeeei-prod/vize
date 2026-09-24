/** Compile-only assertions for the `use-hid` type contracts. */

import { useHID } from "./use-hid.ts";
import type { HIDDeviceInfo, HIDDeviceLike, HIDInputReport } from "./use-hid.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const hid = useHID({
  onInputReport: (report) => {
    type _Report = Expect<Equal<typeof report, HIDInputReport>>;
    report.data satisfies DataView;
  },
});
type _Devices = Expect<Equal<(typeof hid.devices.value)[number], HIDDeviceInfo>>;
type _Feature = Expect<
  Equal<Awaited<ReturnType<typeof hid.receiveFeatureReport>>, DataView | undefined>
>;

declare const device: HIDDeviceLike;
void hid.sendReport(device, 1, Uint8Array.of(1));

// @ts-expect-error report ids are numbers.
void hid.sendReport(device, "1", Uint8Array.of(1));

// @ts-expect-error the device list is read-only.
hid.devices.value = [];
