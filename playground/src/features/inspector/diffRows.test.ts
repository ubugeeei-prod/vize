import { describe, expect, it } from "vite-plus/test";
import { buildSplitDiffRows, type HighlightedDiffLine } from "./diffRows";

function diffLine(
  kind: HighlightedDiffLine["kind"],
  leftLine: number | null,
  rightLine: number | null,
  text: string,
): HighlightedDiffLine {
  return {
    kind,
    leftLine,
    rightLine,
    text,
    tokens: [
      {
        content: text,
        darkColor: undefined,
        lightColor: undefined,
      },
    ],
  };
}

describe("buildSplitDiffRows", () => {
  it("keeps unchanged lines on both sides", () => {
    const rows = buildSplitDiffRows([diffLine("same", 1, 1, "return count")]);

    expect(rows).toHaveLength(1);
    expect(rows[0]!.left).toMatchObject({
      kind: "same",
      line: 1,
    });
    expect(rows[0]!.right).toMatchObject({
      kind: "same",
      line: 1,
    });
  });

  it("pairs similar changed lines after an added offset line", () => {
    const rows = buildSplitDiffRows([
      diffLine("remove", 1, null, "const title = _ctx.title;"),
      diffLine("remove", 2, null, "const count = _ctx.count;"),
      diffLine("add", null, 1, "return _openBlock(), _createElementBlock('main');"),
      diffLine("add", null, 2, "const title = title;"),
      diffLine("add", null, 3, "const count = count;"),
    ]);

    expect(
      rows.map((row) => ({
        left: row.left.tokens[0]?.content ?? "",
        right: row.right.tokens[0]?.content ?? "",
      })),
    ).toEqual([
      {
        left: "",
        right: "return _openBlock(), _createElementBlock('main');",
      },
      {
        left: "const title = _ctx.title;",
        right: "const title = title;",
      },
      {
        left: "const count = _ctx.count;",
        right: "const count = count;",
      },
    ]);
  });

  it("does not pair unrelated changed lines", () => {
    const rows = buildSplitDiffRows([
      diffLine("remove", 1, null, "const title = _ctx.title;"),
      diffLine("add", null, 1, "return _renderSlot(_ctx.$slots, 'default');"),
    ]);

    expect(rows).toHaveLength(2);
    expect(rows[0]!.left.kind).toBe("remove");
    expect(rows[0]!.right.kind).toBe("empty");
    expect(rows[1]!.left.kind).toBe("empty");
    expect(rows[1]!.right.kind).toBe("add");
  });

  it("does not pair a full line with a short matching fragment", () => {
    const rows = buildSplitDiffRows([
      diffLine("remove", 1, null, "import { computed, watch } from 'vue'"),
      diffLine("add", null, 1, "} from 'vue'"),
    ]);

    expect(rows).toHaveLength(2);
    expect(rows[0]!.left.kind).toBe("remove");
    expect(rows[0]!.right.kind).toBe("empty");
    expect(rows[1]!.left.kind).toBe("empty");
    expect(rows[1]!.right.kind).toBe("add");
  });
});
