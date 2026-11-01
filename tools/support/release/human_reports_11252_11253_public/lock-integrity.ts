import assert from "node:assert/strict";
import { sha256 } from "./value.ts";

/** Read one canonical pnpm packages entry, not arbitrary YAML. Unknown shapes,
 * aliases, duplicates and missing integrity refuse; whole original bytes stay. */
export function lockPackageIntegrity(raw: Buffer, name: string, version: string) {
  assert.match(name, /^(?:@[a-z0-9._-]+\/)?[a-z0-9._-]+$/u);
  assert.match(version, /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/u);
  const source = new TextDecoder("utf-8", { fatal: true }).decode(raw);
  assert(!source.includes("\t") && !source.includes("\0"));
  const lines = source.replaceAll("\r\n", "\n").split("\n");
  const versions = lines.filter((line) => line.startsWith("lockfileVersion:"));
  assert.equal(versions.length, 1, "one lockfileVersion declaration required");
  assert.equal(
    lines.filter((line) => line === "packages:").length,
    1,
    "one packages declaration required",
  );
  assert.equal(
    source.split(/^--- *$/mu).filter((document) => document.trim()).length,
    1,
    "one canonical lock document supported",
  );
  assert(
    versions.length > 0 && versions.every((line) => /^lockfileVersion: (['"]?)9\.0\1$/u.test(line)),
    "unsupported pnpm lockfile version",
  );
  const key = name + "@" + version;
  const headers = [`  '${key}':`, `  ${JSON.stringify(key)}:`, `  ${key}:`];
  const blocks: string[][] = [];
  let packages = false;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (line === "packages:") {
      packages = true;
      continue;
    }
    if (/^[^\s#]/u.test(line)) packages = false;
    if (!packages || !headers.includes(line)) continue;
    const block = [line];
    for (let j = i + 1; j < lines.length; j++) {
      const next = lines[j]!;
      if (next && !next.startsWith("    ")) break;
      block.push(next);
    }
    blocks.push(block);
  }
  assert.equal(blocks.length, 1, `one exact locked package required: ${key}`);
  const block = blocks[0]!;
  const resolutions = block.filter((line) => line.startsWith("    resolution:"));
  assert.equal(resolutions.length, 1, "one canonical resolution required");
  const inline = /^    resolution: \{integrity: (['"]?)(sha512-[A-Za-z0-9+/]+={0,2})\1\}$/u.exec(
    resolutions[0]!,
  );
  let integrity = inline?.[2];
  if (!integrity && resolutions[0] === "    resolution:") {
    const fields = block.filter((line) => line.startsWith("      integrity:"));
    assert.equal(fields.length, 1);
    integrity = /^      integrity: (['"]?)(sha512-[A-Za-z0-9+/]+={0,2})\1$/u.exec(fields[0]!)?.[2];
  }
  assert(integrity, "unsupported or missing locked SHA512 resolution");
  assert(
    !block.some((line) => /^    (?:<<:|[^:]+: [*&!])/u.test(line)),
    "lock aliases/tags refuse",
  );
  const entry = block.join("\n");
  return { key, integrity, entry, entrySha256: sha256(entry), lockSha256: sha256(raw) };
}
