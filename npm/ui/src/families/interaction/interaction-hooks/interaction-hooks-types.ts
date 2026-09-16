import type { MaybeRefOrGetter, ShallowRef } from "vue";

import type {
  FocusController,
  FocusProps,
  FocusRingOptions,
  FocusWithinOptions,
} from "../../accessibility/focus/focus.ts";
import type {
  InteractionModality,
  InteractionModalityOptions,
  InteractionModalityTracker,
} from "../../accessibility/interaction-modality/interaction-modality.ts";
import type { HoverController, HoverOptions, HoverProps } from "../hover/hover.ts";
import type { PressController, PressOptions, PressProps } from "../press/press.ts";

export type InteractionFeatureOptions<Options> = false | Options | undefined;

/** Platform family used to resolve `Mod` in host-scoped shortcuts. */
export type InteractionShortcutPlatform = "apple" | "standard";

/** Immutable notification for a host-scoped shortcut match. */
export interface InteractionShortcutMatch {
  /** Original shortcut pattern supplied at registration. */
  readonly shortcut: string;

  /** Help description supplied at registration, or `null`. */
  readonly description: string | null;

  /** Native keyboard event that completed the shortcut. */
  readonly originalEvent: KeyboardEvent;
}

/** Options for one host-scoped shortcut binding. */
export interface InteractionShortcutBinding {
  /** Pattern such as `"Mod+K"` or `"Shift+Enter"`. */
  readonly shortcut: string;

  /** Called when the host receives the matching keydown event. */
  readonly handler: (match: InteractionShortcutMatch) => void;

  /**
   * Cancel the native action when this shortcut matches.
   *
   * @default true
   */
  readonly preventDefault?: boolean;

  /**
   * Dispatch repeated keydown events.
   *
   * @default false
   */
  readonly allowRepeat?: boolean;

  /** Help description surfaced through binding metadata. */
  readonly description?: string;
}

/** Options for lightweight host-scoped keyboard shortcut routing. */
export interface InteractionShortcutOptions {
  /**
   * Initial host-scoped shortcuts.
   *
   * @default []
   */
  readonly bindings?: readonly InteractionShortcutBinding[];

  /**
   * Layout used to resolve the `Mod` platform modifier.
   *
   * @default "standard"
   */
  readonly platform?: InteractionShortcutPlatform;

  /**
   * Ignore input while true.
   *
   * @default false
   */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;
}

/** Stable keyboard handler to spread onto the interaction host. */
export interface InteractionShortcutProps {
  readonly onKeydown: (event: KeyboardEvent) => void;
}

/** Host-scoped shortcut router owned by `createInteractionHooks`. */
export interface InteractionShortcutController {
  /** Stable props merged into the interaction host. */
  readonly shortcutProps: Readonly<InteractionShortcutProps>;

  /** Add a host-scoped binding and return its releaser. */
  readonly register: (binding: InteractionShortcutBinding) => () => void;

  /** Clear transient shortcut state. Host-scoped shortcuts have none. */
  readonly reset: () => false;

  /** Snapshot every binding in registration order. */
  readonly getBindings: () => readonly InteractionShortcutBinding[];

  /** Release bindings and make imperative calls terminal. */
  readonly dispose: () => void;
}

export interface InteractionHooksOptions {
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;
  /**
   * Track document-level keyboard, pointer, touch, and virtual input modality.
   *
   * @default false
   */
  readonly modality?: InteractionFeatureOptions<InteractionModalityOptions>;
  readonly focusRing?: InteractionFeatureOptions<FocusRingOptions>;
  readonly focusWithin?: InteractionFeatureOptions<FocusWithinOptions>;
  readonly hover?: InteractionFeatureOptions<HoverOptions>;
  readonly press?: InteractionFeatureOptions<PressOptions>;
  /**
   * Route element-scoped or explicitly attached keyboard shortcuts.
   *
   * When enabled without a target, the registry stays host-scoped through the
   * returned props instead of attaching a document listener.
   *
   * @default false
   */
  readonly shortcuts?: InteractionFeatureOptions<InteractionShortcutOptions>;
}

type ModalityFeature<Options> = Options extends { readonly modality: infer Feature }
  ? Feature
  : false;
type PressFeature<Options> = Options extends { readonly press: infer Feature }
  ? Feature
  : undefined;
type HoverFeature<Options> = Options extends { readonly hover: infer Feature }
  ? Feature
  : undefined;
type FocusRingFeature<Options> = Options extends { readonly focusRing: infer Feature }
  ? Feature
  : undefined;
type FocusWithinFeature<Options> = Options extends { readonly focusWithin: infer Feature }
  ? Feature
  : false;
type ShortcutFeature<Options> = Options extends { readonly shortcuts: infer Feature }
  ? Feature
  : false;

type FeatureController<Feature, Controller> = [Feature] extends [false] ? null : Controller;
type FeatureProps<Feature, Props> = [Feature] extends [false] ? object : Props;

export type InteractionHooksProps<Options extends InteractionHooksOptions = {}> = FeatureProps<
  PressFeature<Options>,
  PressProps
> &
  FeatureProps<HoverFeature<Options>, HoverProps> &
  FeatureProps<FocusRingFeature<Options>, FocusProps> &
  FeatureProps<FocusWithinFeature<Options>, FocusProps> &
  FeatureProps<ShortcutFeature<Options>, InteractionShortcutProps>;

export interface InteractionHooksCancelResult {
  readonly focusRing: boolean;
  readonly focusWithin: boolean;
  readonly hover: boolean;
  readonly press: boolean;
  readonly shortcuts: boolean;
}

export interface InteractionHooksController<Options extends InteractionHooksOptions = {}> {
  readonly modalityTracker: FeatureController<ModalityFeature<Options>, InteractionModalityTracker>;
  readonly shortcutController: FeatureController<
    ShortcutFeature<Options>,
    InteractionShortcutController
  >;
  readonly focusRing: FeatureController<FocusRingFeature<Options>, FocusController>;
  readonly focusWithin: FeatureController<FocusWithinFeature<Options>, FocusController>;
  readonly hover: FeatureController<HoverFeature<Options>, HoverController>;
  readonly press: FeatureController<PressFeature<Options>, PressController>;
  readonly interactionProps: Readonly<InteractionHooksProps<Options>>;
  readonly currentModality: Readonly<ShallowRef<InteractionModality | null>>;
  readonly isFocused: Readonly<ShallowRef<boolean>>;
  readonly isFocusVisible: Readonly<ShallowRef<boolean>>;
  readonly isFocusWithin: Readonly<ShallowRef<boolean>>;
  readonly isHovered: Readonly<ShallowRef<boolean>>;
  readonly isPressed: Readonly<ShallowRef<boolean>>;
  readonly cancel: () => InteractionHooksCancelResult;
  readonly dispose: () => void;
}
