import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const digest = (file) => {
  const hash = createHash("sha256"),
    buffer = Buffer.allocUnsafe(1024 * 1024),
    fd = fs.openSync(file, "r");
  try {
    for (let bytes; (bytes = fs.readSync(fd, buffer)) > 0;) hash.update(buffer.subarray(0, bytes));
  } finally {
    fs.closeSync(fd);
  }
  return hash.digest("hex");
};
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
export function selectPhaseDirectory(inventories) {
  // Host proc-macro dependencies may live alongside a target-triple build.
  // Only a directory containing the measured Atelier library can own a set.
  const owners = inventories.filter((row) =>
    row.entries.some((file) => file.startsWith("libvize_atelier_sfc-") && file.endsWith(".rlib")),
  );
  assert.equal(owners.length, 1, "one actual Atelier target directory required");
  return { directory: owners[0].directory, libraries: selectPhaseLibraries(owners[0].entries) };
}

export function phaseDependencyPlan(target, inventories, host) {
  const selected = selectPhaseDirectory(inventories);
  assert.ok(selected.directory.startsWith(target + path.sep));
  const relative = path.relative(target, selected.directory).split(path.sep);
  const native = relative.length === 2;
  assert.deepEqual(relative.slice(native ? 0 : 1), ["ci-opt", "deps"]);
  assert.equal(relative.length, native ? 2 : 3);
  const triple = native ? host : relative[0];
  assert.match(triple, /^[a-z0-9_]+(?:-[a-z0-9_]+)+$/u);
  const hostDirectory = path.join(target, "ci-opt/deps");
  assert.ok(
    inventories.some((row) => row.directory === hostDirectory),
    "actual host proc-macro dependency directory required",
  );
  return {
    ...selected,
    triple,
    searchDirectories: [...new Set([selected.directory, hostDirectory])],
  };
}

export function phaseOutputAbsent(file) {
  let absent = false;
  try {
    fs.lstatSync(file);
  } catch (error) {
    if (error.code === "ENOENT") absent = true;
    else throw error;
  }
  assert.equal(absent, true, "phase driver must start absent, including dangling links");
}
export function buildPhaseDriver({ image, root, producerRoot, destination, inputs, receipt }) {
  const target = fs.realpathSync(image.target);
  const roots = [
    target,
    ...fs
      .readdirSync(target, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => path.join(target, entry.name)),
  ];
  const inventories = roots
    .map((root) => path.join(root, "ci-opt/deps"))
    .filter((directory) => fs.existsSync(directory))
    .map((directory) => {
      directory = fs.realpathSync(directory);
      assert.ok(directory.startsWith(target + path.sep));
      return { directory, entries: fs.readdirSync(directory).sort() };
    });
  receipt.phaseDiscovery ??= [];
  receipt.phaseDiscovery.push({ side: image.side, source: image.source, target, inventories });
  write(path.join(destination, "receipt.json"), receipt);
  const toolchain = execFileSync("rustc", ["-Vv"], { cwd: producerRoot, encoding: "utf8" });
  const host = /^host: (.+)$/mu.exec(toolchain)?.[1];
  assert.ok(host);
  const { directory, libraries, triple, searchDirectories } = phaseDependencyPlan(
    target,
    inventories,
    host,
  );
  for (const file of Object.values(libraries))
    assert.equal(
      fs.lstatSync(path.join(directory, file)).isFile(),
      true,
      "actual role rlibs cannot be foreign symlinks",
    );
  const source = path.join(root, "tools/benchmarks/harness/css-source-map-phases.rs");
  const binary = path.join(destination, `phase-${image.side}-${image.source}`);
  phaseOutputAbsent(binary);
  const record = {
    side: image.side,
    source: image.source,
    imageSha256: image.binarySha256,
    imageToken: image.token,
    toolchain,
    producerRoot,
    triple,
    searchDirectories,
    searchDependencies: searchDirectories.flatMap((directory) =>
      fs
        .readdirSync(directory)
        .filter((file) => /\.(?:rlib|so)$/u.test(file))
        .sort()
        .map((file) => {
          file = path.join(directory, file);
          assert.equal(fs.lstatSync(file).isFile(), true);
          return { file, sha256: digest(file) };
        }),
    ),
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
    "--target",
    triple,
    "-C",
    "opt-level=3",
    "-C",
    "lto=thin",
    "-C",
    "codegen-units=16",
    ...searchDirectories.flatMap((directory) => ["-L", `dependency=${directory}`]),
    ...Object.entries(record.libraries).flatMap(([name, value]) => [
      "--extern",
      `${name}=${value.file}`,
    ]),
    "-o",
    binary,
  ];
  record.command = ["timeout", "--signal=TERM", "--kill-after=5s", "300s", "rustc", ...argv];
  record.compileStartedAt = new Date().toISOString();
  receipt.phaseDrivers ??= [];
  receipt.phaseDrivers.push(record);
  write(path.join(destination, "receipt.json"), receipt);
  const log = fs.openSync(path.join(destination, `${image.side}-phase-build.log`), "w");
  try {
    console.log(
      JSON.stringify({
        stage: "phase-driver-build-start",
        side: image.side,
        source: image.source,
        triple,
        searchDirectories,
      }),
    );
    execFileSync(record.command[0], record.command.slice(1), {
      cwd: producerRoot,
      stdio: ["ignore", log, log],
    });
  } catch (error) {
    record.compileFailure = { status: error.status, signal: error.signal, code: error.code };
    throw error;
  } finally {
    fs.closeSync(log);
    record.compileEndedAt = new Date().toISOString();
    record.compileLogSha256 = digest(path.join(destination, `${image.side}-phase-build.log`));
    write(path.join(destination, "receipt.json"), receipt);
  }
  console.log(
    JSON.stringify({
      stage: "phase-driver-build-complete",
      side: image.side,
      source: image.source,
    }),
  );
  record.binary = { file: binary, sha256: digest(binary) };
  write(path.join(destination, "receipt.json"), receipt);
  return record;
}
export function runPhaseDriver(driver, destination, index) {
  assert.equal(digest(driver.binary.file), driver.binary.sha256);
  assert.equal(digest(driver.driver.file), driver.driver.sha256);
  assert.equal(digest(driver.inputs.file), driver.inputs.sha256);
  for (const library of driver.searchDependencies)
    assert.equal(digest(library.file), library.sha256);
  const file = path.join(destination, `${index}-${driver.side}-phases.json`);
  phaseOutputAbsent(file);
  write(file, {
    schema: "vize.css.source-map.source-phases",
    version: 1,
    status: "attempt",
    source: driver.source,
    side: driver.side,
    index,
  });
  const log = fs.openSync(path.join(destination, `${index}-${driver.side}-phase-run.log`), "w");
  try {
    console.log(JSON.stringify({ stage: "phase-driver-run-start", side: driver.side, index }));
    execFileSync(
      "timeout",
      ["--signal=TERM", "--kill-after=5s", "60s", driver.binary.file, driver.inputs.file, file],
      { stdio: ["ignore", log, log] },
    );
  } finally {
    fs.closeSync(log);
  }
  const packet = JSON.parse(fs.readFileSync(file, "utf8"));
  assert.equal(packet.schema, "vize.css.source-map.source-phases");
  assert.equal(packet.status, "complete");
  assert.equal(packet.vectors.length, 3);
  for (const row of packet.vectors) {
    assert.equal(row.samples.length, 60);
    assert.equal(row.outputs.fullCompile.map, undefined);
  }
  return packet;
}
