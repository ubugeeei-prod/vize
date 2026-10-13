import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { qualifyEmptyVueStarHost } from "./warm-type-backed-empty-vue-star-host.ts";

export const emptyVueStarListingPath =
  "crates/vize_canon/tests/empty_vue_star_exports/execution.rs";
export const emptyVueStarListingBeforeSHA256 =
  "dea4749d405c1d43c9d42de9dc38f949dc454981d933f73d6bd2eb442837e904";
export const emptyVueStarListingAfterSHA256 =
  "8036d7f9eace7c439afcf23b785680f6e833ccfd673f2a9c6814735ca3ee3581";
const sha = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const blob = (body: Buffer) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const clauses = [
  [
    'let listing = support::cli(runtime, &config, &["--listFilesOnly"]);',
    'let listing = support::cli(runtime, &config, &["--listFilesOnly", "--pretty", "false"]);',
  ],
  [
    'assert_eq!(listing["arguments"], json!(["--listFilesOnly"]));',
    'assert_eq!(\n        listing["arguments"],\n        json!(["--listFilesOnly", "--pretty", "false"])\n    );',
  ],
] as const;

/** Only the complete current execution body can lend its exact two-clause predecessor. */
export function retainedEmptyVueStarListingExecution(body: Buffer): Buffer {
  assert(Buffer.isBuffer(body), "complete current listing execution required");
  assert.equal(body.length, 10843);
  assert.equal(sha(body), emptyVueStarListingAfterSHA256);
  assert.equal(blob(body), "dc0f5bf729911f9b457cb5f677f0fc7db116f807");
  let text = body.toString("utf8");
  assert.deepEqual(Buffer.from(text), body);
  for (const [before, after] of clauses) {
    assert.equal(text.split(after).length, 2, "one exact listing clause");
    text = text.replace(after, before);
  }
  const restored = Buffer.from(text);
  assert.equal(restored.length, 10779);
  assert.equal(sha(restored), emptyVueStarListingBeforeSHA256);
  assert.equal(blob(restored), "a3738887085a64457c09cff642b3776182ec9847");
  return restored;
}

/** Authenticate the sole new cell before the unchanged six-cell correction and 290-cell law. */
export function qualifyEmptyVueStarListingHost(
  production: string[],
  alreadyQualified: ReadonlySet<string>,
  digest: (side: "before" | "after", file: string) => string | null,
  original: typeof qualifyEmptyVueStarHost,
) {
  if (original !== qualifyEmptyVueStarHost) return null;
  if (digest("after", emptyVueStarListingPath) !== emptyVueStarListingAfterSHA256) return null;
  return original(production, alreadyQualified, (side, file) =>
    side === "after" && file === emptyVueStarListingPath
      ? emptyVueStarListingBeforeSHA256
      : digest(side, file),
  );
}
