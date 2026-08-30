import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

/**
 * Input family that most recently expressed user intent.
 *
 * Pens and unknown pointing hardware intentionally map to `pointer`; consumers
 * that need device-specific geometry should inspect the original pointer event.
 */
export type InteractionModality = "keyboard" | "pointer" | "touch" | "virtual";

/** Why an {@link InteractionModalityTracker} changed. */
export type InteractionModalityChangeReason = InteractionModality | "document" | "manual";

/** Immutable notification emitted after a distinct modality change. */
export interface InteractionModalityChange {
  /** New modality, or `null` when a consumer explicitly resets detection. */
  readonly modality: InteractionModality | null;

  /** Value observed immediately before this change. */
  readonly previousModality: InteractionModality | null;

  /** Event classification or lifecycle operation responsible for the change. */
  readonly reason: InteractionModalityChangeReason;

  /** Native event when the change came from document input. */
  readonly originalEvent: Event | null;

  /** Document whose state produced the change, if the tracker is attached. */
  readonly document: Document | null;
}

/** Options for {@link createInteractionModalityTracker}. */
export interface InteractionModalityOptions {
  /**
   * Reactive document to observe. Pass `null` for SSR or deferred attachment.
   *
   * When omitted, the current global document is resolved lazily. No DOM global
   * is read while the module is evaluated.
   *
   * @default globalThis.document when available
   */
  readonly document?: MaybeRefOrGetter<Document | null | undefined>;

  /**
   * Value used before the first qualifying input event.
   *
   * @default null
   */
  readonly initialModality?: InteractionModality | null;

  /** Called synchronously after each distinct change. */
  readonly onChange?: (change: InteractionModalityChange) => void;
}

/** Reactive, explicitly disposable input-modality observer. */
export interface InteractionModalityTracker {
  /** Document currently observed by this tracker. */
  readonly document: Readonly<ShallowRef<Document | null>>;

  /** Most recently detected input family. */
  readonly modality: Readonly<ShallowRef<InteractionModality | null>>;

  /** Whether global focus treatment should currently be keyboard-visible. */
  readonly isFocusVisible: ComputedRef<boolean>;

  /** Attach to another document, retaining the current value until synchronized. */
  readonly attach: (document: Document | null) => boolean;

  /** Stop observing the current document without clearing the current value. */
  readonly detach: () => boolean;

  /** Explicitly set or reset modality and synchronize peers on the same document. */
  readonly setModality: (modality: InteractionModality | null) => boolean;

  /** Release reactive observation and native listeners. Safe to call repeatedly. */
  readonly dispose: () => void;
}
