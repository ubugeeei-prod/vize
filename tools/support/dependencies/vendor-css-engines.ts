// Import the checksum-authenticated, unchanged Rust sources for the CSS forks.
// Usage: vp node tools/support/dependencies/vendor-css-engines.ts LIGHTNING_CRATE PARCEL_CRATE
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

const inputs = [
  {
    name: "lightningcss",
    prefix: "lightningcss-1.0.0-alpha.72",
    checksum: "d31b760f96e8fdfe1d0c295e4bf76c503d6f15d2d470d53bd8cd1f7aa8c7d934",
    revision: "c6a0c3cebf3395635e61075d2c81a96a710d4910",
    roots: ["src", "tests", "examples", "benches"],
    files: [
      "Cargo.toml",
      "Cargo.toml.orig",
      "LICENSE",
      "README.md",
      "rustfmt.toml",
      ".cargo_vcs_info.json",
    ],
  },
  {
    name: "parcel_sourcemap",
    prefix: "parcel_sourcemap-2.1.1",
    checksum: "485b74d7218068b2b7c0e3ff12fbc61ae11d57cb5d8224f525bd304c6be05bbb",
    revision: "a136bd6ba9f1d3cded501ae824341be0a54c5ce3",
    roots: ["src"],
    files: ["Cargo.toml", "Cargo.toml.orig", ".cargo_vcs_info.json"],
  },
];
const archives = process.argv.slice(2);
if (archives.length !== inputs.length) throw new Error("Supply both original .crate archives");

function tar(args: string[]): string {
  const result = spawnSync("tar", args, { encoding: "utf8", maxBuffer: 8 * 1024 * 1024 });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(result.stderr);
  return result.stdout;
}

// Validate both inputs before creating either destination.
const plans = inputs.map((input, index) => {
  const archive = path.resolve(archives[index]);
  const bytes = readFileSync(archive);
  if (createHash("sha256").update(bytes).digest("hex") !== input.checksum) {
    throw new Error(`Archive checksum mismatch: ${input.name}`);
  }
  const target = path.resolve("vendor", input.name);
  if (existsSync(target)) throw new Error(`Refusing to overwrite ${target}`);
  const paths = tar(["-tzf", archive]).trim().split("\n");
  const selected = paths.filter((entry) => {
    if (!entry.startsWith(input.prefix + "/") || entry.split("/").includes("..")) {
      throw new Error(`Unexpected archive path: ${entry}`);
    }
    const relative = entry.slice(input.prefix.length + 1);
    return (
      input.files.includes(relative) || input.roots.some((root) => relative.startsWith(root + "/"))
    );
  });
  return { ...input, archive, target, selected };
});

for (const plan of plans) {
  const temporary = mkdtempSync(path.join(tmpdir(), "vize-css-vendor-import-"));
  try {
    tar(["-xzf", plan.archive, "-C", temporary, "--", ...plan.selected]);
    const source = path.join(temporary, plan.prefix);
    const vcs = JSON.parse(readFileSync(path.join(source, ".cargo_vcs_info.json"), "utf8"));
    if (vcs.git.sha1 !== plan.revision)
      throw new Error(`Archive source revision mismatch: ${plan.name}`);
    mkdirSync(plan.target, { recursive: true });
    const copied = plan.selected.map((entry) => {
      const relative = entry.slice(plan.prefix.length + 1);
      const from = path.join(source, relative);
      const to = path.join(plan.target, relative);
      mkdirSync(path.dirname(to), { recursive: true });
      cpSync(from, to);
      const bytes = readFileSync(from);
      return {
        path: relative,
        bytes: bytes.length,
        sha256: createHash("sha256").update(bytes).digest("hex"),
      };
    });
    writeFileSync(
      path.join(plan.target, "UPSTREAM-IMPORT.json"),
      JSON.stringify(
        {
          archive: plan.prefix + ".crate",
          archiveSha256: plan.checksum,
          upstreamRevision: plan.revision,
          files: copied,
        },
        null,
        2,
      ) + "\n",
    );
    console.log(
      JSON.stringify({ package: plan.name, copiedFiles: copied.length, revision: plan.revision }),
    );
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
}
