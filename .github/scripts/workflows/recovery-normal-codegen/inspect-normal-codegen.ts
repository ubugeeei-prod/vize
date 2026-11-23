import assert from "node:assert/strict";
import { join } from "node:path";
import {
  requiredEnv,
  textFile,
  writeJson,
  fileSha256,
  fileIdentity,
  environment,
} from "../shared/files.ts";

import { closeSync, copyFileSync, openSync, writeSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { relative, isAbsolute } from "node:path";
import { cargoArtifacts } from "../shared/cargo-artifacts.ts";
const expected = {
  "base": "7c45cf385a6cb75d149f8c8f22a3901499eb64dc2292e295b635a89a5879fb9b",
  "head": "4546f3d4fa466b18845379b80715eac5f047cc499c92e7097c16c30356fa83b3",
};
const root = "codegen",
  recovery = "crates/vize_croquis/src/script_parser/recovery.rs",
  carrier = "davinci/vize_l0/examples/source_recovery_clone_benchmark.rs";
function inspectRole(role: keyof typeof expected) {
  const sourceRoot = join(requiredEnv("RUNNER_TEMP"), "recovery-codegen-" + role),
    binary = join(root, "normal-" + role);
  const recoverySha256 = fileSha256(join(sourceRoot, recovery)),
    lockSha256 = fileSha256(join(sourceRoot, "Cargo.lock")),
    carrierSha256 = fileSha256(join(sourceRoot, carrier));
  assert.equal(recoverySha256, expected[role]);
  assert.equal(lockSha256, "1b6040a4bedd2393261364237dc67474d436a7919d54bbdf7638133b2375010d");
  assert.equal(carrierSha256, "30d7478d54d0eccf631a3c8171beeef791d4acb75bdf5db8f228c6d8d3517404");
  const artifacts = cargoArtifacts(join(root, "build-" + role + ".jsonl")).filter((row) =>
    ["vize_croquis", "source_recovery_clone_benchmark"].includes(row.target.name),
  );
  assert.equal(artifacts.length, 2);
  for (const artifact of artifacts) {
    assert.equal(artifact.fresh, false);
    assert.equal(artifact.profile.opt_level, "3");
    assert.equal(artifact.profile.debuginfo, 0);
    assert.deepEqual(artifact.features, artifact.target.name === "vize_croquis" ? [] : ["default"]);
    const rel = relative(sourceRoot, artifact.manifest_path);
    assert.ok(!isAbsolute(rel) && rel !== ".." && !rel.startsWith("../"));
  }
  const symbols = textFile(join(root, "nm-" + role + ".txt"))
    .split("\n")
    .flatMap((line) => {
      const match = /^\s*([\da-fA-F]+)\s+(\S+)\s+(\S+)\s*$/u.exec(line);
      return match ? [{ address: parseInt(match[1]!, 16), kind: match[2]!, name: match[3]! }] : [];
    });
  const addresses = [...new Set(symbols.map((s) => s.address))].sort((a, b) => a - b);
  const nextAddress = new Map(
    addresses.slice(0, -1).map((start, index) => [start, addresses[index + 1]!]),
  );
  const assemblySymbols: { symbol: string; address: number; end: number }[] = [];
  const out = openSync(join(root, "assembly-" + role + ".txt"), "w");
  try {
    for (const { address, kind, name } of symbols) {
      if (
        !["T", "t"].includes(kind) ||
        !(
          /parse_program_for_analysis|incomplete_call_before_declaration|__rust_alloc|__rdl_alloc/u.test(
            name,
          ) ||
          (name.includes("source_recovery_clone_benchmark") && name.includes("main"))
        )
      )
        continue;
      const end = nextAddress.get(address);
      if (end === undefined) continue;
      writeSync(out, "\nSYMBOL " + name + "\n");
      execFileSync(
        "objdump",
        ["-d", "-C", "--start-address=" + address, "--stop-address=" + end, binary],
        { stdio: ["ignore", out, "pipe"] },
      );
      assemblySymbols.push({ symbol: name, address, end });
    }
  } finally {
    closeSync(out);
  }
  for (const [file, label] of [
    ["Cargo.toml", "cargo"],
    [carrier, "carrier"],
    [recovery, "recovery"],
  ] as const)
    copyFileSync(join(sourceRoot, file), join(root, label + "-" + role + ".txt"));
  const { sha256, bytes } = fileIdentity(binary);
  return {
    sourceRoot,
    source: textFile(join(root, role + ".sha")).trim(),
    binarySha256: sha256,
    binaryBytes: bytes,
    recoverySha256,
    lockSha256,
    carrierSha256,
    artifacts,
    assemblySymbols,
    recoveryOutOfLine: assemblySymbols.some(({ symbol }) =>
      symbol.includes("parse_program_for_analysis"),
    ),
    allocatorQualification:
      "inspect actual shim assembly and dynamic symbols; dependency compilation alone is not allocator selection",
    normalSpeedQualified: false,
    allocationTrace: null,
  };
}
const roles = { base: inspectRole("base"), head: inspectRole("head") };
assert.notEqual(roles.base.binarySha256, roles.head.binarySha256);
const receipt = {
  scope: "codegen-only normal rebuild; no timing, RSS, or allocation measurement",
  pinnedBase: requiredEnv("PINNED_BASE"),
  pinnedHead: requiredEnv("PINNED_HEAD"),
  runner: environment([
    "RUNNER_NAME",
    "RUNNER_OS",
    "RUNNER_ARCH",
    "RUNNER_ENVIRONMENT",
    "CARGO_BUILD_JOBS",
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_INCREMENTAL",
  ]),
  roles,
};
writeJson(join(root, "receipt.json"), receipt);
console.log(JSON.stringify(receipt, null, 2));
