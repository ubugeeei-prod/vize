export function rewriteReportedPaths(
  output: string,
  replacements: ReadonlyMap<string, string>,
): string {
  let rewritten = output;

  const orderedReplacements = [...replacements].sort(
    (left, right) => right[0].length - left[0].length,
  );

  for (const [from, to] of orderedReplacements) {
    rewritten = rewritten.split(from).join(to);
  }

  return rewritten;
}

if (import.meta.vitest) {
  const { describe, expect, it } = import.meta.vitest;

  describe("rewriteReportedPaths", () => {
    it("rewrites both absolute and relative temporary filenames", () => {
      const replacements = new Map<string, string>([
        [
          "/repo/node_modules/.vize/oxlint-plugin-vize/100-abcd/0-Example.vue",
          "/repo/src/Example.vue",
        ],
        ["node_modules/.vize/oxlint-plugin-vize/100-abcd/0-Example.vue", "/repo/src/Example.vue"],
      ]);

      expect(
        rewriteReportedPaths(
          [
            "node_modules/.vize/oxlint-plugin-vize/100-abcd/0-Example.vue",
            "/repo/node_modules/.vize/oxlint-plugin-vize/100-abcd/0-Example.vue",
          ].join("\n"),
          replacements,
        ),
      ).toBe(["/repo/src/Example.vue", "/repo/src/Example.vue"].join("\n"));
    });
  });
}
