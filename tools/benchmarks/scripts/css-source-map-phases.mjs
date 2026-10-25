import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const digest = (file) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const write = (file, value) => fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
const names = ["vize_atelier_sfc", "vize_croquis", "vize_carton", "serde_json"];
export function selectPhaseLibraries(entries) {
  return Object.fromEntries(
    names.map((name) => {
      const matches = entries.filter(
        (file) => file.startsWith(`lib${name}-`) && file.endsWith(".rlib"),
      );
      assert.equal(matches.length, 1, `one actual source-built ${name} rlib required`);
      return [name, matches[0]];
    }),
  );
}
export function phaseOutputAbsent(file) {
  assert.equal(fs.existsSync(file), false, "phase driver must start absent");
}
export function buildPhaseDriver({ image, root, destination, inputs, receipt }) {
  const directory = path.join(image.target, "ci-opt/deps");
  const libraries = selectPhaseLibraries(fs.readdirSync(directory));
  const source = path.join(root, "tools/benchmarks/harness/css-source-map-phases.rs");
  const binary = path.join(destination, `phase-${image.side}-${image.source}`);
  phaseOutputAbsent(binary);
  const record = {
    side: image.side,
    source: image.source,
    imageSha256: image.binarySha256,
    imageToken: image.token,
    toolchain: execFileSync("rustc", ["-Vv"], { encoding: "utf8" }),
    driver: { file: source, sha256: digest(source) },
    inputs: { file: inputs, sha256: digest(inputs) },
    outputInitiallyAbsent: true,
    libraries: Object.fromEntries(
      Object.entries(libraries).map(([name, file]) => [
        name,
        { file: path.join(directory, file), sha256: digest(path.join(directory, file)) },
      ]),
    ),
  };
  const argv = [
    "--edition=2024",
    source,
    "-C",
    "opt-level=3",
    "-C",
    "lto=thin",
    "-C",
    "codegen-units=16",
    "-L",
    `dependency=${directory}`,
    ...Object.entries(record.libraries).flatMap(([name, value]) => [
      "--extern",
      `${name}=${value.file}`,
    ]),
    "-o",
    binary,
  ];
  record.command = ["rustc", ...argv];
  receipt.phaseDrivers ??= [];
  receipt.phaseDrivers.push(record);
  write(path.join(destination, "receipt.json"), receipt);
  const log = fs.openSync(path.join(destination, `${image.side}-phase-build.log`), "w");
  try {
    execFileSync("rustc", argv, { cwd: root, stdio: ["ignore", log, log] });
  } finally {
    fs.closeSync(log);
  }
  record.binary = { file: binary, sha256: digest(binary) };
  write(path.join(destination, "receipt.json"), receipt);
  return record;
}
export function runPhaseDriver(driver, destination, index) {
  assert.equal(digest(driver.binary.file), driver.binary.sha256);
  assert.equal(digest(driver.driver.file), driver.driver.sha256);
  assert.equal(digest(driver.inputs.file), driver.inputs.sha256);
  for (const library of Object.values(driver.libraries))
    assert.equal(digest(library.file), library.sha256);
  const file = path.join(destination, `${index}-${driver.side}-phases.json`),
    log = fs.openSync(file, "wx");
  try {
    execFileSync(driver.binary.file, [driver.inputs.file], { stdio: ["ignore", log, "pipe"] });
  } finally {
    fs.closeSync(log);
  }
  const packet = JSON.parse(fs.readFileSync(file, "utf8"));
  assert.equal(packet.schema, "vize.css.source-map.source-phases");
  assert.equal(packet.vectors.length, 3);
  for (const row of packet.vectors) {
    assert.equal(row.samples.length, 60);
    assert.equal(row.outputs.fullCompile.map, undefined);
  }
  return packet;
}
