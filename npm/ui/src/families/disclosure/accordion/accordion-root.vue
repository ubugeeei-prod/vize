<script
  setup
  lang="ts"
  generic="Value extends AccordionValue = string, Type extends AccordionType = 'single'"
>
import { computed } from "vue";
import type { ComputedRef } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import type { CollectionNavigationDirection } from "../../foundations/collection/collection.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { accordionContext } from "./accordion-context.ts";
import type {
  AccordionContextValue,
  AccordionTriggerRegistrationInput,
} from "./accordion-context.ts";
import type {
  AccordionDirection,
  AccordionHeadingLevel,
  AccordionModelValue,
  AccordionOrientation,
  AccordionRootExpose,
  AccordionSlotState,
  AccordionType,
  AccordionValue,
} from "./accordion-types.ts";
import {
  accordionOpenValues,
  accordionValuesEqual,
  getAccordionValueIdSegment,
  toAccordionModel,
} from "./accordion-value.ts";

const {
  type,
  id = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  disabled = false,
  collapsible = false,
  orientation = "vertical",
  dir = "ltr",
  loop = true,
  hiddenUntilFound = false,
  headingLevel = 3,
} = defineProps<{
  /**
   * Whether one (`"single"`) or many (`"multiple"`) items may be open. Selects
   * the model shape: `Value | null` or `readonly Value[]`.
   *
   * @default required
   */
  readonly type: Type;

  /**
   * Consumer-owned Accordion base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled open item(s). `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: AccordionModelValue<Value, Type>;

  /**
   * Initial open item(s) for uncontrolled use.
   *
   * @default undefined
   */
  readonly defaultValue?: AccordionModelValue<Value, Type>;

  /**
   * Disable user activation of every item while preserving open state.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Let users collapse the open item of a `single` accordion. `multiple`
   * accordions are always collapsible.
   *
   * @default false
   */
  readonly collapsible?: boolean;

  /**
   * Axis used by arrow-key focus navigation between triggers.
   *
   * @default "vertical"
   */
  readonly orientation?: AccordionOrientation;

  /**
   * Reading direction used for horizontal arrow-key navigation.
   *
   * @default "ltr"
   */
  readonly dir?: AccordionDirection;

  /**
   * Whether arrow-key navigation wraps at the first and last enabled trigger.
   *
   * @default true
   */
  readonly loop?: boolean;

  /**
   * Keep closed panels searchable with `hidden="until-found"`; a find-in-page
   * or fragment match expands the item through `beforematch`.
   *
   * @default false
   */
  readonly hiddenUntilFound?: boolean;

  /**
   * Default native heading level rendered by AccordionHeader.
   *
   * @default 3
   */
  readonly headingLevel?: AccordionHeadingLevel;
}>();

const emit = defineEmits<{
  /** Fired when the open item(s) request a new controlled value. */
  "update:modelValue": [value: AccordionModelValue<Value, Type>];

  /** Fired after any distinct open-item request. */
  "value-change": [
    value: AccordionModelValue<Value, Type>,
    previous: AccordionModelValue<Value, Type>,
    nativeEvent: Event | null,
  ];
}>();

defineSlots<{
  /** Compound AccordionItem children. Receives the current model and configuration. */
  default?(props: AccordionSlotState<Value, Type>): unknown;
}>();

const baseId = useDeterministicId({ id: () => id, hint: "accordion" });
const typeState = computed<AccordionType>(() => type);
const disabledState = computed(() => disabled);
const collapsibleState = computed(() => type === "multiple" || collapsible);
const orientationState = computed(() => orientation);
const dirState = computed(() => dir);
const hiddenUntilFoundState = computed(() => hiddenUntilFound);
const headingLevelState = computed(() => headingLevel);
const registry = createCollectionRegistry<AccordionValue, AccordionValue>({
  disabledBehavior: "skip",
});
const itemValues = new Map<AccordionValue, number>();
const openState = useControllableState<readonly Value[]>({
  value: () => (modelValue === undefined ? undefined : accordionOpenValues(modelValue)),
  defaultValue: () => accordionOpenValues(defaultValue),
  equals: accordionValuesEqual,
});
const openValues = computed(() => normalizeForType(openState.value.value));
const model = computed(() => toAccordionModel(type, openValues.value));
const slotState = computed<AccordionSlotState<Value, Type>>(() => ({
  collapsible: collapsibleState.value,
  disabled: disabledState.value,
  openValues: openValues.value,
  orientation: orientationState.value,
  type,
  value: model.value,
}));

function normalizeForType(values: readonly Value[]): readonly Value[] {
  return type === "single" && values.length > 1 ? values.slice(0, 1) : values;
}

/**
 * Items register through the shared context, so registration is the runtime
 * proof that a context value belongs to this root's value domain.
 */
function isRootValue(candidate: AccordionValue): candidate is Value {
  return (itemValues.get(candidate) ?? 0) > 0 || isOpen(candidate);
}

function registerItem(value: AccordionValue): () => void {
  itemValues.set(value, (itemValues.get(value) ?? 0) + 1);
  let registered = true;
  return () => {
    if (!registered) return;
    registered = false;
    const count = (itemValues.get(value) ?? 1) - 1;
    if (count > 0) itemValues.set(value, count);
    else itemValues.delete(value);
  };
}

