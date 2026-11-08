import {
  computed,
  inject,
  provide,
  toValue,
  type InjectionKey,
  type MaybeRefOrGetter,
  type Ref,
} from "vue";

import type { FrescoBorderStyle } from "./protocol.js";

/** Semantic terminal colors consumed by Fresco components and applications. */
export interface FrescoThemeColors {
  foreground: string;
  background: string;
  muted: string;
  primary: string;
  accent: string;
  positive: string;
  caution: string;
  negative: string;
  info: string;
  focus: string;
}

/** Cell-based spacing scale shared by layout components. */
export interface FrescoThemeSpacing {
  none: number;
  xs: number;
  sm: number;
  md: number;
  lg: number;
  xl: number;
}

/** Border defaults for progressively stronger terminal surfaces. */
export interface FrescoThemeBorders {
  subtle: FrescoBorderStyle;
  regular: FrescoBorderStyle;
  strong: FrescoBorderStyle;
}

/** Animation timing contract, including the effective reduced-motion policy. */
export interface FrescoThemeMotion {
  reduced: boolean;
  fastMs: number;
  regularMs: number;
  slowMs: number;
}

/** Complete, resolved Fresco theme available to descendants. */
export interface FrescoTheme {
  colors: FrescoThemeColors;
  spacing: FrescoThemeSpacing;
  borders: FrescoThemeBorders;
  motion: FrescoThemeMotion;
}

/** Scoped partial theme layered over the nearest provider. */
export interface FrescoThemeOverride {
  colors?: Partial<FrescoThemeColors>;
  spacing?: Partial<FrescoThemeSpacing>;
  borders?: Partial<FrescoThemeBorders>;
  motion?: Partial<FrescoThemeMotion>;
}

/** Static defaults shared without runtime stylesheet or token generation. */
export const defaultFrescoTheme: Readonly<FrescoTheme> = Object.freeze({
  colors: Object.freeze({
    foreground: "white",
    background: "black",
    muted: "gray",
    primary: "cyan",
    accent: "magenta",
    positive: "green",
    caution: "yellow",
    negative: "red",
    info: "blue",
    focus: "cyan",
  }),
  spacing: Object.freeze({ none: 0, xs: 1, sm: 1, md: 2, lg: 3, xl: 4 }),
  borders: Object.freeze({ subtle: "single", regular: "rounded", strong: "double" }),
  motion: Object.freeze({ reduced: false, fastMs: 80, regularMs: 160, slowMs: 240 }),
});

/** Injection identity for framework adapters that bridge Fresco theme scopes. */
export const FRESCO_THEME_KEY: InjectionKey<Readonly<Ref<FrescoTheme>>> = Symbol("fresco-theme");

/** Resolve one scoped override without mutating its parent theme. */
export function resolveFrescoTheme(
  parent: Readonly<FrescoTheme>,
  override: Readonly<FrescoThemeOverride> = {},
): FrescoTheme {
  return {
    colors: { ...parent.colors, ...override.colors },
    spacing: { ...parent.spacing, ...override.spacing },
    borders: { ...parent.borders, ...override.borders },
    motion: { ...parent.motion, ...override.motion },
  };
}

/** Provide a reactive theme scoped to the current component subtree. */
export function provideFrescoTheme(
  override: MaybeRefOrGetter<FrescoThemeOverride> = {},
): Readonly<Ref<FrescoTheme>> {
  const parent = inject(FRESCO_THEME_KEY, undefined);
  const theme = computed(() =>
    resolveFrescoTheme(parent?.value ?? defaultFrescoTheme, toValue(override)),
  );
  provide(FRESCO_THEME_KEY, theme);
  return theme;
}

/** Read the nearest resolved theme, falling back to the static defaults. */
export function useFrescoTheme(): Readonly<Ref<FrescoTheme>> {
  const theme = inject(FRESCO_THEME_KEY, undefined);
  return theme ?? computed(() => defaultFrescoTheme as FrescoTheme);
}
