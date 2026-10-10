type FontUsage = { familyName: string; glyphCount: number; isCustomFont?: boolean };

export function usesInstalledCjkFont(fonts: readonly FontUsage[]): boolean {
  return fonts.some(
    (font) =>
      font.glyphCount > 0 &&
      (/Noto Sans CJK/.test(font.familyName) ||
        (font.familyName === "Noto Sans Mono CJK JP" && font.isCustomFont === false)),
  );
}
