import type { ComputedRef, MaybeRefOrGetter } from "vue";

import type {
  CollectionRegistration,
  CollectionRegistry,
} from "../../foundations/collection/collection.ts";
import type { CompositeNavigationController } from "../../foundations/composite-navigation/composite-navigation.ts";
import { createContext } from "../../foundations/context/context.ts";
import type {
  StepperDirection,
  StepperItemState,
  StepperNavigationMode,
  StepperOrientation,
  StepperRootState,
  StepperValue,
} from "./stepper-types.ts";

/** Reactive step data registered with the Stepper root collection. */
export interface StepperCollectionValue {
  readonly id: ComputedRef<string>;
  readonly triggerId: ComputedRef<string>;
  readonly contentId: ComputedRef<string>;
  readonly completed: ComputedRef<boolean>;
}

/** Registration input owned by StepperItem. */
export interface StepperItemRegistrationInput {
  readonly value: string;
  readonly id: ComputedRef<string>;
  readonly triggerId: ComputedRef<string>;
  readonly contentId: ComputedRef<string>;
  readonly completed: ComputedRef<boolean>;
  readonly element?: MaybeRefOrGetter<Element | null | undefined>;
  readonly textValue?: MaybeRefOrGetter<string | null | undefined>;
  readonly disabled?: MaybeRefOrGetter<boolean | undefined>;
  readonly order?: MaybeRefOrGetter<number | undefined>;
}

/** Shared state and actions for the Stepper compound components. */
export interface StepperContextValue {
  readonly id: ComputedRef<string>;
  readonly listId: ComputedRef<string>;
  readonly value: ComputedRef<StepperValue>;
  readonly completedValues: ComputedRef<readonly string[]>;
  readonly currentIndex: ComputedRef<number>;
  readonly disabled: ComputedRef<boolean>;
  readonly navigationMode: ComputedRef<StepperNavigationMode>;
  readonly orientation: ComputedRef<StepperOrientation>;
  readonly dir: ComputedRef<StepperDirection>;
  readonly state: ComputedRef<StepperRootState>;
  readonly registry: CollectionRegistry<string, StepperCollectionValue>;
  readonly navigation: CompositeNavigationController<string>;
  readonly focus: (options?: FocusOptions) => void;
  readonly focusValue: (value: string, options?: FocusOptions) => boolean;
  readonly getItemId: (value: string) => string;
  readonly getTriggerId: (value: string) => string;
  readonly getContentId: (value: string) => string;
  readonly getItemIndex: (value: string) => number;
  readonly getItemState: (value: string) => StepperItemState;
  readonly isCompleted: (value: string) => boolean;
  readonly isCurrent: (value: string) => boolean;
  readonly isSelectable: (value: string) => boolean;
  readonly isStepDisabled: (value: string) => boolean;
  readonly registerItem: (input: StepperItemRegistrationInput) => CollectionRegistration<string>;
  readonly selectValue: (value: string, event?: Event | null) => boolean;
  readonly setValue: (value: StepperValue, event?: Event | null) => boolean;
  readonly syncActiveValue: () => void;
}

/** Shared state owned by one StepperItem and consumed by its trigger. */
export interface StepperItemContextValue {
  readonly value: ComputedRef<string>;
  readonly index: ComputedRef<number>;
  readonly id: ComputedRef<string>;
  readonly triggerId: ComputedRef<string>;
  readonly contentId: ComputedRef<string>;
  readonly current: ComputedRef<boolean>;
  readonly completed: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly selectable: ComputedRef<boolean>;
  readonly locked: ComputedRef<boolean>;
  readonly orientation: ComputedRef<StepperOrientation>;
  readonly navigationMode: ComputedRef<StepperNavigationMode>;
  readonly state: ComputedRef<StepperItemState>;
  readonly setTriggerElement: (element: HTMLButtonElement | null) => void;
  readonly focus: (options?: FocusOptions) => boolean;
  readonly select: (event?: Event | null) => boolean;
}

export const stepperContext = createContext<StepperContextValue>("Stepper");
export const stepperItemContext = createContext<StepperItemContextValue>("StepperItem");
