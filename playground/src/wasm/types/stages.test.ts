import { describe, expect, it } from "vite-plus/test";
import { layerId } from "./stages";

describe("layerId", () => {
  it("maps stage ids to display layers", () => {
    expect(
      ["s0", "s1", "s2-plan", "s3-values", "s4", "s2-to-s3", "custom", "custom-s2", "l2"].map(
        layerId,
      ),
    ).toEqual(["l0", "l1", "l2-plan", "l3-values", "l4", "l2-to-l3", "custom", "custom-s2", "l2"]);
  });
});
