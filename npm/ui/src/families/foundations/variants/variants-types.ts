/** Any value accepted by {@link cx}: strings, numbers, conditional dictionaries, and nested lists. */
export type ClassValue =
  | string
  | number
  | bigint
  | boolean
  | null
  | undefined
  | ClassDictionary
  | readonly ClassValue[];

/** Conditional classes: every key whose value is truthy is included. */
export interface ClassDictionary {
  readonly [className: string]: unknown;
}

/**
 * Post-processor applied to every joined class string, e.g. `tailwind-merge`'s
 * `twMerge` to resolve conflicting utilities.
 */
export type ClassMerger = (className: string) => string;

/** Class value allowed for a variant option or slot base when slots exist (no dictionaries). */
export type SlotClassValue = string | null | undefined | false | readonly SlotClassValue[];

/** Per-slot classes for one variant option. */
export type SlotClassMap<SlotName extends string> = {
  readonly [Slot in SlotName]?: SlotClassValue;
};

/** Slot definitions: slot name to base classes. */
export type SlotDefinitions = Readonly<Record<string, SlotClassValue>>;

/** Variant definitions: variant name to option name to classes. */
export type VariantDefinitions = Readonly<
  Record<string, Readonly<Record<string, ClassValue | SlotClassMap<string>>>>
>;

/** Map `"true" | "false"` option keys to `boolean`; keep other option keys. */
export type StringToBoolean<Key> = Key extends "true" | "false" ? boolean : Key;

/** Selectable value of one variant, with boolean variants inferred from `true`/`false` keys. */
export type VariantValue<Options> = StringToBoolean<Extract<keyof Options, string>>;

/** Responsive value: a plain value or a per-breakpoint map with an optional `initial`. */
export type ResponsiveValue<Value, Breakpoint extends string> =
  | Value
  | ({ readonly initial?: Value } & { readonly [Key in Breakpoint]?: Value });

/** Props accepted by a recipe, inferred from its variants and breakpoints. */
export type RecipeVariantProps<Variants extends VariantDefinitions, Breakpoint extends string> = {
  readonly [Name in keyof Variants]?: [Breakpoint] extends [never]
    ? VariantValue<Variants[Name]>
    : ResponsiveValue<VariantValue<Variants[Name]>, Breakpoint>;
};

/** Default option per variant. */
export type DefaultVariants<Variants extends VariantDefinitions> = {
  readonly [Name in keyof Variants]?: VariantValue<Variants[Name]>;
};

/** Condition matching one or several options of each named variant. */
export type CompoundCondition<Variants extends VariantDefinitions> = {
  readonly [Name in keyof Variants]?:
    | VariantValue<Variants[Name]>
    | readonly VariantValue<Variants[Name]>[];
};

/** Classes added when every condition of a compound variant matches. */
export type CompoundVariant<
  Variants extends VariantDefinitions,
  SlotName extends string,
> = CompoundCondition<Variants> & {
  /** Classes (or per-slot classes when the recipe has slots). */
  readonly class: [SlotName] extends [never] ? ClassValue : SlotClassMap<SlotName>;
};

/** Configuration accepted by `defineVariants`. */
export interface VariantsConfig<
  Variants extends VariantDefinitions,
  Slots extends SlotDefinitions,
  Breakpoint extends string,
> {
  /** Base classes (the `base` slot when slots are defined). */
  readonly base?: ClassValue;
  /** Named slots and their base classes. */
  readonly slots?: Slots;
  /** Variant definitions. */
  readonly variants?: Variants;
  /** Classes applied when several variant conditions hold together. */
  readonly compoundVariants?: readonly CompoundVariant<
    Variants,
    [keyof Slots] extends [never] ? never : Extract<keyof Slots, string> | "base"
  >[];
  /** Option used when a prop is omitted. */
  readonly defaultVariants?: DefaultVariants<Variants>;
  /**
   * Breakpoint names enabling responsive variant values. Responsive classes
   * are emitted as `${breakpoint}${separator}${class}` (Tailwind syntax).
   */
  readonly responsive?: readonly Breakpoint[];
}

/** Options for `defineVariants`. */
export interface DefineVariantsOptions {
  /**
   * Post-processor for the joined class string (plug `tailwind-merge` here).
   *
   * @default undefined (no merging)
   */
  readonly merge?: ClassMerger;

  /**
   * Separator between a breakpoint and a class in responsive output.
   *
   * @default ":"
   */
  readonly responsiveSeparator?: string;
}

/** Props of a recipe call: variant props plus an extra `class`. */
export type RecipeProps<
  Variants extends VariantDefinitions,
  Breakpoint extends string,
> = RecipeVariantProps<Variants, Breakpoint> & {
  /** Extra classes appended last (after compound variants). */
  readonly class?: ClassValue;
};

/** Per-slot class function returned by a slot recipe. */
export type SlotClassFunction<Variants extends VariantDefinitions, Breakpoint extends string> = (
  overrides?: RecipeProps<Variants, Breakpoint>,
) => string;

/** Metadata shared by every recipe. */
export interface RecipeMetadata<Variants extends VariantDefinitions> {
  /** Variant names in declaration order. */
  readonly variantKeys: readonly Extract<keyof Variants, string>[];
  /**
   * Split props into variant props and the rest (for forwarding the rest to a
   * child component).
   */
  readonly splitVariantProps: <Props extends object>(
    props: Props,
  ) => readonly [Pick<Props, Extract<keyof Props, keyof Variants>>, Omit<Props, keyof Variants>];
}

/** Recipe without slots: returns one class string. */
export interface ClassRecipe<
  Variants extends VariantDefinitions,
  Breakpoint extends string,
> extends RecipeMetadata<Variants> {
  (props?: RecipeProps<Variants, Breakpoint>): string;
}

/** Recipe with slots: returns one class function per slot (plus `base`). */
export interface SlotRecipe<
  Variants extends VariantDefinitions,
  SlotName extends string,
  Breakpoint extends string,
> extends RecipeMetadata<Variants> {
  (props?: RecipeProps<Variants, Breakpoint>): {
    readonly [Slot in SlotName | "base"]: SlotClassFunction<Variants, Breakpoint>;
  };
  /** Slot names in declaration order (including `base`). */
  readonly slotKeys: readonly (SlotName | "base")[];
}

/** Any recipe produced by `defineVariants`. */
export type AnyRecipe =
  | ClassRecipe<VariantDefinitions, string>
  | SlotRecipe<VariantDefinitions, string, string>;

/**
 * Variant props of a recipe, for component `defineProps`.
 *
 * ```ts
 * type ButtonVariants = VariantProps<typeof button>;
 * ```
 */
export type VariantProps<Recipe> = Recipe extends (props?: infer Props) => unknown
  ? Omit<NonNullable<Props>, "class">
  : never;
