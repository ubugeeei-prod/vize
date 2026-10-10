import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import type { ReplacementFixture } from "../gallery/components/palette-replacement.browser-contracts";
import { restorePaletteState } from "../gallery/composables/paletteState";

void test("saved custom replacement retains its control and value over a deleted palette prop", async () => {
  const fixture: ReplacementFixture = JSON.parse(
    await readFile(
      new URL(
        "../../../../tests/_fixtures/differential/musea/palette-replacement.json",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  const restored = restorePaletteState(fixture.palette.controls, {
    version: 1,
    values: { label: "Original label", tone: fixture.editedValue },
    customProps: [fixture.replacement],
    deletedPaletteProps: [fixture.replacement.name],
  });
  assert.deepEqual(restored.customProps, [fixture.replacement]);
  assert.deepEqual(restored.values, { label: "Original label", tone: fixture.editedValue });
  assert.deepEqual([...restored.deletedPaletteProps], [fixture.replacement.name]);
});
