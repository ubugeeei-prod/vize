import type {
  ClassMerger,
  ClassRecipe,
  ClassValue,
  DefineVariantsOptions,
  RecipeMetadata,
  SlotDefinitions,
  SlotRecipe,
  VariantDefinitions,
  VariantsConfig,
} from "./variants-types.ts";

export type {
  AnyRecipe,
  ClassDictionary,
  ClassMerger,
  ClassRecipe,
  ClassValue,
  CompoundCondition,
  CompoundVariant,
  DefaultVariants,
  DefineVariantsOptions,
  RecipeMetadata,
  RecipeProps,
  RecipeVariantProps,
  ResponsiveValue,
  SlotClassFunction,
  SlotClassMap,
  SlotClassValue,
  SlotDefinitions,
  SlotRecipe,
  StringToBoolean,
  VariantDefinitions,
  VariantProps,
  VariantsConfig,
  VariantValue,
} from "./variants-types.ts";

const invalidBreakpoint = "VIZE_UI_VARIANTS_BREAKPOINT";

/** Structural view of every accepted configuration, used by the implementation. */
interface LooseVariantsConfig {
  readonly base?: ClassValue;
  readonly slots?: SlotDefinitions | undefined;
  readonly variants?: VariantDefinitions;
  readonly compoundVariants?: readonly Readonly<Record<string, unknown>>[];
  readonly defaultVariants?: Readonly<Record<string, unknown>>;
  readonly responsive?: readonly string[];
}

/**
 * Join class values into one space-separated string.
 *
 * Accepts strings, numbers, nested arrays, and dictionaries whose truthy keys
 * are included. Falsy values are skipped. Dependency-free and pure, so it is
 * safe during SSR and produces identical output on server and client.
 *
 * @param inputs Class values.
 * @returns Joined class string (no duplicates are removed; use a merger for that).
 */
export function cx(...inputs: readonly ClassValue[]): string {
  let output = "";
  const append = (value: ClassValue): void => {
    if (value === null || value === undefined || value === false || value === true) return;
    if (typeof value === "string" || typeof value === "number" || typeof value === "bigint") {
      const text = String(value);
      if (text.length === 0) return;
      output = output.length === 0 ? text : `${output} ${text}`;
      return;
    }
    if (isClassList(value)) {
      for (const item of value) append(item);
      return;
    }
    for (const key of Object.keys(value)) if (value[key]) append(key);
  };
  for (const input of inputs) append(input);
  return output;
}

/**
 * Create a {@link cx} variant that post-processes its output, e.g. with
 * `tailwind-merge`:
 *
 * ```ts
 * import { twMerge } from "tailwind-merge";
 * export const cn = createCx(twMerge);
 * ```
 *
 * @param merge Post-processor for the joined string.
 * @returns A class joiner that applies `merge`.
 */
export function createCx(merge: ClassMerger): (...inputs: readonly ClassValue[]) => string {
  return (...inputs) => merge(cx(...inputs));
}

/**
 * Define a typed class recipe (a `cva`/`tailwind-variants` style API).
 *
 * Variant props are inferred from the configuration: option keys become a
 * string-literal union, `true`/`false` keys become `boolean`, and
 * `defaultVariants`/`compoundVariants` are checked against them. With
 * `responsive`, each variant also accepts `{ initial, sm, md, ... }` maps whose
 * classes are prefixed with the breakpoint. Pure and allocation-light, so it
 * renders identically during SSR.
 *
 * @param config Base classes, variants, compound variants, defaults, and breakpoints.
 * @param options Class merger and responsive separator.
 * @returns A function from variant props to a class string.
 */
export function defineVariants<
  const Variants extends VariantDefinitions = {},
  const Breakpoint extends string = never,
>(
  config: VariantsConfig<Variants, {}, Breakpoint> & { readonly slots?: undefined },
  options?: DefineVariantsOptions,
): ClassRecipe<Variants, Breakpoint>;
/**
 * Define a typed slot recipe: one class function per slot (`base` included).
 *
 * Each slot function accepts per-call overrides (variant props and `class`).
 * Variant options and compound classes are per-slot maps.
 *
 * @param config Slots, variants, compound variants, defaults, and breakpoints.
 * @param options Class merger and responsive separator.
 * @returns A function from variant props to per-slot class functions.
 */
export function defineVariants<
  const Slots extends SlotDefinitions,
  const Variants extends VariantDefinitions = {},
  const Breakpoint extends string = never,
