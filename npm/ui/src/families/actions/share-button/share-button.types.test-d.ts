/** Compile-only assertions for the public ShareButton contract. */

import type { Component } from "vue";

import { ShareButton } from "./share-button.ts";
import type {
  ShareButtonAction,
  ShareButtonCssCustomProperty,
  ShareButtonDataAttribute,
  ShareButtonDataName,
  ShareButtonElement,
  ShareButtonEmits,
  ShareButtonExpose,
  ShareButtonPart,
  ShareButtonPayload,
  ShareButtonProps,
  ShareButtonSlotState,
  ShareButtonSlots,
  ShareButtonState,
  ShareButtonType,
} from "./share-button.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const file: File;
declare const exposed: ShareButtonExpose;
declare const slotState: ShareButtonSlotState;

type ComponentShareButtonProps = InstanceType<typeof ShareButton>["$props"];

type _TypeIsLiteral = Expect<Equal<ShareButtonType, "button" | "reset" | "submit">>;
type _StateIsLiteral = Expect<Equal<ShareButtonState, "idle" | "sharing" | "shared" | "error">>;
type _PartNamesAreClosed = Expect<Equal<ShareButtonPart, "label" | "root">>;
type _DataNamesAreClosed = Expect<
  Equal<ShareButtonDataName, "share-button" | "share-button-label">
>;
type _DataAttributesAreClosed = Expect<
  Equal<ShareButtonDataAttribute, "data-disabled" | "data-sharing" | "data-state" | "data-vize-ui">
>;
type _CssCustomPropertiesAreClosed = Expect<Equal<ShareButtonCssCustomProperty, never>>;
type _PayloadKeysAreClosed = Expect<
  Equal<keyof ShareButtonPayload, "files" | "text" | "title" | "url">
>;
type _PropsKeysAreClosed = Expect<
  Equal<
    keyof ShareButtonProps,
    | "action"
    | "ariaDescribedby"
    | "ariaLabel"
    | "ariaLabelledby"
    | "as"
    | "disabled"
    | "errorLabel"
    | "files"
    | "idleLabel"
    | "native"
    | "sharedLabel"
    | "sharingLabel"
    | "text"
    | "title"
    | "type"
    | "url"
  >
>;
type _EmitsAreClosed = Expect<Equal<keyof ShareButtonEmits, "share" | "error">>;
type _SlotReceivesState = Expect<
  Equal<Parameters<ShareButtonSlots["default"]>[0], ShareButtonSlotState>
>;
type _SlotStateIsExact = Expect<
  Equal<
    typeof slotState,
    {
      readonly payload: ShareButtonPayload;
      readonly disabled: boolean;
      readonly sharing: boolean;
      readonly unavailable: boolean;
      readonly state: ShareButtonState;
      readonly label: string;
    }
  >
>;
type _ExposeStateIsLiteral = Expect<Equal<typeof exposed.state, ShareButtonState>>;
type _ExposeElementIsNullable = Expect<Equal<typeof exposed.element, ShareButtonElement | null>>;
type _ActionIsNarrow = Expect<
  Equal<
    ShareButtonAction,
    (payload: ShareButtonPayload, nativeEvent: MouseEvent) => void | Promise<void>
  >
>;
type _PropsFeedComponentProps = Expect<
  ShareButtonProps extends InstanceType<typeof ShareButton>["$props"] ? true : false
>;
type _PropsAllowNoPayload = Expect<Equal<{} extends ShareButtonProps ? true : false, true>>;

const action: ShareButtonAction = async (payload, event) => {
  const title: string | undefined = payload.title;
  const text: string | undefined = payload.text;
  const url: string | undefined = payload.url;
  const files: File[] | undefined = payload.files;
  const nativeEvent: MouseEvent = event;
  void title;
  void text;
  void url;
  void files;
  void nativeEvent;
};
const payload = {
  files: [file],
  text: "Read the docs",
  title: "Vize docs",
  url: "https://vize.dev/docs",
} satisfies ShareButtonPayload;
const props = {
  action,
  ariaDescribedby: "share-help",
  ariaLabel: "Share docs",
  ariaLabelledby: "share-label",
  as: componentTarget,
  disabled: false,
  errorLabel: "Share failed",
  files: [file],
  idleLabel: "Share",
  native: false,
  sharedLabel: "Shared",
  sharingLabel: "Sharing",
  text: "Read the docs",
  title: "Vize docs",
  type: "button",
  url: "https://vize.dev/docs",
} satisfies ShareButtonProps;
const componentProps: ComponentShareButtonProps = {
  ...props,
  onShare: (nextPayload: ShareButtonPayload, event: MouseEvent) => {
    void nextPayload;
    void event;
  },
  onError: (error: unknown, nextPayload: ShareButtonPayload, event: MouseEvent) => {
    void error;
    void nextPayload;
    void event;
  },
};

const element: ShareButtonExpose["element"] = exposed.element;
const state: ShareButtonState = exposed.state;
const label: string = exposed.label;
const sharing: boolean = exposed.sharing;
const unavailable: boolean = exposed.unavailable;
const exposedPayload: ShareButtonPayload = exposed.payload;

exposed.focus();

// @ts-expect-error native button type is limited to platform submit modes.
const badType: ShareButtonProps = { type: "menu" };

// @ts-expect-error state is a closed lifecycle contract.
const badState: ShareButtonState = "loading";

// @ts-expect-error share payload keys are closed to Web Share fields.
const extraPayload = { title: "Docs", value: "not allowed" } satisfies ShareButtonPayload;

// @ts-expect-error files must be File instances.
const badFiles: ShareButtonPayload = { files: ["report.txt"] };

// @ts-expect-error action receives the normalized payload first.
const badAction: ShareButtonAction = (event: KeyboardEvent) => {
  void event;
};

void ShareButton;
void badAction;
void badFiles;
void badState;
void badType;
void componentProps;
void element;
void exposedPayload;
void extraPayload;
void label;
void payload;
void props;
void sharing;
void state;
void unavailable;
