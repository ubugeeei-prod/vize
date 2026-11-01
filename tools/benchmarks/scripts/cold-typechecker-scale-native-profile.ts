/** One untimed native instrumentation call, after the same-carrier bare replay. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { gunzipSync } from "node:zlib";
import {
  recordNativeBeforeInspection,
  type ReplayOutcome,
} from "./cold-typechecker-scale-native-custody.ts";

const hash = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const write = (file: string, value: unknown) =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
export function assertProfileInputUnchanged(before: unknown, after: unknown) {
  assert.deepEqual(after, before, "profile call changed sealed input");
}
export function validateProfileOutput(
  baseline: ReplayOutcome,
  result: ReplayOutcome,
  directory: string,
) {
  assert.equal(baseline.error, undefined);
  assert.equal(baseline.signal, null);
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, baseline.status);
  assert.deepEqual(result.stderr, baseline.stderr);
  assert(
    Number.isInteger(result.pid) && Number(result.pid) > 0,
    "profile requires actual invoked PID",
  );
  assert(path.isAbsolute(directory) && !/[\r\n]/u.test(directory));
  const cpu = path.join(directory, `${result.pid}-cpuprofile.pb.gz`),
    allocations = path.join(directory, `${result.pid}-memprofile.pb.gz`);
  const suffix = Buffer.from(`Memory profile: ${allocations}\nCPU profile: ${cpu}\n`);
  assert.deepEqual(
    result.stdout,
    Buffer.concat([baseline.stdout ?? Buffer.alloc(0), suffix]),
    "only the two primary-source PID/path suffix lines are admitted",
  );
  return { cpu, allocations, suffixSha256: createHash("sha256").update(suffix).digest("hex") };
}
export function validateProfileNames(members: string[], pid: number) {
  assert.deepEqual(
    members.toSorted(),
    [`${pid}-cpuprofile.pb.gz`, `${pid}-memprofile.pb.gz`].toSorted(),
  );
}
function profileFile(file: string, kind: string) {
  const stat = fs.lstatSync(file);
  assert(stat.isFile() && !stat.isSymbolicLink() && stat.nlink === 1);
  assert(stat.size > 0 && stat.size <= 256 * 1024 * 1024);
  const bytes = fs.readFileSync(file);
  assert.equal(bytes.length, stat.size);
  const expanded = gunzipSync(bytes, { maxOutputLength: 256 * 1024 * 1024 });
  assert(expanded.length > 0);
  return {
    kind,
    path: file,
    bytes: bytes.length,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    uncompressedBytes: expanded.length,
  };
}

export function captureNativeProfile(options: {
  program: string;
  programSha256: string;
  args: string[];
  cwd: string;
  env: NodeJS.ProcessEnv;
  out: string;
  baseline: ReplayOutcome;
  baselineBuildInfo: unknown;
  initialSeal: unknown;
  snapshot: () => unknown;
  retainBuildInfo: (role: string) => unknown;
}) {
  const { program, args, cwd, env, out } = options;
  assert(
    !args.some((arg) => /^--(?:pprofDir|generateTrace|generateCpuProfile)(?:=|$)/iu.test(arg)),
  );
  assert(!fs.existsSync(args[4]), "same owned build-info path must be absent again");
  assert.equal(hash(program), options.programSha256);
  const before = options.snapshot();
  assertProfileInputUnchanged(options.initialSeal, before);
  write(path.join(out, "native-profile.input-before.json"), before);
  const directory = path.join(out, "native-profile-data");
  fs.mkdirSync(directory, { mode: 0o700 });
  assert.equal(fs.realpathSync(directory), directory);
  const profileArgs = [...args, "--pprofDir", directory];
  write(path.join(out, "native-profile.request.json"), {
    program,
    programSha256: options.programSha256,
    args: profileArgs,
    cwd,
    env,
    buildInfoAbsent: true,
    instrumentationOnly: true,
    excludedFromTimings: true,
    sourceLineScope: "image hints only; vcs.modified=true",
  });
  const started = process.hrtime.bigint();
  const result = spawnSync(program, profileArgs, {
    cwd,
    env,
    maxBuffer: 64 * 1024 * 1024,
    timeout: 600_000,
  });
  return recordNativeBeforeInspection(
    out,
    "native-profile",
    result,
    (process.hrtime.bigint() - started).toString(),
    () => {
      const generatedBuildInfo = options.retainBuildInfo("profile");
      write(path.join(out, "native-profile.build-info.json"), generatedBuildInfo);
      const after = options.snapshot();
      write(path.join(out, "native-profile.input-after.json"), after);
      const observed = validateProfileOutput(options.baseline, result, directory);
      const stat = fs.lstatSync(directory);
      assert(stat.isDirectory() && !stat.isSymbolicLink());
      assert.equal(fs.realpathSync(directory), directory);
      validateProfileNames(fs.readdirSync(directory), Number(result.pid));
      const profiles = [
        profileFile(observed.cpu, "cpu"),
        profileFile(observed.allocations, "sampled-allocations-not-live-heap"),
      ];
      write(path.join(out, "native-profile.files.json"), profiles);
      assertProfileInputUnchanged(before, after);
      assert.deepEqual(
        generatedBuildInfo,
        options.baselineBuildInfo,
        "profile changed generated build-info",
      );
      if (fs.existsSync(path.join(out, "replay.tsbuildinfo"))) {
        assert.deepEqual(
          fs.readFileSync(path.join(out, "profile.tsbuildinfo")),
          fs.readFileSync(path.join(out, "replay.tsbuildinfo")),
        );
      }
      assert.equal(hash(program), options.programSha256);
      write(path.join(out, "native-profile.validation.json"), {
        passed: true,
        instrumentationOnly: true,
        excludedFromTimings: true,
        statusEqual: true,
        stderrEqual: true,
        stdoutPlusOnlyExactSuffix: true,
        inputSealsEqual: true,
        buildInfoEqual: true,
        profileFiles: profiles,
        suffixSha256: observed.suffixSha256,
        nativeLoadedFileVector: "unobserved",
        normalizedOptions: "unobserved",
        sourceLineScope: "image hints; vcs.modified=true",
      });
    },
  );
}
