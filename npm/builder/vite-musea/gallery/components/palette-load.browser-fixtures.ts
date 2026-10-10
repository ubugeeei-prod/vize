import assert from "node:assert/strict";
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Page } from "playwright";
import type { Fixture } from "./palette-load.browser-contracts";

export async function checkPaletteLoadOrder(page: Page, repository: string, output: string) {
  const fixture: Fixture = JSON.parse(
    await readFile(
      path.join(repository, "tests/_fixtures/differential/musea/palette-load-order.json"),
      "utf8",
    ),
  );
  const observations = await page.evaluate(async (input) => {
    const moduleUrl = "/__musea__/components/palette-load.browser-contracts.ts";
    const { runPaletteLoadOrder } = (await import(
      moduleUrl
    )) as typeof import("./palette-load.browser-contracts");
    return runPaletteLoadOrder(input);
  }, fixture);
  await writeFile(
    path.join(output, "palette-load-order.json"),
    JSON.stringify({ fixture, observations }, null, 2),
  );
  assert.deepEqual(observations.beforeOldSuccess, {
    palette: fixture.current,
    values: { label: "Selected edit", tone: "primary", extra: 7 },
    customProps: [{ name: "extra", control: "number", default_value: 7 }],
    deletedPaletteProps: ["tone"],
    loading: false,
    error: null,
  });
  assert.deepEqual(observations.afterOldSuccess, observations.beforeOldSuccess);
  assert.deepEqual(observations.afterOldError, observations.beforeOldError);
  assert.deepEqual(observations.whileCurrentPending, {
    palette: null,
    values: {},
    customProps: [],
    deletedPaletteProps: [],
    loading: true,
    error: null,
  });
  assert.equal(observations.afterCurrentPending.palette.title, fixture.current.title);
  assert.equal(observations.afterCurrentPending.loading, false);
  assert.equal(observations.beforeObsolete.error, "API error: 503 Unavailable");
  assert.equal(observations.beforeObsolete.palette, null);
  assert.equal(observations.beforeObsolete.loading, false);
  assert.deepEqual(observations.afterObsolete, observations.beforeObsolete);
  assert.equal(observations.beforeRepeated.values.label, "Newest visit edit");
  assert.deepEqual(observations.afterRepeated, observations.beforeRepeated);
  assert.equal(observations.requests.length, 11);
  return { fixture, observations };
}
