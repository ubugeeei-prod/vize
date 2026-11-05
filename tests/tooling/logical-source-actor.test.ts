import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import {
  decodeLogicalSourceActor,
  readLogicalSourceActor,
} from "./support/logical-source-actor.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const authority = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/typechecker/logical-workspace-overlay-revision-11145/source-authority.json",
    ),
    "utf8",
  ),
).production;
const actor = JSON.parse(
  gunzipSync(
    fs.readFileSync(
      path.join(root, "tests/_fixtures/tooling/logical-source-actor-11647/after.json.gz"),
    ),
  ).toString("utf8"),
);

test("sealed logical actor retains all eight original source images without Git object availability", () => {
  const images = readLogicalSourceActor(root, authority);
  assert.equal(images.size, 8);
  for (const entry of authority.files) {
    const bytes = images.get(entry.path)!;
    assert.equal(bytes.length, entry.afterBytes);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), entry.afterSha256);
  }
});

test("sealed logical actor refuses other revisions, identities, bytes and incomplete source sets", () => {
  for (const mutate of [
    (value: typeof actor) => (value.revision = "0".repeat(40)),
    (value: typeof actor) => value.files.pop(),
    (value: typeof actor) => value.files.push(value.files[0]),
    (value: typeof actor) => (value.files[1] = value.files[0]),
    (value: typeof actor) => (value.files[0].base64 = Buffer.from("changed").toString("base64")),
    (value: typeof actor) => (value.files[0].base64 += "="),
    (value: typeof actor) => (value.files[0].foreign = true),
  ]) {
    const changed = structuredClone(actor);
    mutate(changed);
    assert.throws(() => decodeLogicalSourceActor(changed, authority));
  }
});
