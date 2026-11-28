import type { ComputedRef, InjectionKey } from "vue";

/** State a segmented item reads from its radiogroup. */
export interface SegmentedContext {
  readonly model: ComputedRef<string>;
  readonly disabled: ComputedRef<boolean>;
  readonly fallback: ComputedRef<string | undefined>;
  claim(value: string): void;
  choose(value: string): void;
}

/** Injection key for {@link SegmentedContext}. */
export const segmentedKey: InjectionKey<SegmentedContext> = Symbol("vize.segmented");
