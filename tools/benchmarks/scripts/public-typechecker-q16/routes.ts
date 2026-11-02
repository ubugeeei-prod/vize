import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { existsSync, lstatSync, readFileSync, readlinkSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { asRecord, asString } from "../../../support/json-value.ts";
import {
  fileImage,
  inside,
  type FileImage,
} from "../../../support/release/computed_event_custody.ts";
import type { PackageBase } from "../../../support/release/human_reports_11252_11253_public/model.ts";

export type CommandBin = "tsrs" | "vue-tsc" | "vize";
export function sourceBin(root: string, name: CommandBin, files: FileImage[]) {
  const directory = realpathSync(join(root, "node_modules", name));
  const manifest = asRecord(JSON.parse(readFileSync(join(directory, "package.json"), "utf8")));
  const declared = typeof manifest.bin === "string" ? manifest.bin : asRecord(manifest.bin)[name];
  const target = realpathSync(join(directory, asString(declared)));
  assert(inside(directory, target));
  const owned = files.filter((file) => file.path === target);
  assert.equal(owned.length, 1);
  assert.deepEqual(fileImage(target), owned[0]);
  const path = join(root, "node_modules/.bin", name);
  assert(lstatSync(path).isSymbolicLink());
  assert.equal(realpathSync(path), target);
  return { path, target, link: readlinkSync(path), image: owned[0]! };
}
const tsrsPrograms: Record<string, string> = {
  "bin/tsrs": "9d7f26b2c80d4e45576f6e783675c5f624b055a15f94cb4089b9e9cc529722f3",
  "bin/package.json": "3d04369a44dc75f6a70dfc51080beb357eb39b395a2db0b6557ce8b9900ceec4",
  "lib/tsrs.js": "43641bc84d518ed2db46ebb82ea27d0982898c23c8f5a8c114478310551e61fa",
  "lib/getExePath.js": "a1b6b7fa30a9a97c3fc0653805233f73cd2045c0f432ab608b8e1df9c8854ab9",
  "lib/platform.js": "442eeeced9e7065ab6ed643063ce0def230c63fe32d425e8023a459a6307f877",
};
export function tsrsRoute(root: string, packages: PackageBase[]) {
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  const one = (name: string) => {
    const matches = packages.filter((pkg) => pkg.name === name);
    assert.equal(matches.length, 1);
    return matches[0]!;
  };
  const parent = one("tsrs"),
    target = one("@tsrs/linux-x64-gnu");
  assert.equal(parent.version, "0.20.1-ts7.1.0-beta");
  assert.equal(target.version, parent.version);
  const manifest = asRecord(
    JSON.parse(readFileSync(join(parent.directory, "package.json"), "utf8")),
  );
  assert.equal(manifest.type, "module");
  assert.equal(asRecord(manifest.optionalDependencies)[target.name], parent.version);
  for (const [path, sha256] of Object.entries(tsrsPrograms)) {
    const absolute = join(parent.directory, path),
      stat = lstatSync(absolute);
    assert(stat.isFile() && !stat.isSymbolicLink() && realpathSync(absolute) === absolute);
    const expected = parent.files.find((file) => file.path === absolute);
    assert(expected);
    assert.equal(expected.sha256, sha256);
    assert.deepEqual(fileImage(absolute), expected);
  }
  // The reviewed platform.js selects this readable GNU ldd branch without running ldd.
  const ldd = "/usr/bin/ldd",
    stat = lstatSync(ldd);
  assert(stat.isFile() && !stat.isSymbolicLink() && realpathSync(ldd) === ldd);
  assert(!readFileSync(ldd, "utf8").includes("musl"), "only source-proven GNU target admitted");
  const resolver = createRequire(join(parent.directory, "lib/getExePath.js"));
  assert.equal(
    resolver.resolve(target.name + "/package.json"),
    join(target.directory, "package.json"),
  );
  const executable = join(target.directory, "tsrs");
  assert(
    lstatSync(executable).isFile() &&
      !lstatSync(executable).isSymbolicLink() &&
      realpathSync(executable) === executable,
  );
  const expected = target.files.find((file) => file.path === executable);
  assert(expected);
  assert.deepEqual(fileImage(executable), expected);
  const executableStat = lstatSync(executable);
  assert((executableStat.mode & 0o111) !== 0);
  assert(!existsSync(join(root, "node_modules/.bin/node")), "local Node shadow refuses");
  const nodeCandidates = (process.env.PATH ?? "")
    .split(":")
    .map((directory) => join(directory, "node"))
    .filter((path) => existsSync(path));
  assert(nodeCandidates.length > 0);
  assert.equal(realpathSync(nodeCandidates[0]!), realpathSync(process.execPath));
  return {
    parent: parent.directory,
    target: target.directory,
    executable: expected,
    ldd: fileImage(ldd),
    node: fileImage(realpathSync(process.execPath)),
    sourceBranch: "GNU selected without execFile; launcher may execve or spawn",
    actualCompilerPid: null,
    actualExecveOrSpawn: "UNKNOWN",
    runtimeImageObserved: false,
  };
}
