// usage: node strings-through-rename.ts <repo> <args-file> <base.ll> <head.ll>
// Checks that a release IR difference is only the codename rename's string spellings: every base
// byte-string constant is rewritten through the rename (bytes and array length), symbol names are
// relocated with the same --rename pairs ir-identity uses, and each remaining differing line may
// differ from its counterpart only in integer constants (the renamed strings' lengths).
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { bodyDiff, compareModules, normalizeModule } from "./ir-normalize.ts";

const [repo, argsFile, baseFile, headFile] = process.argv.slice(2);
const rename = await import(
  pathToFileURL(path.join(repo, "tools/support/levels/rename-codenames.ts")).href
);
const identifiers = new Map<string, string>();
for (const [, from, to] of fs
  .readFileSync(argsFile, "utf8")
  .matchAll(/--rename ([A-Za-z0-9_]+)=([A-Za-z0-9_]+)/gu))
  identifiers.set(from, to);
const word = /[A-Za-z_][A-Za-z0-9_]*/gu;
const relocate = (name: string) => name.replace(word, (w) => identifiers.get(w) ?? w);

const decode = (body: string) =>
  Buffer.from(
    body.replace(/\\([0-9A-Fa-f]{2})|\\\\/gu, (m, hex) =>
      hex ? String.fromCharCode(parseInt(hex, 16)) : "\\",
    ),
    "latin1",
  );
const encode = (bytes: Buffer) =>
  [...bytes]
    .map((b) =>
      b >= 0x20 && b < 0x7f && b !== 0x22 && b !== 0x5c
        ? String.fromCharCode(b)
        : `\\${b.toString(16).toUpperCase().padStart(2, "0")}`,
    )
    .join("");
let rewritten = 0;
const base = fs
  .readFileSync(baseFile, "utf8")
  .replace(/\[(\d+) x i8\] c"((?:[^"\\]|\\[0-9A-Fa-f]{2}|\\\\)*)"/gu, (whole, n, body) => {
    const text = decode(body).toString("utf8");
    const next = rename.rewriteText("crates/x/src/lib.rs", text);
    if (next === text) return whole;
    rewritten++;
    const bytes = Buffer.from(next, "utf8");
    return `[${bytes.length} x i8] c"${encode(bytes)}"`;
  })
  // the array type of a rewritten string also appears in its global's declared type
  ;
const head = fs.readFileSync(headFile, "utf8");
const b = normalizeModule(base, relocate);
const h = normalizeModule(head, relocate);
const diff = compareModules(b, h);
let lengthOnly = 0;
const other: string[] = [];
for (const name of diff.changed) {
  const lines = bodyDiff(b, h, name, 10_000);
  let ok = lines.length > 0;
  for (let i = 0; i + 1 < lines.length; i += 2) {
    const strip = (line: string) => line.slice(2).replace(/-?\b\d+\b/gu, "N");
    if (strip(lines[i]) !== strip(lines[i + 1])) ok = false;
  }
  if (ok) lengthOnly++;
  else other.push(name);
}
console.log(
  JSON.stringify(
    {
      rewrittenStrings: rewritten,
      identical: diff.identical,
      changedLengthOnly: lengthOnly,
      changedOther: other.length,
      onlyBase: diff.onlyBase.length,
      onlyHead: diff.onlyHead.length,
    },
  ),
);
for (const name of other.slice(0, 20)) {
  console.log(`changed: ${name}`);
  for (const line of bodyDiff(b, h, name, 3)) console.log(`  ${line.slice(0, 200)}`);
}
for (const name of diff.onlyBase.slice(0, 20)) console.log(`only base: ${name}`);
for (const name of diff.onlyHead.slice(0, 20)) console.log(`only head: ${name}`);
