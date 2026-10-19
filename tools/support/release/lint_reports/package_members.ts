// Compare every installed regular member to an authenticated registry archive.
// usage: node package_members.ts ARCHIVE INSTALLED_DIRECTORY
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { readTarGz } from "../../archive/tar.ts";

interface MemberRecord {
  readonly bytes: number;
  readonly path: string;
  readonly sha256: string;
}

/** POSIX path parts as Python's PurePosixPath spells them. */
function parts(name: string): string[] {
  return name.split("/").filter((part) => part !== "" && part !== ".");
}

function files(directory: string, prefix = ""): string[] {
  const found: string[] = [];
  for (const entry of fs.readdirSync(path.join(directory, prefix), { withFileTypes: true })) {
    const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) found.push(...files(directory, relative));
    else if (fs.statSync(path.join(directory, relative), { throwIfNoEntry: false })?.isFile())
      found.push(relative);
  }
  return found;
}

export function packageMembers(archive: string, directory: string): MemberRecord[] {
  const root = fs.realpathSync(directory);
  assert.ok(fs.statSync(root).isDirectory());
  const records: MemberRecord[] = [];
  const seen = new Set<string>();
  for (const member of readTarGz(fs.readFileSync(archive))) {
    const name = parts(member.name);
    assert.ok(name[0] === "package" && !name.includes(".."));
    assert.ok(!member.name.startsWith("/"));
    if (member.kind === "directory") continue;
    assert.equal(member.kind, "file", "registry links and special members refused");
    const relative = name.slice(1).join("/");
    assert.ok(relative && !seen.has(relative));
    seen.add(relative);
    const target = path.join(root, relative);
    const stat = fs.lstatSync(target, { throwIfNoEntry: false });
    assert.ok(stat?.isFile());
    assert.ok(fs.realpathSync(target).startsWith(root + path.sep));
    assert.ok(
      fs.readFileSync(target).equals(member.data),
      `installed package differs: ${relative}`,
    );
    records.push({
      bytes: member.data.length,
      path: relative,
      sha256: createHash("sha256").update(member.data).digest("hex"),
    });
  }
  assert.ok(records.length > 0);
  // npm archives may own bundled node_modules members. Keep unrelated installed
  // dependencies outside this package's archive scope, but include every physical
  // file under a bundled package root so injected bundle files cannot disappear.
  const bundles = new Set<string>();
  for (const name of seen) {
    const segments = name.split("/");
    const index = segments.indexOf("node_modules") + 1;
    if (index === 0) continue;
    const count = segments[index]?.startsWith("@") ? 2 : 1;
    bundles.add(segments.slice(0, index + count).join("/"));
  }
  const actual = new Set<string>();
  for (const relative of files(root)) {
    const owned =
      !relative.split("/").includes("node_modules") ||
      [...bundles].some((bundle) => relative === bundle || relative.startsWith(`${bundle}/`));
    if (!owned) continue;
    assert.ok(
      !fs.lstatSync(path.join(root, relative)).isSymbolicLink(),
      "installed package links refused",
    );
    actual.add(relative);
  }
  assert.deepEqual(actual, seen, "extra installed package members refused");
  return records;
}

if (import.meta.main) {
  const [archive, directory] = process.argv.slice(2);
  assert.ok(archive && directory, "usage: package_members.ts ARCHIVE DIRECTORY");
  process.stdout.write(`${JSON.stringify(packageMembers(archive, directory))}\n`);
}
