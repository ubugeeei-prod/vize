import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../../../tests/differential/build-receipt.ts";
import {
  CONFIG_NAMES,
  PROFILES,
  REVISION,
  SUITE_HASH,
  loadUpstream,
  machineDiagnostics,
  relativeFiles,
  sha256,
  verifyConfigs,
  verifyUpstream,
  verifyWorkInventory,
} from "./vue-benchmarks-current-lint-contract.mjs";

const root = fileURLToPath(new URL("../../../", import.meta.url));

export async function probeCurrentLint(repoRoot = root) {
  const destination = path.join(repoRoot, "target/differential/vue-benchmarks-current-lint");
  fs.mkdirSync(destination, { recursive: true });
  const observation = {
    schema: "vize.benchmark.current-lint-observation",
    version: 1,
    upstreamRevision: REVISION,
    suiteHash: SUITE_HASH,
    publishedVizeVersion: "0.429.1",
    ranked: false,
    nativeMigrationCredit: 0,
    status: "UNKNOWN",
    attempts: [],
  };
  let workspace;
  try {
    const expected = expectedBuildIdentity(repoRoot);
    const binary = path.join(repoRoot, expected.binaryPath);
    const receipt = JSON.parse(fs.readFileSync(`${binary}.differential-build.json`, "utf8"));
    validateBuildReceipt(receipt, expected);
    observation.build = receipt;
    const schemaSource = "npm/cli/schemas/vize.config.schema.json";
    const schemaBytes = fs.readFileSync(path.join(repoRoot, schemaSource));
    const bundledSchema = {
      file: "node_modules/.vize/vize.config.schema.json",
      bytes: schemaBytes.length,
      sha256: sha256(schemaBytes),
    };
    observation.generatedSchema = { source: schemaSource, ...bundledSchema };
    const version = execute(binary, ["--version"], repoRoot, process.env);
    retainRun("version", version);
    assert.equal(version.error, null);
    assert.equal(version.signal, null);
    assert.equal(version.exitStatus, 0);
    assert.equal(version.stdout.toString("utf8").trim(), expected.cliVersion);
    const upstream = await loadUpstream(repoRoot);
    observation.custody = {
      revision: upstream.custody.revision,
      sourcePins: upstream.custody.sourcePins,
    };
    const plants = upstream.LINT_VALIDITY_PLANTS;
    const files = relativeFiles(plants);
    workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-current-lint-"));
    const input = path.join(workspace, "empty-input");
    fs.mkdirSync(input);
    // Untouched upstream generator, same n200 timed-config label; no Vize config.
    const configRoot = upstream.prepareLintDir(input, [], workspace, "n200");
    observation.configs = verifyConfigs(configRoot);
    observation.plants = plants.map((plant, index) => ({
      id: plant.id,
      file: files[index],
      dirtyLine: plant.dirtyLine,
      dirtySha256: sha256(plant.dirty),
      cleanSha256: sha256(plant.clean),
    }));
    observation.profiles = [];
    for (const profile of PROFILES) {
      const command = upstream.lintCliCommand(profile);
      const env = { ...process.env };
      // The default-pool row must not inherit the single-thread row's override.
      delete env.RAYON_NUM_THREADS;
      Object.assign(env, command.env);
      const result = { profile, command, environment: {}, runs: [], human: [], machine: [] };
      for (const key of ["RAYON_NUM_THREADS", "NO_COLOR", "FORCE_COLOR", "TERM", "CI"]) {
        result.environment[key] = env[key] ?? null;
      }
      observation.profiles.push(result);
      const diagnostics = { human: {}, machine: {} };
      for (const polarity of ["dirty", "clean"]) {
        const cwd = path.join(workspace, profile, polarity);
        fs.mkdirSync(path.join(cwd, ".git"), { recursive: true });
        for (const config of CONFIG_NAMES)
          fs.copyFileSync(path.join(configRoot, config), path.join(cwd, config));
        for (const [index, plant] of plants.entries()) {
          fs.mkdirSync(path.dirname(path.join(cwd, files[index])), { recursive: true });
          fs.writeFileSync(path.join(cwd, files[index]), plant[polarity]);
        }
        const before = inventory(cwd);
        assert.deepEqual(
          before.map((row) => row.file),
          [...CONFIG_NAMES, ...files].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0)),
        );
        verifyConfigs(cwd);
        for (const reporter of ["human", "machine"]) {
          const beforeRun = inventory(cwd);
          verifyWorkInventory(beforeRun, before, bundledSchema, false);
          const args = [...command.args, ...(reporter === "machine" ? ["--format", "json"] : [])];
          const run = execute(binary, args, cwd, env);
          const name = `${profile}-${polarity}-${reporter}`;
          const retained = retainRun(name, run);
          Object.assign(retained, {
            name,
            reporter,
            polarity,
            args,
            cwd,
            inventory: before,
            beforeInventory: beforeRun,
            afterInventory: inventory(cwd),
          });
          result.runs.push(retained);
          if (fs.existsSync(path.join(cwd, bundledSchema.file))) {
            const generatedBytes = fs.readFileSync(path.join(cwd, bundledSchema.file));
            const generatedFile = `${name}.schema.json`;
            fs.writeFileSync(path.join(destination, generatedFile), generatedBytes);
            retained.generatedSchema = {
              file: generatedFile,
              bytes: generatedBytes.length,
              sha256: sha256(generatedBytes),
            };
          }
          assert.equal(run.error, null, `${name}: ${run.error}`);
          assert.equal(run.signal, null, name);
          assert.ok([0, 1].includes(run.exitStatus), `${name}: exit ${run.exitStatus}`);
          verifyWorkInventory(retained.afterInventory, before, bundledSchema, false);
          assert.deepEqual(fs.readdirSync(path.join(cwd, ".git")), [], "repository marker changed");
          verifyConfigs(cwd);
          for (const stream of [run.stdout, run.stderr]) {
            assert.deepEqual(
              Buffer.from(stream.toString("utf8")),
              stream,
              `${name}: invalid UTF-8`,
            );
          }
          if (reporter === "human") {
            // Exact original concatenation/parser; preserve its failures unchanged.
            diagnostics.human[polarity] = upstream.cliDiagnostics(
              `${run.stdout.toString("utf8")}\n${run.stderr.toString("utf8")}`,
            );
          } else {
            const rows = JSON.parse(run.stdout.toString("utf8"));
            diagnostics.machine[polarity] = machineDiagnostics(rows);
            for (const file of files)
              assert.equal(
                rows.filter((row) => row.file === file).length,
                1,
                `missing/duplicate ${file}`,
              );
            retained.completeRows = rows;
            retained.additionalFiles = rows
              .map((row) => row.file)
              .filter((file) => !files.includes(file));
            assert.equal(run.exitStatus, rows.some((row) => row.errorCount > 0) ? 1 : 0);
          }
        }
      }
      for (const reporter of ["human", "machine"]) {
        result[reporter] = plants.map((plant, index) => ({
          id: plant.id,
          file: files[index],
          ...upstream.judgeLintPair(
            plant,
            diagnostics[reporter].dirty,
            diagnostics[reporter].clean,
            files[index],
          ),
        }));
        result[`${reporter}Diagnostics`] = diagnostics[reporter];
      }
    }
    verifyUpstream(repoRoot);
    observation.status = "OBSERVED";
    return observation;
  } catch (error) {
    observation.status = "FAIL";
    observation.failure = error instanceof Error ? error.stack : String(error);
    throw error;
  } finally {
    try {
      fs.writeFileSync(
        path.join(destination, "observation.json"),
        `${JSON.stringify(observation, null, 2)}\n`,
      );
    } finally {
      if (workspace) fs.rmSync(workspace, { recursive: true, force: true });
    }
  }

  function retainRun(name, run) {
    const retained = {
      exitStatus: run.exitStatus,
      signal: run.signal,
      error: run.error,
      streams: {},
    };
    for (const stream of ["stdout", "stderr"]) {
      const file = `${name}.${stream}`;
      fs.writeFileSync(path.join(destination, file), run[stream]);
      retained.streams[stream] = { file, bytes: run[stream].length, sha256: sha256(run[stream]) };
    }
    observation.attempts.push({ name, ...retained });
    return retained;
  }
}

