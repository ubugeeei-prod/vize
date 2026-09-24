/** Compile-only assertions for the `use-usb` type contracts. */

import { useUSB } from "./use-usb.ts";
import type {
  USBDeviceInfo,
  USBDeviceLike,
  USBInTransferResultLike,
  USBOutTransferResultLike,
} from "./use-usb.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const usb = useUSB();
type _Devices = Expect<Equal<(typeof usb.devices.value)[number], USBDeviceInfo>>;
type _In = Expect<
  Equal<Awaited<ReturnType<typeof usb.transferIn>>, USBInTransferResultLike | undefined>
>;
type _Out = Expect<
  Equal<Awaited<ReturnType<typeof usb.controlTransferOut>>, USBOutTransferResultLike | undefined>
>;

declare const device: USBDeviceLike;
void usb.controlTransferIn(
  device,
  { requestType: "class", recipient: "interface", request: 1, value: 0, index: 0 },
  8,
);

void usb.controlTransferIn(
  device,
  // @ts-expect-error requestType is a closed union.
  { requestType: "custom", recipient: "interface", request: 1, value: 0, index: 0 },
  8,
);

// @ts-expect-error the device list is read-only.
usb.devices.value = [];