function isOpen(value: AccordionValue): boolean {
  return openValues.value.some((candidate) => Object.is(candidate, value));
}

function isLocked(value: AccordionValue): boolean {
  return !collapsibleState.value && isOpen(value);
}

function currentOpenValues(): readonly Value[] {
  return openValues.value;
}

function commit(next: readonly Value[], nativeEvent: Event | null): boolean {
  const normalized = normalizeForType(next);
  const previous = toAccordionModel(type, currentOpenValues());
  if (!openState.set(normalized)) return false;
  const value = toAccordionModel(type, normalized);
  emit("update:modelValue", value);
  emit("value-change", value, previous, nativeEvent);
  return true;
}

function setOpenValue(value: Value, open: boolean, nativeEvent: Event | null): boolean {
  if (open === isOpen(value)) return false;
  if (open) {
    return commit(type === "multiple" ? [...currentOpenValues(), value] : [value], nativeEvent);
  }
  if (!collapsibleState.value) return false;
  return commit(
    openValues.value.filter((candidate) => !Object.is(candidate, value)),
    nativeEvent,
  );
}

function setItemOpen(
  value: AccordionValue,
  open: boolean,
  nativeEvent: Event | null = null,
): boolean {
  return isRootValue(value) ? setOpenValue(value, open, nativeEvent) : false;
}

function setValue(value: AccordionModelValue<Value, Type>, nativeEvent: Event | null = null) {
  return commit(accordionOpenValues(value), nativeEvent);
}

function expandAll(nativeEvent: Event | null = null): boolean {
  if (type !== "multiple") return false;
  const enabled = registry.items.value
    .filter((item) => !item.disabled)
    .map((item) => item.key)
    .filter(isRootValue);
  const next = [...currentOpenValues()];
  for (const value of enabled) if (!next.some((open) => Object.is(open, value))) next.push(value);
  return commit(next, nativeEvent);
}

function collapseAll(nativeEvent: Event | null = null): boolean {
  if (!collapsibleState.value) return false;
  return commit([], nativeEvent);
}

function focusTrigger(value: AccordionValue | null, options?: FocusOptions): boolean {
  const target = value ?? registry.getNavigationKey("first");
  const element = target === null ? undefined : registry.getItem(target)?.element;
  if (!(element instanceof HTMLElement)) return false;
  element.focus(options);
  return true;
}

function navigationDirection(key: string): CollectionNavigationDirection | null {
  if (key === "Home") return "first";
  if (key === "End") return "last";
  if (orientationState.value === "vertical") {
    if (key === "ArrowDown") return "next";
    if (key === "ArrowUp") return "previous";
    return null;
  }
  const forward = dirState.value === "rtl" ? "ArrowLeft" : "ArrowRight";
  const backward = dirState.value === "rtl" ? "ArrowRight" : "ArrowLeft";
  if (key === forward) return "next";
  if (key === backward) return "previous";
  return null;
}

function handleTriggerKeydown(value: AccordionValue, event: KeyboardEvent): void {
  if (event.defaultPrevented || event.altKey || event.ctrlKey || event.metaKey) return;
  if (event.isComposing) return;
  const direction = navigationDirection(event.key);
  if (direction === null) return;
  const target = registry.getNavigationKey(direction, { fromKey: value, loop });
  if (target === null || Object.is(target, value)) return;
  event.preventDefault();
  focusTrigger(target);
}

function registerTrigger(input: AccordionTriggerRegistrationInput) {
  return registry.register({
    disabled: input.disabled,
    element: input.element,
    key: input.value,
    value: input.value,
  });
}

accordionContext.provide({
  collapsible: collapsibleState,
  dir: dirState,
  disabled: disabledState,
  getItemId: (value) => deriveDeterministicId(baseId.value, getAccordionValueIdSegment(value)),
  handleTriggerKeydown,
  headingLevel: headingLevelState,
  hiddenUntilFound: hiddenUntilFoundState,
  id: baseId,
  isLocked,
  isOpen,
  orientation: orientationState,
  registerItem,
  registerTrigger,
  setItemOpen,
  type: typeState,
} satisfies AccordionContextValue);

type AccordionRootSetupExpose = Omit<
  AccordionRootExpose<Value, Type>,
  "id" | "openValues" | "type" | "value"
> & {
  readonly id: ComputedRef<string>;
  readonly type: ComputedRef<Type>;
  readonly openValues: ComputedRef<readonly Value[]>;
  readonly value: ComputedRef<AccordionModelValue<Value, Type>>;
};

const exposed = {
  collapse: (value, nativeEvent = null) => setOpenValue(value, false, nativeEvent),
  collapseAll,
  expand: (value, nativeEvent = null) => setOpenValue(value, true, nativeEvent),
  expandAll,
  focus: (value, options) => focusTrigger(value ?? null, options),
  id: baseId,
  isOpen: (value) => isOpen(value),
  openValues,
  setValue,
  toggle: (value, nativeEvent = null) => setOpenValue(value, !isOpen(value), nativeEvent),
  type: computed(() => type),
  value: model,
} satisfies AccordionRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="accordion-root"
    part="root"
    :dir
    :data-type="type"
    :data-orientation="orientation"
    :data-disabled="disabled ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
