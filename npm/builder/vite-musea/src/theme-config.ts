/**
 * Build the theme config object from plugin options for runtime injection.
 */
export function buildThemeConfig(
  theme?:
    | string
    | { name: string; base?: "dark" | "light"; colors: Record<string, string> }
    | Array<{ name: string; base?: "dark" | "light"; colors: Record<string, string> }>,
):
  | {
      default: string;
      custom?: Record<string, { base?: "dark" | "light"; colors: Record<string, string> }>;
    }
  | undefined {
  if (!theme) return undefined;

  if (typeof theme === "string") {
    // 'dark' | 'light' | 'system'
    return { default: theme };
  }

  // Single custom theme or array of custom themes
  const themes = Array.isArray(theme) ? theme : [theme];
  const custom: Record<string, { base?: "dark" | "light"; colors: Record<string, string> }> = {};
  for (const t of themes) {
    custom[t.name] = {
      base: t.base,
      colors: t.colors as Record<string, string>,
    };
  }
  return {
    default: themes[0].name,
    custom,
  };
}
