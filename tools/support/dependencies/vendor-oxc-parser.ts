// Reproduce the unmodified parser source from an exact official Git object.
// Usage: vp node tools/support/dependencies/vendor-oxc-parser.ts OXC_CLONE
import { spawnSync } from "node:child_process";
import { mkdirSync, existsSync, writeFileSync } from "node:fs";
import path from "node:path";

const revision = "fc702c1fa9f0412d06ec6908b58cd395b826cf7f";
const source = process.argv[2];
if (!source) throw new Error("Supply the verified OXC source clone");
const target = path.resolve("vendor/oxc_parser");
if (existsSync(target)) throw new Error("Refusing to overwrite existing vendor source");
function git(...args: string[]): Buffer {
  const result = spawnSync("git", ["-C", source, ...args], { maxBuffer: 8 * 1024 * 1024 });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(result.stderr.toString());
  return result.stdout;
}
const entries = git("ls-tree", "-rz", revision, "--", "crates/oxc_parser", "LICENSE")
  .toString()
  .split("\0")
  .filter(Boolean);
let bytes = 0;
for (const entry of entries) {
  const match = /^(100644|100755) blob ([a-f0-9]{40})\t(.+)$/.exec(entry);
  if (!match) throw new Error(`Unsupported source object: ${entry}`);
  const [, , oid, upstreamPath] = match;
  const relative =
    upstreamPath === "LICENSE" ? "LICENSE" : upstreamPath.slice("crates/oxc_parser/".length);
  const file = path.join(target, relative);
  mkdirSync(path.dirname(file), { recursive: true });
  const content = git("cat-file", "blob", oid);
  writeFileSync(file, content);
  bytes += content.length;
}
writeFileSync(
  path.join(target, "UPSTREAM.md"),
  `# Pinned OXC parser source

Copied without source changes from official OXC commit
\`${revision}\` (oxc_parser 0.142.0):
https://github.com/oxc-project/oxc/tree/${revision}/crates/oxc_parser

The adjacent LICENSE is the exact upstream MIT license at the same revision.
Reproduce this source-copy commit with:

\`vp node tools/support/dependencies/vendor-oxc-parser.ts VERIFIED_OXC_CLONE\`

Following commits normalize standalone manifest dependencies and apply the
reviewed PURE recovery correction. Supporting OXC crates stay at the original
official pinned revision and share one AST/allocator/span identity.
`,
);
console.log(`Copied ${entries.length} official blobs (${bytes} bytes) from ${revision}`);
