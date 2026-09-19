import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";

const { waitFor } = createRequire(import.meta.url)("../../editors/vscode/test/suite/wait-for.cjs");

test("editor host deadlines bound a provider that never responds", { timeout: 5_000 }, async () => {
  await assert.rejects(
    waitFor(
      () => new Promise(() => {}),
      () => true,
      "semantic tokens",
      25,
    ),
    {
      name: "AssertionError",
      message: "semantic tokens did not happen within 25 ms",
    },
  );
});

test("editor host retries retain one overall deadline", { timeout: 5_000 }, async () => {
  let attempts = 0;
  await assert.rejects(
    waitFor(
      () => ++attempts,
      () => false,
      "hover",
      250,
    ),
    {
      name: "AssertionError",
      message: "hover did not happen within 250 ms",
    },
  );
  assert.ok(attempts >= 1);
});

test("editor host polling returns provider results and preserves failures", async () => {
  const result = { data: [0, 0, 4, 1, 0] };
  assert.equal(
    await waitFor(
      () => result,
      (value: unknown) => value === result,
      "tokens",
      1_000,
    ),
    result,
  );
  const failure = new Error("provider failed");
  await assert.rejects(
    waitFor(
      () => {
        throw failure;
      },
      () => true,
      "tokens",
      1_000,
    ),
    failure,
  );
});
