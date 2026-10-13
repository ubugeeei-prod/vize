import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

const root = new URL("../../../", import.meta.url);
const directory = new URL(
  "tests/_fixtures/differential/lsp/native-lexical-composition-before/",
  root,
);
const metadata = JSON.parse(fs.readFileSync(new URL("entries.json", directory), "utf8")) as {
  version: number;
  entries: Array<{
    authority: "sdk" | "lexical";
    source: string;
    path: string;
    body: string;
    gitEntryHex: string;
    byteLength: number;
    sha256: string;
  }>;
};
const hash = (value: Buffer | string) => createHash("sha256").update(value).digest("hex");
assert.equal(
  hash(JSON.stringify(metadata)),
  "48a998a5450d542904cfe960f2351008f1f7887394b89690cb8e772747d7a9b9",
);
assert.equal(metadata.version, 1);
assert.equal(metadata.entries.length, 6);
const bodies = new Map<string, Buffer>();
for (const entry of metadata.entries) {
  assert.equal(
    entry.source,
    entry.authority === "sdk"
      ? "b4f7bab24f8ecbf4aa07bfd0fc60e8c7bcaf74a3"
      : "9708be553d7dcf5ae3a546bde6061e5630be02ba",
  );
  const body = fs.readFileSync(new URL(entry.body, directory));
  assert.equal(body.length, entry.byteLength);
  assert.equal(hash(body), entry.sha256);
  const blob = createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
  assert.deepEqual(
    Buffer.from(entry.gitEntryHex, "hex"),
    Buffer.from(`100644 blob ${blob}\t${entry.path}\0`),
  );
  const key = `${entry.authority}:${entry.path}`;
  assert.equal(bodies.has(key), false);
  bodies.set(key, body);
}

/** Replay only retained original whole bodies; this grants no current-source credit. */
export function historicalCompositionBody(authority: "sdk" | "lexical", file: string) {
  return bodies.get(`${authority}:${file}`) ?? fs.readFileSync(new URL(file, root));
}
