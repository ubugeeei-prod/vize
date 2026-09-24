import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";

/** Type-erased wizard state shared with parts (step ids are plain strings here). */
export interface FormWizardContextValue {
  readonly baseId: ComputedRef<string>;
  readonly current: ComputedRef<string>;
  readonly index: ComputedRef<number>;
  readonly count: ComputedRef<number>;
  readonly progress: ComputedRef<number>;
  readonly isFirst: ComputedRef<boolean>;
  readonly isLast: ComputedRef<boolean>;
  readonly validating: ComputedRef<boolean>;
  readonly indexOf: (step: string) => number;
  readonly isVisited: (step: string) => boolean;
  readonly registerPanel: (step: string, element: HTMLElement | null) => void;
  readonly next: () => Promise<boolean>;
  readonly back: () => boolean;
  readonly goToStep: (step: string) => Promise<boolean>;
}

/** Typed context shared by FormWizard and its parts. */
export const formWizardContext = createContext<FormWizardContextValue>("FormWizard");
