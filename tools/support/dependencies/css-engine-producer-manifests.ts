// Only the two reviewed release-inheritance edits extend the frozen producer authority.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

const checksum = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const continuations: Record<
  string,
  { original: string; current: string; fragment: string; inverse: string }
> = {
  "Cargo.toml": {
    original: "c8fe5d4b91f49524fb4f8cd06c7453affd936b2373c29b622e94a01dc7a0793f",
    current: "38babfd635c5af9df1d401b0b59923940310a405868cbb0772d9bfefbd7e6e68",
    fragment:
      'parcel_sourcemap = { package = "vize_parcel_sourcemap", version = "=0.443.0", path = "vendor/parcel_sourcemap" }\n',
    inverse: "",
  },
  "vendor/lightningcss/Cargo.toml": {
    original: "21486cbc79ea73caf098501beef644a8d1a64f55706a9dc24b9e2fc4df4044aa",
    current: "4aa6d8ae5cc0c5525f67ccc16fee2c91e7b9cbd81b2159ff7c82704dae0da501",
    fragment: "[dependencies.parcel_sourcemap]\nworkspace = true\n",
    inverse:
      '[dependencies.parcel_sourcemap]\npackage = "vize_parcel_sourcemap"\nversion = "=0.443.0"\npath = "../parcel_sourcemap"\n',
  },
};

export function authenticateProducerSource(
  sourcePath: string,
  bytes: Buffer,
  originalSha256: string,
) {
  const actualSha256 = checksum(bytes);
  const continuation = Object.hasOwn(continuations, sourcePath)
    ? continuations[sourcePath]
    : undefined;
  if (continuation) assert.equal(originalSha256, continuation.original, sourcePath);
  if (actualSha256 === originalSha256) {
    return { path: sourcePath, actualSha256, originalSha256, reviewedContinuation: false };
  }
  assert(continuation, `unreviewed producer input: ${sourcePath}`);
  assert.equal(actualSha256, continuation.current, sourcePath);
  const text = bytes.toString("utf8");
  assert.equal(text.split(continuation.fragment).length, 2, sourcePath);
  const inverse = Buffer.from(text.replace(continuation.fragment, continuation.inverse));
  assert.equal(checksum(inverse), originalSha256, sourcePath);
  return { path: sourcePath, actualSha256, originalSha256, reviewedContinuation: true };
}
