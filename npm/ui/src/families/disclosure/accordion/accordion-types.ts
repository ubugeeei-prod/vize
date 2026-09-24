/** Serializable item identity accepted by Accordion items. */
export type AccordionValue = string | number;

/** Whether one or many Accordion items may be expanded at once. */
export type AccordionType = "multiple" | "single";

/** Layout axis used by Accordion arrow-key focus navigation. */
export type AccordionOrientation = "horizontal" | "vertical";

/** Reading direction used for horizontal Accordion navigation. */
export type AccordionDirection = "ltr" | "rtl";

/** Open state mirrored to Accordion item data contracts. */
export type AccordionItemState = "closed" | "open";

/** Native heading level rendered by AccordionHeader. */
export type AccordionHeadingLevel = 1 | 2 | 3 | 4 | 5 | 6;

/** Landmark role used by AccordionContent. */
export type AccordionContentRole = "group" | "region";

/**
 * Model shape for each Accordion {@link AccordionType}.
 *
 * `single` accordions use the open item value or `null`; `multiple` accordions
 * use a readonly list of open item values.
 */
export interface AccordionModelMap<Value extends AccordionValue> {
  /** At most one open item, or `null` when every item is collapsed. */
  readonly single: Value | null;

  /** Every open item in the order it was opened. */
  readonly multiple: readonly Value[];
}

/** Model value accepted and emitted by an Accordion of the given type. */
export type AccordionModelValue<
  Value extends AccordionValue,
  Type extends AccordionType,
> = Type extends "multiple" ? readonly Value[] : Value | null;

/** State exposed to AccordionRoot slots. */
export interface AccordionSlotState<Value extends AccordionValue, Type extends AccordionType> {
  /** Selection mode. */
  readonly type: Type;

  /** Current model value in the shape selected by `type`. */
  readonly value: AccordionModelValue<Value, Type>;

  /** Every open item value, regardless of `type`. */
  readonly openValues: readonly Value[];

  /** Whether every item ignores user activation. */
  readonly disabled: boolean;

  /** Whether a `single` accordion lets users collapse the open item. */
  readonly collapsible: boolean;

  /** Arrow-key navigation axis. */
  readonly orientation: AccordionOrientation;
}

/** State exposed to AccordionItem, AccordionHeader, AccordionTrigger, and AccordionContent slots. */
export interface AccordionItemSlotState {
  /** Item identity. */
  readonly value: AccordionValue;

  /** Whether the item panel is expanded. */
  readonly open: boolean;

  /** Whether user activation is disabled for this item. */
  readonly disabled: boolean;

  /**
   * Whether the trigger is the open item of a non-collapsible `single`
   * accordion and therefore reports `aria-disabled="true"`.
   */
  readonly locked: boolean;

  /** Stable state token for styling and tests. */
  readonly state: AccordionItemState;
}

/** Public instance exposed by AccordionRoot. */
export interface AccordionRootExpose<Value extends AccordionValue, Type extends AccordionType> {
  /** Root-owned base id for the Accordion family. */
  readonly id: string;

  /** Selection mode. */
  readonly type: Type;

  /** Current model value in the shape selected by `type`. */
  readonly value: AccordionModelValue<Value, Type>;

  /** Every open item value, regardless of `type`. */
  readonly openValues: readonly Value[];

  /** Whether the item with `value` is expanded. */
  readonly isOpen: (value: Value) => boolean;

  /** Replace the model value, bypassing the `collapsible` guard. Returns whether it changed. */
  readonly setValue: (value: AccordionModelValue<Value, Type>, event?: Event | null) => boolean;

  /** Expand one item. In `single` mode this collapses the previously open item. */
  readonly expand: (value: Value, event?: Event | null) => boolean;

  /** Collapse one item, respecting `collapsible` in `single` mode. */
  readonly collapse: (value: Value, event?: Event | null) => boolean;

  /** Toggle one item, respecting `collapsible` in `single` mode. */
  readonly toggle: (value: Value, event?: Event | null) => boolean;

  /** Expand every registered enabled item. `single` accordions return `false`. */
  readonly expandAll: (event?: Event | null) => boolean;

  /** Collapse every item, respecting `collapsible` in `single` mode. */
  readonly collapseAll: (event?: Event | null) => boolean;

  /** Focus the trigger for `value`, or the first enabled trigger. */
  readonly focus: (value?: Value, options?: FocusOptions) => boolean;
}

/** Public instance exposed by AccordionItem. */
export interface AccordionItemExpose extends AccordionItemSlotState {
  /** Id wired to the item trigger. */
  readonly triggerId: string;

  /** Id wired to the item content. */
  readonly contentId: string;

  /** Rendered item element. */
  readonly element: HTMLDivElement | null;

  /** Request expansion. */
  readonly expand: (event?: Event | null) => boolean;

  /** Request collapse, respecting `collapsible` in `single` mode. */
  readonly collapse: (event?: Event | null) => boolean;

  /** Request the opposite open state. */
  readonly toggle: (event?: Event | null) => boolean;
}

/** Public instance exposed by AccordionHeader. */
export interface AccordionHeaderExpose {
  /** Rendered heading element or component instance. */
  readonly element: Element | null;
}

/** Public instance exposed by AccordionTrigger. */
export interface AccordionTriggerExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Move focus to the trigger. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by AccordionContent. */
export interface AccordionContentExpose extends AccordionItemSlotState {
  /** Rendered content element. */
  readonly element: HTMLDivElement | null;

  /** Whether closed content stays searchable through `hidden="until-found"`. */
  readonly hiddenUntilFound: boolean;
}
