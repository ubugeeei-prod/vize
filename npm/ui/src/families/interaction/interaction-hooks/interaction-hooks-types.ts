import type { MaybeRefOrGetter, ShallowRef } from "vue";

import type {
  FocusController,
  FocusProps,
  FocusRingOptions,
  FocusWithinOptions,
} from "../../accessibility/focus/focus.ts";
import type { HoverController, HoverOptions, HoverProps } from "../hover/hover.ts";
import type { PressController, PressOptions, PressProps } from "../press/press.ts";

export type InteractionFeatureOptions<Options> = false | Options | undefined;

export interface InteractionHooksOptions {
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;
  readonly focusRing?: InteractionFeatureOptions<FocusRingOptions>;
  readonly focusWithin?: InteractionFeatureOptions<FocusWithinOptions>;
  readonly hover?: InteractionFeatureOptions<HoverOptions>;
  readonly press?: InteractionFeatureOptions<PressOptions>;
}

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

type FeatureController<Feature, Controller> = [Feature] extends [false] ? null : Controller;
type FeatureProps<Feature, Props> = [Feature] extends [false] ? object : Props;

export type InteractionHooksProps<Options extends InteractionHooksOptions = {}> = FeatureProps<
  PressFeature<Options>,
  PressProps
> &
  FeatureProps<HoverFeature<Options>, HoverProps> &
  FeatureProps<FocusRingFeature<Options>, FocusProps> &
  FeatureProps<FocusWithinFeature<Options>, FocusProps>;

export interface InteractionHooksCancelResult {
  readonly focusRing: boolean;
  readonly focusWithin: boolean;
  readonly hover: boolean;
  readonly press: boolean;
}

export interface InteractionHooksController<Options extends InteractionHooksOptions = {}> {
  readonly focusRing: FeatureController<FocusRingFeature<Options>, FocusController>;
  readonly focusWithin: FeatureController<FocusWithinFeature<Options>, FocusController>;
  readonly hover: FeatureController<HoverFeature<Options>, HoverController>;
  readonly press: FeatureController<PressFeature<Options>, PressController>;
  readonly interactionProps: Readonly<InteractionHooksProps<Options>>;
  readonly isFocused: Readonly<ShallowRef<boolean>>;
  readonly isFocusVisible: Readonly<ShallowRef<boolean>>;
  readonly isFocusWithin: Readonly<ShallowRef<boolean>>;
  readonly isHovered: Readonly<ShallowRef<boolean>>;
  readonly isPressed: Readonly<ShallowRef<boolean>>;
  readonly cancel: () => InteractionHooksCancelResult;
  readonly dispose: () => void;
}
