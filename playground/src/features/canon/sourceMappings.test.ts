import { describe, expect, it } from "vite-plus/test";
import { mapGeneratedRange, parseSourceMap } from "./sourceMappings";

describe("virtual TypeScript source mappings", () => {
  const mapping = { genStart: 10, genEnd: 20, srcStart: 30, srcEnd: 40 };

  it("maps direct ranges and excludes the half-open endpoint", () => {
    expect(mapGeneratedRange(12, 15, [mapping])).toEqual({ start: 32, end: 35 });
    expect(mapGeneratedRange(18, 25, [mapping])).toEqual({ start: 38, end: 40 });
    expect(mapGeneratedRange(20, 21, [mapping])).toBeNull();
    expect(mapGeneratedRange(0, 2, [mapping])).toBeNull();
  });

  it("uses the authored range for synthetic checks and prefers expression subspans", () => {
    const synthetic = { ...mapping, genEnd: 100 };
    expect(mapGeneratedRange(80, 95, [synthetic])).toEqual({ start: 30, end: 40 });
    expect(mapGeneratedRange(12, 14, [{ ...synthetic, subSpans: [mapping] }])).toEqual({
      start: 32,
      end: 34,
    });
  });

  it("retains compatibility with older WASM numeric mapping comments", () => {
    expect(parseSourceMap("// @vize-map: 10:20 -> 30:40\n// @vize-map: binding:x")).toEqual([
      mapping,
    ]);
  });
});