>(
  config: VariantsConfig<Variants, Slots, Breakpoint> & { readonly slots: Slots },
  options?: DefineVariantsOptions,
): SlotRecipe<Variants, Extract<keyof Slots, string>, Breakpoint>;
export function defineVariants(
  config: LooseVariantsConfig,
  options: DefineVariantsOptions = {},
): ClassRecipe<VariantDefinitions, string> | SlotRecipe<VariantDefinitions, string, string> {
  const variants = config.variants ?? {};
  const variantKeys = Object.keys(variants);
  const defaults: Readonly<Record<string, unknown>> = config.defaultVariants ?? {};
  const compounds = config.compoundVariants ?? [];
  const breakpoints = new Set<string>(config.responsive ?? []);
  const separator = options.responsiveSeparator ?? ":";
  const finish = (value: string): string => (options.merge ? options.merge(value) : value);
  const slots = config.slots;
  const slotKeys = slots ? ["base", ...Object.keys(slots).filter((key) => key !== "base")] : [];

  const optionClass = (option: unknown, slot: string | null): ClassValue => {
    if (slot === null) return toClassValue(option);
    if (!isPlainObject(option)) return slot === "base" ? toClassValue(option) : undefined;
    return toClassValue(option[slot]);
  };

  const resolve = (props: Readonly<Record<string, unknown>>, slot: string | null): string => {
    const parts: ClassValue[] = [];
    if (slot === null || slot === "base") parts.push(config.base);
    if (slot !== null && slots && slot !== "base") parts.push(toClassValue(slots[slot]));
    if (slot === "base" && slots) parts.push(toClassValue(slots["base"]));
    const selection: Record<string, string | undefined> = {};

    for (const name of variantKeys) {
      const options = variants[name] ?? {};
      const raw = props[name] === undefined ? defaults[name] : props[name];
      if (isResponsive(raw)) {
        for (const [breakpoint, value] of Object.entries(raw)) {
          const key = toOptionKey(value, options);
          if (key === undefined) continue;
          const className = cx(optionClass(options[key], slot));
          if (breakpoint === "initial") {
            selection[name] = key;
            parts.push(className);
            continue;
          }
          if (!breakpoints.has(breakpoint)) {
            throw new Error(`${invalidBreakpoint}: unknown breakpoint "${breakpoint}" for ${name}`);
          }
          parts.push(prefix(className, `${breakpoint}${separator}`));
        }
        continue;
      }
      const key = toOptionKey(raw, options);
      selection[name] = key;
      if (key !== undefined) parts.push(optionClass(options[key], slot));
    }

    for (const compound of compounds) {
      if (!matches(compound, selection)) continue;
      const classes: unknown = compound.class;
      parts.push(slot === null ? toClassValue(classes) : optionClass(classes, slot));
    }
    parts.push(toClassValue(props["class"]));
    return finish(cx(parts));
  };

  const metadata: RecipeMetadata<VariantDefinitions> = {
    variantKeys,
    // The runtime split mirrors the declared Pick/Omit key partition exactly.
    splitVariantProps: (props) => splitProps(props, variantKeys) as never,
  };

  if (!slots) {
    return Object.assign(
      (props: Readonly<Record<string, unknown>> = {}) => resolve(props, null),
      metadata,
    );
  }
  return Object.assign(
    (props: Readonly<Record<string, unknown>> = {}) => {
      const result: Record<string, (overrides?: Readonly<Record<string, unknown>>) => string> = {};
      for (const slot of slotKeys) {
        result[slot] = (overrides = {}) =>
          resolve(
            {
              ...props,
              ...overrides,
              class: slot === "base" ? [props["class"], overrides["class"]] : overrides["class"],
            },
            slot,
          );
      }
      return result;
    },
    { ...metadata, slotKeys },
  );
}

function splitProps(
  props: object,
  keys: readonly string[],
): readonly [Record<string, unknown>, Record<string, unknown>] {
  const picked: Record<string, unknown> = {};
  const rest: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(props)) {
    if (keys.includes(key)) picked[key] = value;
    else rest[key] = value;
  }
  return [picked, rest];
}

function toOptionKey(
  value: unknown,
  options: Readonly<Record<string, unknown>>,
): string | undefined {
  if (value === undefined || value === null) {
    return Object.hasOwn(options, "false") ? "false" : undefined;
  }
  if (typeof value !== "string" && typeof value !== "number" && typeof value !== "boolean") {
    return undefined;
  }
  const key = String(value);
  return Object.hasOwn(options, key) ? key : undefined;
}

function matches(
  compound: Readonly<Record<string, unknown>>,
  selection: Readonly<Record<string, string | undefined>>,
): boolean {
  for (const [name, expected] of Object.entries(compound)) {
    if (name === "class") continue;
    const actual = selection[name];
    const accepted = Array.isArray(expected) ? expected : [expected];
    if (!accepted.some((value) => String(value) === actual)) return false;
  }
  return true;
}

function prefix(className: string, value: string): string {
  return className
    .split(/\s+/u)
    .filter((token) => token.length > 0)
    .map((token) => `${value}${token}`)
    .join(" ");
}

function isResponsive(value: unknown): value is Readonly<Record<string, unknown>> {
  return isPlainObject(value);
}

function isPlainObject(value: unknown): value is Readonly<Record<string, unknown>> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isClassList(value: object): value is readonly ClassValue[] {
  return Array.isArray(value);
}

function toClassValue(value: unknown): ClassValue {
  if (
    value === null ||
    value === undefined ||
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "bigint" ||
    typeof value === "boolean"
  ) {
    return value;
  }
  if (Array.isArray(value)) return value.map(toClassValue);
  if (isPlainObject(value)) return value;
  return undefined;
}
