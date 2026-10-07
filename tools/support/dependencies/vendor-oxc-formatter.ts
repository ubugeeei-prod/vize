// Reproduce the source-only import from exact official Git objects.
// Usage: node tools/support/dependencies/vendor-oxc-formatter.ts OXC_CLONE
import { spawnSync } from "node:child_process";
import { mkdirSync, existsSync, writeFileSync } from "node:fs";
import path from "node:path";

const revision = "fc702c1fa9f0412d06ec6908b58cd395b826cf7f";
const source = process.argv[2];
if (!source) throw new Error("Supply the verified OXC source clone");
const target = path.resolve("vendor/oxc_formatter");
if (existsSync(target)) throw new Error("Refusing to overwrite existing vendor source");
function git(...args: string[]): Buffer {
  const result = spawnSync("git", ["-C", source, ...args], { maxBuffer: 8 * 1024 * 1024 });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(result.stderr.toString());
  return result.stdout;
}
const entries = git(
  "ls-tree",
  "-rz",
  revision,
  "--",
  "crates/oxc_formatter/src",
  "crates/oxc_formatter/Cargo.toml",
  "LICENSE",
)
  .toString()
  .split("\0")
  .filter(Boolean);
let bytes = 0;
for (const entry of entries) {
  const match = /^(100644|100755) blob ([a-f0-9]{40})\t(.+)$/.exec(entry);
  if (!match) throw new Error(`Unsupported source object: ${entry}`);
  const [, , oid, upstreamPath] = match;
  const relative =
    upstreamPath === "LICENSE" ? "LICENSE" : upstreamPath.slice("crates/oxc_formatter/".length);
  const file = path.join(target, relative);
  mkdirSync(path.dirname(file), { recursive: true });
  const content = git("cat-file", "blob", oid);
  writeFileSync(file, content);
  bytes += content.length;
}
console.log(`Copied ${entries.length} official blobs (${bytes} bytes) from ${revision}`);
