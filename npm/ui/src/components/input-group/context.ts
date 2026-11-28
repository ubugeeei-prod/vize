import type { ComputedRef, InjectionKey } from "vue";

/** Disabled state shared by every control inside an input group. */
export interface InputGroupContext {
  readonly disabled: ComputedRef<boolean>;
}

/** Injection key for {@link InputGroupContext}. */
export const inputGroupKey: InjectionKey<InputGroupContext> = Symbol("vize.input-group");
