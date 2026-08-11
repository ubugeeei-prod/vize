import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Viewport-locking strategy selected for a document. */
export type ScrollLockStrategy = "auto" | "fixed" | "overflow";

/** Reactive configuration for one document scroll-lock owner. */
export interface ScrollLockOptions {
  /** Document whose layout viewport is locked. It may be `null` during SSR. */
  readonly document: MaybeRefOrGetter<Document | null | undefined>;

  /** Whether an activated controller currently participates in the lock stack. @default true */
  readonly enabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Lock using root overflow or a fixed body. `auto` uses fixed positioning on iOS-like
   * touch platforms and root overflow elsewhere.
   * @default "auto"
   */
  readonly strategy?: MaybeRefOrGetter<ScrollLockStrategy | undefined>;

  /** Reserve a classic scrollbar gutter so viewport-width content does not shift. @default true */
  readonly preserveScrollbarGap?: MaybeRefOrGetter<boolean | undefined>;

  /** Restore the captured layout-viewport offset when the final owner releases. @default true */
  readonly restoreScroll?: MaybeRefOrGetter<boolean | undefined>;
}

/** Lifecycle and inspection interface returned by `createScrollLock`. */
export interface ScrollLockController {
  /** Whether this controller has been activated, independently of reactive enablement. */
  readonly isActive: Readonly<ShallowRef<boolean>>;

  /** Whether this controller currently contributes to a document lock. */
  readonly isLocked: Readonly<ShallowRef<boolean>>;

  /** Classic scrollbar width measured before the current document lock. */
  readonly scrollbarGap: Readonly<ShallowRef<number>>;

  /** Effective strategy after platform resolution and nested-lock composition. */
  readonly resolvedStrategy: Readonly<ShallowRef<Exclude<ScrollLockStrategy, "auto"> | null>>;

  /** Join the document lock stack. Safe to call repeatedly. */
  readonly activate: () => void;

  /** Leave the lock stack and restore document state when the final owner exits. */
  readonly deactivate: () => void;

  /** Re-read reactive options and recover after imperative document changes. */
  readonly refresh: () => void;

  /** Permanently release reactive and document-level ownership. */
  readonly dispose: () => void;
}
