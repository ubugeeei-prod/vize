import { describe, expect, it } from "vite-plus/test";
import { offsetToLineColumn } from "./position";

describe("offsetToLineColumn", () => {
  it("maps offsets without allocating source slices", () => {
    const source = "one\ntwo\n";

    expect(offsetToLineColumn(source, 0)).toEqual({ line: 1, column: 1 });
    expect(offsetToLineColumn(source, 3)).toEqual({ line: 1, column: 4 });
    expect(offsetToLineColumn(source, 4)).toEqual({ line: 2, column: 1 });
    expect(offsetToLineColumn(source, 7)).toEqual({ line: 2, column: 4 });
    expect(offsetToLineColumn(source, 8)).toEqual({ line: 3, column: 1 });
  });

  it("clamps offsets outside the source range", () => {
    expect(offsetToLineColumn("one\ntwo\n", -10)).toEqual({ line: 1, column: 1 });
    expect(offsetToLineColumn("one\ntwo\n", 100)).toEqual({ line: 3, column: 1 });
  });
});
