// Inspect one public CLI member before writing a self-contained GNU consumer.
// usage: node archive.ts ARCHIVE FRESH_DESTINATION
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { readTarGz } from "../../archive/tar.ts";

export interface ExtractedMember {
  readonly member: string;
  readonly bytes: number;
  readonly mode: number;
  readonly sha256: string;
  readonly binary: string;
}

const elfX8664 = (data: Buffer): boolean =>
  data.subarray(0, 6).equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01])) &&
  data.subarray(18, 20).equals(Buffer.from([0x3e, 0x00]));

export function extract(archive: string, destination: string): ExtractedMember {
  if (fs.existsSync(destination)) throw new Error("public distribution destination must be fresh");
  const members = readTarGz(fs.readFileSync(archive));
  const [member] = members;
  if (members.length !== 1 || member?.name !== "vize" || member.kind !== "file")
    throw new Error("exactly one regular vize archive member required");
  if (member.size < 64 || member.size > 256 * 1024 * 1024 || !(member.mode & 0o111))
    throw new Error("bounded executable CLI member required");
  const data = member.data;
  if (data.length !== member.size) throw new Error("complete declared CLI member required");
  if (!elfX8664(data)) throw new Error("actual Linux x86_64 ELF member required");
  fs.mkdirSync(destination, { mode: 0o700 });
  const binary = path.join(destination, "vize");
  fs.writeFileSync(binary, data, { flag: "wx" });
  fs.chmodSync(binary, member.mode & 0o777);
  return {
    member: member.name,
    bytes: data.length,
    mode: member.mode & 0o777,
    sha256: createHash("sha256").update(data).digest("hex"),
    binary,
  };
}

if (import.meta.main) {
  const args = process.argv.slice(2);
  const [archive, destination] = args;
  if (args.length !== 2 || !archive || !destination) {
    process.stderr.write("usage: archive.ts ARCHIVE FRESH_DESTINATION\n");
    process.exitCode = 1;
  } else {
    process.stdout.write(`${JSON.stringify(extract(archive, destination))}\n`);
  }
}
