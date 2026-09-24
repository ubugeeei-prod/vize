import type { ComputedRef, ShallowRef } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { createContext } from "../../foundations/context/context.ts";
import type {
  AccordionDirection,
  AccordionHeadingLevel,
  AccordionOrientation,
  AccordionType,
  AccordionValue,
} from "./accordion-types.ts";

/** Reactive trigger data registered with the Accordion root collection. */
export interface AccordionTriggerRegistrationInput {
  readonly value: AccordionValue;
  readonly element: Readonly<ShallowRef<HTMLButtonElement | null>>;
  readonly disabled: ComputedRef<boolean>;
}

/**
 * Shared state and actions for the Accordion compound components.
 *
 * Item values cross a provide/inject boundary, so the context is typed with the
 * widest {@link AccordionValue}; AccordionRoot narrows registered values back to
 * its own generic value type.
 */
export interface AccordionContextValue {
  readonly id: ComputedRef<string>;
  readonly type: ComputedRef<AccordionType>;
  readonly disabled: ComputedRef<boolean>;
  readonly collapsible: ComputedRef<boolean>;
  readonly orientation: ComputedRef<AccordionOrientation>;
  readonly dir: ComputedRef<AccordionDirection>;
  readonly hiddenUntilFound: ComputedRef<boolean>;
  readonly headingLevel: ComputedRef<AccordionHeadingLevel>;
  readonly getItemId: (value: AccordionValue) => string;
  readonly isOpen: (value: AccordionValue) => boolean;
  readonly isLocked: (value: AccordionValue) => boolean;
  readonly setItemOpen: (value: AccordionValue, open: boolean, event?: Event | null) => boolean;
  readonly registerItem: (value: AccordionValue) => () => void;
  readonly registerTrigger: (
    input: AccordionTriggerRegistrationInput,
  ) => CollectionRegistration<AccordionValue>;
  readonly handleTriggerKeydown: (value: AccordionValue, event: KeyboardEvent) => void;
}

export const accordionContext = createContext<AccordionContextValue>("Accordion");

/** Per-item state shared by AccordionHeader, AccordionTrigger, and AccordionContent. */
export interface AccordionItemContextValue {
  readonly value: ComputedRef<AccordionValue>;
  readonly locked: ComputedRef<boolean>;
}

export const accordionItemContext = createContext<AccordionItemContextValue>("AccordionItem");
