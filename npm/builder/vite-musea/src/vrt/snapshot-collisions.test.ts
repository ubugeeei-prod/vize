import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { parseArtFile } from "../cli/utils.ts";
import { assertUniqueSnapshotNames } from "./snapshot-collisions.ts";
import { buildSnapshotName } from "./utils.ts";

const fixtures = fileURLToPath(
  new URL("../../../../../tests/tooling/fixtures/musea/snapshot-collision/", import.meta.url),
);

void test("two genuine same-basename Arts retain separate native identities and cannot share a baseline", async () => {
  const arts = await Promise.all(
    ["left", "right"].map((side) => parseArtFile(path.join(fixtures, side, "Button.art.vue"))),
  );
  assert.ok(arts[0] && arts[1]);
  assert.notEqual(arts[0].path, arts[1].path);
  assert.deepEqual(
    arts.map((art) => art!.metadata.title),
    ["Left", "Right"],
  );
  assert.deepEqual(
    arts.map((art) => art!.variants[0].name),
    ["Default", "Default"],
  );
  assert.ok(arts[0].variants[0].template.includes("#0000ff"));
  assert.ok(arts[1].variants[0].template.includes("#ff0000"));
  const jobs = arts.map((art) => ({
    art: art!,
    variantName: "Default",
    viewport: { width: 200, height: 100, name: "small" },
  }));
  assert.equal(
    buildSnapshotName(arts[0].path, "Default", jobs[0].viewport),
    "Button--Default--small.png",
  );
  assert.throws(
    () => assertUniqueSnapshotNames(jobs),
    (error: unknown) =>
      error instanceof Error &&
      error.message.includes(arts[0]!.path) &&
      error.message.includes(arts[1]!.path),
  );
});
