import type { ThemeTokenPackName, ThemeTokenName, ThemeTokenOverrides } from "./theme-types.ts";
export {
  themeCascadeLayerOrder,
  themeDensityAttribute,
  themeDensityScales,
  themePresetAttribute,
  themePresets,
} from "./theme-constants.ts";

const invalidTokenDiagnostic = "VIZE_UI_THEME_TOKEN";

/** Independent token packs that can be composed with any preset. */
export const themeTokenPackNames: readonly ThemeTokenPackName[] = Object.freeze([
  "color",
  "typography",
  "space",
  "size",
  "radius",
  "border",
  "elevation",
  "opacity",
  "z-index",
  "focus",
  "density",
]);

/**
 * Semantic token contract mirrored from `theme.css`.
 *
 * The packaged stylesheet is the source of truth; this mirror exists so
 * JavaScript consumers can read the same headless defaults the CSS ships
 * with. Preset values are deliberately not mirrored: presets are opt-in
 * cascade, not API.
 */
export const themeTokens: Readonly<Record<ThemeTokenName, string>> = Object.freeze({
  "color-canvas": "Canvas",
  "color-surface": "Canvas",
  "color-text": "CanvasText",
  "color-text-muted": "GrayText",
  "color-accent": "LinkText",
  "color-accent-contrast": "Canvas",
  "color-border": "GrayText",
  "color-danger": "#d92d20",
  "type-family-sans": "system-ui, sans-serif",
  "type-family-mono": "ui-monospace, monospace",
  "type-size-xs": "0.75rem",
  "type-size-sm": "0.875rem",
  "type-size-md": "1rem",
  "type-size-lg": "1.125rem",
  "type-size-xl": "1.25rem",
  "type-size-2xl": "1.5rem",
  "type-leading-tight": "1.25",
  "type-leading-normal": "1.5",
  "type-leading-loose": "1.75",
  "type-weight-regular": "400",
  "type-weight-medium": "500",
  "type-weight-bold": "700",
  density: "1",
  "space-xs": "calc(0.25rem * var(--vize-ui-density))",
  "space-sm": "calc(0.5rem * var(--vize-ui-density))",
  "space-md": "calc(0.75rem * var(--vize-ui-density))",
  "space-lg": "calc(1rem * var(--vize-ui-density))",
  "space-xl": "calc(1.5rem * var(--vize-ui-density))",
  "space-2xl": "calc(2rem * var(--vize-ui-density))",
  "space-3xl": "calc(3rem * var(--vize-ui-density))",
  "size-control-sm": "calc(1.75rem * var(--vize-ui-density))",
  "size-control-md": "calc(2.25rem * var(--vize-ui-density))",
  "size-control-lg": "calc(2.75rem * var(--vize-ui-density))",
  "radius-sm": "0.25rem",
  "radius-md": "0.5rem",
  "radius-lg": "1rem",
  "radius-full": "9999px",
  "border-width-thin": "1px",
  "border-width-thick": "2px",
  "elevation-raised": "none",
  "elevation-overlay": "none",
  "elevation-floating": "none",
  "opacity-muted": "0.7",
  "opacity-disabled": "0.45",
  "z-sticky": "100",
  "z-dropdown": "600",
  "z-overlay": "1000",
  "z-toast": "1200",
  "focus-ring-width": "2px",
  "focus-ring-offset": "2px",
  "focus-ring-color": "var(--vize-ui-color-accent)",
});

function assertTokenName(name: string): void {
  if (!Object.hasOwn(themeTokens, name)) {
    throw new TypeError(`${invalidTokenDiagnostic}: unknown theme token "${name}"`);
  }
}

function assertTokenPackName(name: string): asserts name is ThemeTokenPackName {
  if (!themeTokenPackNames.includes(name as ThemeTokenPackName)) {
    throw new TypeError(`${invalidTokenDiagnostic}: unknown theme token pack "${name}"`);
  }
}

/** Custom-property name (`--vize-ui-*`) for one theme token. */
export function themeTokenProperty(name: ThemeTokenName): string {
  assertTokenName(name);
  return `--vize-ui-${name}`;
}

/** `var()` reference to one theme token for imperative style composition. */
export function themeTokenVar(name: ThemeTokenName): string {
  return `var(${themeTokenProperty(name)})`;
}

/** Tokens in one independent token pack, derived from the shared token mirror. */
export function themeTokensForPack(pack: ThemeTokenPackName): readonly ThemeTokenName[] {
  assertTokenPackName(pack);
  const prefix = pack === "typography" ? "type" : pack === "z-index" ? "z" : pack;

  return Object.freeze(
    (Object.keys(themeTokens) as ThemeTokenName[]).filter(
      (token) => token === prefix || token.startsWith(`${prefix}-`),
    ),
  );
}

/**
 * Override theme tokens on one element's subtree.
 *
 * Custom properties inherit, so scoping an override to an element retunes
 * every consumer below it — a nested theme scope without forking components.
 * Returns a restore function that reinstates the element's previous values.
 */
export function setThemeTokens(
  element: ElementCSSInlineStyle,
  overrides: ThemeTokenOverrides,
): () => void {
  if (typeof element?.style?.setProperty !== "function") {
    throw new TypeError(`${invalidTokenDiagnostic}: overrides need an element with inline style`);
  }

  const previous = new Map<string, string>();
  for (const [name, value] of Object.entries(overrides)) {
    if (value === undefined) continue;
    if (typeof value !== "string" || value.trim().length === 0) {
      throw new TypeError(`${invalidTokenDiagnostic}: token "${name}" needs a non-empty string`);
    }
    const property = themeTokenProperty(name as ThemeTokenName);
    if (!previous.has(property)) previous.set(property, element.style.getPropertyValue(property));
    element.style.setProperty(property, value);
  }

  return () => {
    for (const [property, value] of previous) {
      if (value === "") element.style.removeProperty(property);
      else element.style.setProperty(property, value);
    }
    previous.clear();
  };
}
