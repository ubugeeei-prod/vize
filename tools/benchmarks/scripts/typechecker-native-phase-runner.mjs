/** Complete byte/map/program/ordered-diagnostic gates for untimed native phase probes. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileSha256 } from "./benchmark-binary.mjs";
import { corpusManifest } from "./type-snapshot-cli-corpus.mjs";
import {
  MODES,
  checkEnvironment,
  normalizeCliReport,
  writeJson,
  compareStrings,
} from "./type-snapshot-cli-protocol.mjs";
import { diagnosticFingerprint } from "./typecheck-command.mjs";
const command = (binary, args, cwd, env = process.env) =>
  spawnSync(binary, args, {
    cwd,
    env,
    encoding: "utf8",
    timeout: 300_000,
    maxBuffer: 256 * 1024 * 1024,
  });
function successful(result, label) {
  assert.equal(result.error, undefined, `${label}: ${result.error?.message}`);
  assert.equal(result.status, 0, `${label}: ${result.stderr}`);
}
export function createPhaseRunner({ root, directory, binaries, wrapper }) {
  let sequence = 0;
  function project(corpus, label) {
    const result = command(
      binaries.projection.measuredPath,
      [corpus.dir],
      root,
      checkEnvironment(MODES[0]),
    );
    const stem = join(directory, "projections", label);
    writeFileSync(`${stem}.json`, result.stdout ?? "");
    writeFileSync(`${stem}.stderr.txt`, result.stderr ?? "");
    successful(result, "projection");
    assert.equal(result.stderr, "");
    const parsed = JSON.parse(result.stdout);
    const expected = corpusManifest(corpus.dir)
      .files.filter((file) => file.file.endsWith(".vue"))
      .map((file) => file.file);
    assert.deepEqual(
      parsed.files.map((file) => file.file),
      expected,
      "mapping probe omitted inputs",
    );
    return { text: result.stdout, sha256: fileSha256(`${stem}.json`) };
  }
  function run(corpus, mode, wrapped, label, captureVirtual = false) {
    const id = `${String(sequence++).padStart(3, "0")}-${label}-${wrapped ? "wrapped" : "direct"}`;
    const nativeDir = join(directory, "native", id);
    const settingsPath = join(directory, "raw", `${id}.settings.json`);
    writeJson(settingsPath, { directory: nativeDir, runtime: binaries.native.measuredPath });
    const expected = readdirSync(corpus.dir).filter((file) => file.endsWith(".vue"));
    const authoredInputs = corpusManifest(corpus.dir)
      .files.filter((file) => /\.(?:vue|[cm]?[jt]sx?)$/u.test(file.file))
      .map((file) => file.file);
    const virtualTargets = captureVirtual
      ? [...expected.map((file) => `${file}.virtual.ts`), "__vize_helpers.d.ts"]
      : [];
    for (const file of virtualTargets)
      assert(!existsSync(join(corpus.dir, file)), `unexpected old virtual output ${file}`);
    try {
      const args = [
        "check",
        ...(corpus.args ?? ["."]),
        "--quiet",
        "--format",
        "json",
        "--tsconfig",
        "tsconfig.json",
        "--corsa-path",
        wrapped ? wrapper : binaries.native.measuredPath,
        ...mode.args,
      ];
      if (captureVirtual)
        for (const file of [...expected, "__vize_helpers.d.ts"])
          args.push("--save-virtual-ts-for", file);
      const env = checkEnvironment(mode);
      env.NATIVE_PHASE_CONFIG = settingsPath;
      const result = command(binaries.vize.measuredPath, args, corpus.dir, env);
      writeFileSync(join(directory, "raw", `${id}.stdout.json`), result.stdout ?? "");
      writeFileSync(join(directory, "raw", `${id}.stderr.txt`), result.stderr ?? "");
      const record = {
        id,
        wrapped,
        command: [binaries.vize.measuredPath, ...args],
        cwd: corpus.dir,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        environment: {
          RAYON_NUM_THREADS: env.RAYON_NUM_THREADS ?? null,
          GOMAXPROCS: env.GOMAXPROCS ?? null,
        },
        excludedFromTimings: true,
      };
      writeJson(join(directory, "raw", `${id}.json`), record);
      assert.equal(result.error, undefined, result.error?.message);
      const normalized = normalizeCliReport(result, corpus.dir, expected);
      record.normalized = normalized;
      record.fingerprint = diagnosticFingerprint(normalized);
      if (captureVirtual) {
        record.virtualFiles = [];
        for (const file of virtualTargets) {
          const from = join(corpus.dir, file);
          assert(existsSync(from), `missing virtual output ${file}`);
          const target = join(directory, "virtual-ts", id, file);
          mkdirSync(dirname(target), { recursive: true });
          copyFileSync(from, target);
          record.virtualFiles.push({
            file,
            bytes: statSync(target).size,
            sha256: fileSha256(target),
          });
        }
      }
      if (wrapped) {
        assert(existsSync(nativeDir), "wrapper produced no native shard evidence");
        record.nativeShards = readdirSync(nativeDir)
          .filter((file) => /^\d+\.json$/u.test(file))
          .map((file) => ({
            record: relative(directory, join(nativeDir, file)),
            ...JSON.parse(readFileSync(join(nativeDir, file), "utf8")),
          }))
          .sort((a, b) => a.config.localeCompare(b.config, "en"));
        assert(record.nativeShards.length > 0, "no native shard receipt");
        if (captureVirtual)
          for (const file of record.virtualFiles) {
            const nativeName = file.file.replace(/\.virtual\.ts$/u, ".ts");
            const matches = record.nativeShards.flatMap((shard) =>
              shard.members.filter((member) => relative(shard.cwd, member.path) === nativeName),
            );
            assert(matches.length > 0, `saved output absent from native program: ${file.file}`);
            assert(
              matches.every(
                (member) => member.sha256 === file.sha256 && member.bytes === file.bytes,
              ),
              `saved output differs from native program bytes: ${file.file}`,
            );
          }
        const covered = new Set(
          record.nativeShards.flatMap((shard) =>
            shard.members
              .filter((file) => file.path.endsWith(".vue.ts"))
              .map((file) => relative(shard.cwd, file.path).replace(/\.ts$/u, "")),
          ),
        );
        assert.deepEqual(
          [...covered].sort(compareStrings),
          [...expected].sort(compareStrings),
          "actual native shards omitted Vue sources",
        );
        const programSources = new Set(normalized.programs.flatMap((program) => program.files));
        for (const file of authoredInputs) {
          assert(programSources.has(file), `frontend program omitted authored input: ${file}`);
          const nativeName = file.endsWith(".vue") ? `${file}.ts` : file;
          assert(
            record.nativeShards.some((shard) =>
              shard.members.some((member) => relative(shard.cwd, member.path) === nativeName),
            ),
            `native program omitted authored input: ${file}`,
          );
        }
      }
      writeJson(join(directory, "raw", `${id}.json`), record);
      return {
        ...record,
        report: JSON.parse(result.stdout),
        stdout: result.stdout,
        stderr: result.stderr,
      };
    } finally {
      for (const file of virtualTargets) rmSync(join(corpus.dir, file), { force: true });
    }
  }
  function pair(corpus, mode, label, virtual = false) {
    const manifest = corpusManifest(corpus.dir);
    const snapshot = join(directory, "inputs", label);
    mkdirSync(snapshot, { recursive: true });
    for (const file of manifest.files) {
      // Initial full source archives already retain unchanged corpus bytes.
      if (
        corpus.manifest.files.some(
          (original) => original.file === file.file && original.sha256 === file.sha256,
        )
      )
        continue;
      const target = join(snapshot, file.file);
      mkdirSync(dirname(target), { recursive: true });
      copyFileSync(join(corpus.dir, file.file), target);
    }
    writeJson(join(snapshot, "manifest.json"), manifest);
    const direct = run(corpus, mode, false, label, virtual);
    const wrapped = run(corpus, mode, true, label, virtual);
    assert.equal(
      wrapped.fingerprint,
      direct.fingerprint,
      `${label}: full ordered CLI report differs`,
    );
    if (virtual)
      assert.deepEqual(
        wrapped.virtualFiles,
        direct.virtualFiles,
        `${label}: actual virtual bytes differ`,
      );
    assert.equal(
      corpusManifest(corpus.dir).sha256,
      manifest.sha256,
      `${label}: probe changed fixture inputs`,
    );
    return { direct, wrapped };
  }
  return { project, pair };
}
