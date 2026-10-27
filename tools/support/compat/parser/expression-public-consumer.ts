import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

type Control = { packet: string; profile: string; source: string };
const root = resolve(process.env.GITHUB_WORKSPACE ?? process.cwd());
const version = process.env.VIZE_EXPRESSION_PUBLIC_VERSION!;
assert(/^0\.\d+\.\d+$/.test(version));
const proof = resolve(process.env.VIZE_EXPRESSION_PUBLIC_PROOF!);
const project = join(proof, "consumer");
const cargoHome = join(proof, "cargo-home");
const target = join(proof, "cargo-target");
const sha = (bytes: Uint8Array | string) => createHash("sha256").update(bytes).digest("hex");
const json = (path: string, value: unknown) =>
  writeFileSync(path, JSON.stringify(value, null, 2) + "\n");
const archivePath = join(
  root,
  "tests/fuzz/archives/issue-10845-public-expression-packets.json.txt",
);
const archive = readFileSync(archivePath);
assert.equal(sha(archive), "104542bf241094ee2a8d54ae32162ca16a3c648543085fdb32f9fc27a93b8576");
const controls = JSON.parse(archive.toString()) as Control[];
assert.equal(controls.length, 91);
const source = readFileSync(
  join(root, "tests/fuzz/regressions/js_ts_expression/issue-10845.input"),
);
assert.equal(source.length, 23857);
assert.equal(sha(source), "60c49daaab7e06a062b0ffa7dced0eb4ddab9dbee1b52f1657fc4f1c08aa14a5");
mkdirSync(join(project, "src"), { recursive: true });
mkdirSync(cargoHome);
mkdirSync(target);
const raw = (text: string) => {
  let fence = "#";
  while (text.includes('"' + fence)) fence += "#";
  return "r" + fence + '"' + text + '"' + fence;
};
const cases = controls.slice(0, -1).map((control) => {
  assert(["ts", "tsx", "mjs", "jsx"].includes(control.profile));
  return (
    "        (SourceType::" +
    control.profile +
    "(), " +
    raw(control.source) +
    ", " +
    raw(control.packet) +
    "),"
  );
});
writeFileSync(join(project, "original.input"), source);
writeFileSync(join(project, "expected-expression.txt"), controls.at(-1)!.packet);
writeFileSync(
  join(project, "Cargo.toml"),
  '[package]\nname = "public-expression-overlap-acceptance"\nversion = "0.0.0"\nedition = "2024"\n\n[workspace]\n\n[dependencies]\noxc_parser = { package = "vize_oxc_parser", version = "=' +
    version +
    '" }\noxc_allocator = "=0.142.0"\noxc_span = "=0.142.0"\n',
);
writeFileSync(
  join(project, "src/main.rs"),
  [
    "use oxc_allocator::Allocator;",
    "use oxc_parser::Parser;",
    "use oxc_span::SourceType;",
    "use std::time::Instant;",
    "",
    "fn main() {",
    '    let source = include_str!("../original.input");',
    "    let allocator = Allocator::default();",
    "    let start = Instant::now();",
    "    let result = Parser::new(&allocator, source, SourceType::ts()).parse_expression();",
    "    let elapsed = start.elapsed();",
    "    assert!(result.is_err());",
    '    assert_eq!(format!("{result:?}"), include_str!("../expected-expression.txt"));',
    "    let arena_bytes = allocator.used_bytes();",
    '    assert!(arena_bytes < 16 * 1024 * 1024, "repeated allocation returned: {arena_bytes}");',
    "    let controls = [",
    ...cases,
    "    ];",
    "    for (source_type, source, expected) in controls {",
    "        let allocator = Allocator::default();",
    "        let result = Parser::new(&allocator, source, source_type).parse_expression();",
    '        assert_eq!(format!("{result:?}"), expected, "{source_type:?}: {source}");',
    "    }",
    '    println!("{{\\"pass\\":true,\\"controlPackets\\":90,\\"arenaBytes\\":{arena_bytes},\\"parserPhaseNs\\":{}}}", elapsed.as_nanos());',
    "}",
    "",
  ].join("\n"),
);
const env = {
  ...process.env,
  CARGO_HOME: cargoHome,
  CARGO_TARGET_DIR: target,
  CARGO_BUILD_JOBS: "2",
  RAYON_NUM_THREADS: "2",
};
function cargo(name: string, args: string[]) {
  const result = spawnSync(
    "cargo",
    ["+1.99.1", ...args, "--manifest-path", join(project, "Cargo.toml")],
    { cwd: project, env, encoding: "utf8", timeout: 900_000, maxBuffer: 32 * 1024 * 1024 },
  );
  json(join(proof, name + ".json"), {
    args,
    status: result.status,
    signal: result.signal,
    error: result.error?.message,
    stdout: result.stdout,
    stderr: result.stderr,
  });
  assert.equal(result.status, 0, result.stderr || result.error?.message);
  return result.stdout;
}
cargo("lock", ["generate-lockfile"]);
const metadata = JSON.parse(
  cargo("metadata", ["metadata", "--locked", "--format-version", "1"]),
) as {
  packages: {
    id: string;
    name: string;
    version: string;
    source: string | null;
    manifest_path: string;
  }[];
};
for (const [name, expected] of [
  ["vize_oxc_parser", version],
  ["oxc_allocator", "0.142.0"],
  ["oxc_span", "0.142.0"],
]) {
  const packages = metadata.packages.filter(
    (entry) => entry.name === name && entry.version === expected,
  );
  assert.equal(packages.length, 1, name);
  assert.equal(
    packages[0]!.source,
    "registry+https://github.com/rust-lang/crates.io-index",
    name + " must be the real registry package",
  );
}
for (const entry of metadata.packages) {
  if (entry.name === "public-expression-overlap-acceptance") continue;
  assert.equal(entry.source, "registry+https://github.com/rust-lang/crates.io-index", entry.id);
}
const stdout = cargo("run", ["run", "--locked", "--release"]);
const result = JSON.parse(stdout.trim()) as {
  pass: boolean;
  controlPackets: number;
  arenaBytes: number;
  parserPhaseNs: number;
};
assert.equal(result.pass, true);
assert.equal(result.controlPackets, 90);
assert(result.arenaBytes < 16 * 1024 * 1024);
json(join(proof, "acceptance.json"), {
  version,
  source: process.env.SOURCE_SHA,
  inputSha256: sha(source),
  packetArchiveSha256: sha(archive),
  cargoLockSha256: sha(readFileSync(join(project, "Cargo.lock"))),
  cargoBuildJobs: 2,
  patchSections: false,
  freshCargoHome: true,
  result,
  historicalObserverLabel: controls.at(-1)!.source,
  note: "The predecessor observer label is preserved byte for byte. This run supplies the exact authenticated issue-10845 input and compares the entire diagnostic packet. Time is an observation, not a cold project guarantee.",
});
console.log(JSON.stringify({ version, ...result }));
