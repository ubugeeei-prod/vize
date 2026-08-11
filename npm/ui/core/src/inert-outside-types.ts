import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Attribute strategy used to isolate content outside an allowed subtree. */
export type InertOutsideMode = "aria-hidden" | "both" | "inert";

/** Reactive configuration for one document-level inerting layer. */
export interface InertOutsideOptions {
  /** Primary allowed root. It may be `null` during SSR and before mount. */
  readonly root: MaybeRefOrGetter<Element | null | undefined>;

  /** Additional portalled or exceptional roots that remain available. */
  readonly branches?: MaybeRefOrGetter<readonly Element[] | undefined>;

  /** Whether an activated controller currently isolates outside content. @default true */
  readonly enabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Isolation attributes to apply. `aria-hidden` alone does not prevent focus or pointer input;
   * reserve that mode for integrations that provide equivalent interaction blocking separately.
   * @default "both"
   */
  readonly mode?: MaybeRefOrGetter<InertOutsideMode | undefined>;
}

/** Lifecycle and inspection interface returned by `createInertOutside`. */
export interface InertOutsideController {
  /** Whether this controller is activated, independently of reactive enablement. */
  readonly isActive: Readonly<ShallowRef<boolean>>;

  /** Elements selected by this layer during the latest recomputation. */
  readonly affectedElements: Readonly<ShallowRef<readonly Element[]>>;

  /** Join the document isolation stack. Safe to call repeatedly. */
  readonly activate: () => void;

  /** Leave the stack and restore every attribute owned by this layer. */
  readonly deactivate: () => void;

  /** Re-read roots and options after imperative DOM movement. */
  readonly refresh: () => void;

  /** Permanently release reactive and document-level ownership. */
  readonly dispose: () => void;
}
