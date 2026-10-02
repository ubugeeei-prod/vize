import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";

// Scoped root-discovery proof for the historical JSON/template moves only.
// Current selected Cargo compile/runtime remains a separate captured arm.
const [rootArg] = process.argv.slice(2);
assert(rootArg && process.argv.length === 3);
const root = fs.realpathSync(rootArg);
const { stripRust } = await import(
  pathToFileURL(path.join(root, "tools/support/compat/davinci/lib/rust-source.mjs")).href
);
const pin = (file: string) => {
  const bytes = fs.readFileSync(path.join(root, file));
  return {
    path: file,
    bytes: bytes.length,
    sha256: createHash("sha256").update(bytes).digest("hex"),
  };
};
const libPath = "crates/vize_glyph/src/lib.rs";
const libBytes = fs.readFileSync(path.join(root, libPath));
const source = libBytes.toString("utf8");
assert(Buffer.from(source).equals(libBytes));
const clean = stripRust(source);
assert(!/\b(?:r#)?path\s*=/.test(clean), "Glyph root must not override ordinary module discovery");
const modules = ["json", "template"].map((name) => {
  assert.equal([...clean.matchAll(new RegExp(`^mod ${name};$`, "gm"))].length, 1);
  const file = `crates/vize_glyph/src/${name}.rs`;
  assert(fs.statSync(path.join(root, file)).isFile());
  const oldOwner = `crates/vize_glyph/src/${name}/mod.rs`;
  assert(!fs.existsSync(path.join(root, oldOwner)), "old module owner must be absent");
  return {
    name,
    ...pin(file),
    ordinaryDeclaration: `mod ${name};`,
    oldOwner,
    oldModOwnerAbsent: true,
  };
});
console.log(
  JSON.stringify({
    schema: "vize.formatter.glyph-module-source",
    version: 1,
    lib: pin(libPath),
    modules,
    scope: "JSON/template root discovery only; not a whole-Glyph module-layout gate",
    publicOutputCredit: false,
    nativeCredit: false,
  }),
);
