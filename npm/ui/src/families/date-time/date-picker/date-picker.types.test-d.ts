/** Compile-only assertions for the public DatePicker contract. */

import {
  DatePicker,
  DatePickerCalendar,
  DatePickerContent,
  DatePickerField,
  DatePickerRoot,
  DatePickerTrigger,
  type DatePickerRootExpose,
  type DatePickerSlotState,
  type PickerOpenState,
  type PlainDate,
} from "./date-picker.ts";
import { PopoverTrigger } from "../../overlays/popover/popover.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: DatePickerRootExpose;
declare const slot: DatePickerSlotState;

type _Value = Expect<Equal<typeof root.value, PlainDate | null>>;
type _OpenState = Expect<Equal<PickerOpenState, "closed" | "open">>;
type _SlotOpen = Expect<Equal<typeof slot.open, boolean>>;
type _TriggerIsPopoverTrigger = Expect<Equal<typeof DatePickerTrigger, typeof PopoverTrigger>>;

const rootProps: InstanceType<typeof DatePickerRoot>["$props"] = {
  modelValue: null,
  open: true,
  closeOnSelect: false,
  today: { year: 2026, month: 9, day: 25 },
  "onUpdate:open": (value: boolean) => value,
};
const fieldProps: InstanceType<typeof DatePickerField>["$props"] = { ariaLabel: "Departure" };
const calendarProps: InstanceType<typeof DatePickerCalendar>["$props"] = { numberOfMonths: 2 };
const contentProps: InstanceType<typeof DatePickerContent>["$props"] = { placement: "bottom-end" };

root.setOpen(true);

// @ts-expect-error open is boolean-only.
const badOpen: InstanceType<typeof DatePickerRoot>["$props"] = { open: "yes" };

void DatePicker;
void rootProps;
void fieldProps;
void calendarProps;
void contentProps;
void badOpen;
