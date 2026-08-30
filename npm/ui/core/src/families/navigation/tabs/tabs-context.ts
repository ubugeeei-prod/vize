import type { ComputedRef, ShallowRef } from "vue";

import type { CollectionRegistration, CollectionRegistry } from "../../../collection.ts";
import type { CompositeNavigationController } from "../../../composite-navigation.ts";
import { createContext } from "../../../context.ts";
import type {
  TabsActivationMode,
  TabsDirection,
  TabsOrientation,
  TabsState,
  TabsTriggerState,
  TabsValue,
} from "./tabs-types.ts";

/** Reactive trigger data registered with the Tabs root collection. */
export interface TabsTriggerRegistrationInput {
  readonly value: string;
  readonly element: Readonly<ShallowRef<HTMLButtonElement | null>>;
  readonly disabled: ComputedRef<boolean>;
  readonly textValue: () => string | null | undefined;
  readonly order: () => number | undefined;
}

/** Shared state and actions for the Tabs compound components. */
export interface TabsContextValue {
  readonly id: ComputedRef<string>;
  readonly listId: ComputedRef<string>;
  readonly value: ComputedRef<TabsValue>;
  readonly disabled: ComputedRef<boolean>;
  readonly activationMode: ComputedRef<TabsActivationMode>;
  readonly orientation: ComputedRef<TabsOrientation>;
  readonly dir: ComputedRef<TabsDirection>;
  readonly state: ComputedRef<TabsState>;
  readonly registry: CollectionRegistry<string, string>;
  readonly navigation: CompositeNavigationController<string>;
  readonly focus: (options?: FocusOptions) => void;
  readonly getTriggerId: (value: string) => string;
  readonly getContentId: (value: string) => string;
  readonly getTriggerState: (value: string, disabled: boolean) => TabsTriggerState;
  readonly isSelected: (value: string) => boolean;
  readonly registerTrigger: (input: TabsTriggerRegistrationInput) => CollectionRegistration<string>;
  readonly selectValue: (value: string, event?: Event | null) => boolean;
  readonly setValue: (value: TabsValue, event?: Event | null) => boolean;
  readonly syncActiveValue: () => void;
}

export const tabsContext = createContext<TabsContextValue>("Tabs");