function execute(binary, args, cwd, env) {
  const run = spawnSync(binary, args, { cwd, env, timeout: 30_000, maxBuffer: 8 * 1024 * 1024 });
  return {
    stdout: run.stdout ?? Buffer.alloc(0),
    stderr: run.stderr ?? Buffer.alloc(0),
    exitStatus: run.status,
    signal: run.signal,
    error: run.error?.message ?? null,
  };
}

export function inventory(cwd, directory = "") {
  return fs
    .readdirSync(path.join(cwd, directory), { withFileTypes: true })
    .flatMap((entry) => {
      if (entry.name === ".git" && directory === "") return [];
      const file = path.posix.join(directory, entry.name);
      if (entry.isDirectory()) {
        const children = inventory(cwd, file);
        return children.length ? children : [{ file, kind: "directory" }];
      }
      assert.ok(entry.isFile(), `unexpected input kind: ${file}`);
      const bytes = fs.readFileSync(path.join(cwd, file));
      return [{ file, bytes: bytes.length, sha256: sha256(bytes) }];
    })
    .sort((a, b) => (a.file < b.file ? -1 : a.file > b.file ? 1 : 0));
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const result = await probeCurrentLint();
  for (const profile of result.profiles) {
    console.log(
      `${profile.profile}: original human judge ${profile.human.filter((row) => row.ok).length}/11; supplemental machine judge ${profile.machine.filter((row) => row.ok).length}/11; unranked`,
    );
  }
}
