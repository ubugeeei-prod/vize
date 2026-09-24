/** Compile-only assertions for the public Editable contract. */

import {
  Editable,
  EditableInput,
  EditablePreview,
  EditableTrigger,
  type EditableActivationMode,
  type EditableExpose,
  type EditableSlotState,
  type EditableState,
  type EditableSubmitMode,
  type EditableTriggerAction,
} from "./editable.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const control: EditableExpose;
declare const slot: EditableSlotState;

type _ActivationIsClosed = Expect<
  Equal<EditableActivationMode, "click" | "dblclick" | "focus" | "none">
>;
type _SubmitIsClosed = Expect<Equal<EditableSubmitMode, "blur" | "both" | "enter" | "none">>;
type _ActionIsClosed = Expect<Equal<EditableTriggerAction, "cancel" | "edit" | "submit">>;
type _StateIsClosed = Expect<Equal<EditableState, "disabled" | "editing" | "preview" | "readonly">>;
type _ExposeSubmit = Expect<Equal<ReturnType<typeof control.submit>, boolean>>;
type _SlotDraft = Expect<Equal<typeof slot.draft, string>>;

const rootProps: InstanceType<typeof Editable>["$props"] = {
  activationMode: "dblclick",
  defaultValue: "Title",
  submitMode: "enter",
  onSubmit: (value: string, previous: string) => {
    void value;
    void previous;
  },
  onCancel: (value: string, discarded: string) => {
    void value;
    void discarded;
  },
  "onUpdate:editing": (editing: boolean) => editing,
};
const triggerProps: InstanceType<typeof EditableTrigger>["$props"] = { action: "submit" };
const previewProps: InstanceType<typeof EditablePreview>["$props"] = {};
const inputProps: InstanceType<typeof EditableInput>["$props"] = {};

control.edit();
control.cancel();

// @ts-expect-error activation modes are closed.
const invalidActivation: InstanceType<typeof Editable>["$props"] = { activationMode: "hover" };

// @ts-expect-error triggers need an action.
const missingAction: InstanceType<typeof EditableTrigger>["$props"] = {};

// @ts-expect-error values are strings.
control.setValue(1);

void inputProps;
void invalidActivation;
void missingAction;
void previewProps;
void rootProps;
void triggerProps;
