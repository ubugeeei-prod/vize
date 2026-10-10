import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import type { Page } from "playwright";
import type { ReplacementFixture } from "./palette-replacement.browser-contracts";

export async function checkPaletteReplacement(
  page: Page,
  repository: string,
  observations: unknown[],
) {
  const fixture: ReplacementFixture = JSON.parse(
    await readFile(
      path.join(repository, "tests/_fixtures/differential/musea/palette-replacement.json"),
      "utf8",
    ),
  );
  const received = await page.evaluate(async (input) => {
    const moduleUrl = "/__musea__/components/palette-replacement.browser-contracts.ts";
    const { runPaletteReplacement } = (await import(
      moduleUrl
    )) as typeof import("./palette-replacement.browser-contracts");
    return runPaletteReplacement(input);
  }, fixture);
  observations.push({ panel: "palette-replacement", fixture, received });
  const initial = {
    controls: fixture.palette.controls,
    values: { label: "Original label", tone: "primary" },
    customProps: [],
    deletedPaletteProps: [],
  };
  assert.deepEqual(received.undeleted, initial);
  const replacement = {
    controls: [
      fixture.palette.controls[0],
      { ...fixture.replacement, required: false, options: [] },
    ],
    values: { label: "Original label", tone: fixture.replacement.default_value },
    customProps: [fixture.replacement],
    deletedPaletteProps: ["tone"],
  };
  assert.deepEqual(received.replacedDefault, replacement);
  const edited = { ...replacement, values: { label: "Original label", tone: fixture.editedValue } };
  assert.deepEqual(received.replacedEdit, edited);
  assert.deepEqual(received.saved, {
    version: 1,
    values: edited.values,
    customProps: edited.customProps,
    deletedPaletteProps: edited.deletedPaletteProps,
  });
  assert.deepEqual(received.afterReload, edited);
  const removed = {
    controls: [fixture.palette.controls[0]],
    values: { label: "Original label" },
    customProps: [],
    deletedPaletteProps: ["tone"],
  };
  assert.deepEqual(received.afterRemove, removed);
  assert.deepEqual(received.afterRemovedReload, removed);
  assert.deepEqual(received.afterReset, initial);
}
